use crate::config::{BackupTarget, PgDumpConfig};
use crate::error::{BackupError, BackupResult};
use crate::storage::StorageBackendTrait;
use agrocore_logging::{error, info, warn};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

/// Chunk size used when streaming dumps to storage.
const STREAM_BUFFER_SIZE: usize = 1024 * 1024;

pub struct PgDump {
    config: PgDumpConfig,
    database_url: String,
}

impl Clone for PgDump {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            database_url: self.database_url.clone(),
        }
    }
}

impl PgDump {
    pub fn new(_pool: sqlx::PgPool, config: PgDumpConfig) -> Self {
        Self {
            config,
            database_url: std::env::var("DATABASE_URL").unwrap_or_default(),
        }
    }

    /// Build the pg_dump argument list from the configuration.
    fn dump_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "--format=custom".to_string(),
            "--no-owner".to_string(),
            "--no-privileges".to_string(),
            "--no-comments".to_string(),
            "--no-security-labels".to_string(),
            "--no-tablespaces".to_string(),
            format!("--compress={}", self.config.compression_level),
        ];

        for table in &self.config.exclude_tables {
            args.push("--exclude-table".to_string());
            args.push(table.clone());
        }
        for table in &self.config.include_tables {
            args.push("--table".to_string());
            args.push(table.clone());
        }
        if let Some(jobs) = self.config.jobs {
            args.push("--jobs".to_string());
            args.push(jobs.to_string());
        }
        args.push(self.database_url.clone());
        args
    }

    /// Stream a database dump into `storage`.
    ///
    /// The dump is piped through a fixed-size buffer instead of being buffered
    /// in memory, so databases larger than available RAM can be backed up. The
    /// returned size is the number of bytes actually uploaded.
    pub async fn dump_to_storage(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &BackupTarget,
        prefix: &str,
        job_id: Uuid,
    ) -> BackupResult<u64> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        // The job id is part of the object name so a backup can be located
        // without listing every object in the bucket.
        let dump_name = format!("{prefix}{job_id}_dump_{timestamp}.dump");

        info!("Starting pg_dump for job {job_id}");

        let mut cmd = tokio::process::Command::new("pg_dump");
        cmd.args(self.dump_args())
            // stdout carries the dump, stderr carries diagnostics.
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| BackupError::postgres(format!("failed to start pg_dump: {e}")))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| BackupError::postgres("pg_dump produced no stdout".to_string()))?;

        // Pump stdout -> storage in fixed-size chunks. `upload_stream` takes an
        // async byte stream, so memory use stays bounded regardless of dump size.
        let chunks = async_stream::stream! {
            let mut buf = vec![0u8; STREAM_BUFFER_SIZE];
            loop {
                let read = match stdout.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading pg_dump output: {e}");
                        break;
                    }
                };
                yield Ok::<bytes::Bytes, BackupError>(bytes::Bytes::copy_from_slice(&buf[..read]));
            }
        };

        let size = storage
            .upload_stream(target, &dump_name, Box::pin(chunks))
            .await?;

        // Collect stderr for the error message, then reap the child.
        let mut stderr = String::new();
        if let Some(mut err) = child.stderr.take() {
            use tokio::io::AsyncReadExt;
            let _ = err.read_to_string(&mut stderr).await;
        }
        let status = child.wait().await.map_err(BackupError::Io)?;

        if !status.success() {
            return Err(BackupError::postgres(format!(
                "pg_dump failed ({status}): {stderr}"
            )));
        }

        if size == 0 {
            return Err(BackupError::postgres(
                "pg_dump produced an empty dump".to_string(),
            ));
        }

        info!("pg_dump completed for job {job_id}: {size} bytes");
        Ok(size)
    }

    /// Restore a dump by streaming it into `pg_restore` via stdin.
    ///
    /// The dump is never held in memory: bytes are read from storage in chunks
    /// and written to the child process as they arrive.
    pub async fn restore_from_storage(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &BackupTarget,
        object_name: &str,
        target_db: Option<String>,
    ) -> BackupResult<()> {
        let dbname = target_db.unwrap_or_else(|| self.database_url.clone());

        let mut cmd = tokio::process::Command::new("pg_restore");
        cmd.arg("--no-owner")
            .arg("--no-privileges")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--no-comments")
            .arg("--dbname")
            .arg(&dbname)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| BackupError::postgres(format!("failed to start pg_restore: {e}")))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| BackupError::postgres("pg_restore has no stdin".to_string()))?;

        // Pull the object from storage in chunks and feed it to pg_restore.
        let chunks = storage.download_stream(target, object_name).await?;
        let mut chunks = chunks;

        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            stdin.write_all(&chunk).await.map_err(BackupError::Io)?;
        }
        // pg_restore needs EOF before it finishes writing the database.
        stdin.shutdown().await.map_err(BackupError::Io)?;
        drop(stdin);

        let output = child.wait_with_output().await.map_err(BackupError::Io)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // A newer pg_dump against an older server emits `SET` statements for
            // parameters that server does not know (e.g. transaction_timeout,
            // which arrived in PostgreSQL 17). pg_restore logs those as errors
            // yet continues the restore, so they must not fail the backup.
            // Any other `pg_restore: error:` line is a real failure.
            let unknown_params_only = stderr.contains("unrecognized configuration parameter")
                && !stderr.contains("pg_restore: error:");

            if unknown_params_only {
                warn!(
                    "pg_restore completed with unknown-parameter warnings (client/server version \
                     mismatch): {stderr}"
                );
            } else {
                return Err(BackupError::postgres(format!(
                    "pg_restore failed ({}): {stderr}",
                    output.status
                )));
            }
        }

        info!("pg_restore completed from {object_name}");
        Ok(())
    }
}
