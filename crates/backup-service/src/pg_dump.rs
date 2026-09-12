use crate::config::{BackupTarget, PgDumpConfig};
use crate::error::{BackupError, BackupResult};
use crate::storage::StorageBackendTrait;
use agrocore_logging::info;
use std::sync::Arc;
use uuid::Uuid;

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

    pub async fn dump_to_storage(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &BackupTarget,
        prefix: &str,
        job_id: Uuid,
    ) -> BackupResult<u64> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let dump_name = format!("{}dump_{}.dump", prefix, timestamp);

        info!("Starting pg_dump for job {}", job_id);

        // Build pg_dump command
        let mut cmd = tokio::process::Command::new("pg_dump");
        cmd.arg("--format=custom")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg("--no-comments")
            .arg("--no-security-labels")
            .arg("--no-tablespaces")
            .arg("--compress=6")
            .arg(&self.database_url);

        // Add exclude tables
        for table in &self.config.exclude_tables {
            cmd.arg("--exclude-table").arg(table);
        }

        // Add include tables
        for table in &self.config.include_tables {
            cmd.arg("--table").arg(table);
        }

        // Add jobs if specified
        if let Some(jobs) = self.config.jobs {
            cmd.arg("--jobs").arg(jobs.to_string());
        }

        let output = cmd.output().await.map_err(|e| BackupError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::postgres(stderr.to_string()));
        }

        let dump_data = output.stdout;
        let size = dump_data.len() as u64;

        // Upload to storage
        storage.upload_bytes(target, &dump_name, &dump_data).await?;

        info!("pg_dump completed for job {}: {} bytes", job_id, size);
        Ok(size)
    }

    pub async fn restore_from_storage(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &BackupTarget,
        object_name: &str,
        target_db: Option<String>,
    ) -> BackupResult<()> {
        // Download dump file
        let dump_data = storage.download_bytes(target, object_name).await?;

        // Build pg_restore command
        let mut cmd = tokio::process::Command::new("pg_restore");
        cmd.arg("--no-owner")
            .arg("--no-privileges")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--dbname")
            .arg(target_db.unwrap_or_else(|| self.database_url.clone()));

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| BackupError::Io(e))?;

        // Write dump data to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(&dump_data)
                .await
                .map_err(|e| BackupError::Io(e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| BackupError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::postgres(stderr.to_string()));
        }

        Ok(())
    }
}

use sqlx;
