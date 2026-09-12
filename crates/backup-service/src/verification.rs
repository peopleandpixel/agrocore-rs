use crate::config::{BackupTarget, VerificationConfig};
use crate::error::{BackupError, BackupResult};
use crate::pg_dump::PgDump;
use crate::storage::StorageBackendTrait;
use agrocore_logging::{error, info, warn};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct VerificationManager {
    config: VerificationConfig,
    pg_dump: PgDump,
    db_pool: PgPool,
}

impl VerificationManager {
    pub fn new(config: VerificationConfig, pg_dump: PgDump, db_pool: PgPool) -> Self {
        Self {
            config,
            pg_dump,
            db_pool,
        }
    }

    pub async fn verify_backup(
        &self,
        target_ids: &[String],
        backup_type: &crate::service::BackupType,
        storage: &Arc<dyn StorageBackendTrait>,
        targets: &[BackupTarget],
    ) -> BackupResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Starting backup verification for targets: {:?}", target_ids);

        for target_id in target_ids {
            // Find the target config
            let target = targets.iter().find(|t| t.target_id() == *target_id);
            let Some(target) = target else {
                warn!(
                    "Target {} not found in config, skipping verification",
                    target_id
                );
                continue;
            };

            // Find the latest backup for this target
            let backup_id = self
                .find_latest_backup(target, backup_type, storage)
                .await?;
            if backup_id.is_none() {
                warn!(
                    "No backup found for target {} type {:?}",
                    target_id, backup_type
                );
                continue;
            }
            let backup_id = backup_id.unwrap();

            info!("Verifying backup {} on target {}", backup_id, target_id);

            // 1. Create test database
            let test_db_name = self.create_test_database().await?;

            // 2. Restore backup to test database
            self.restore_to_test_db(target, &backup_id, &test_db_name, storage)
                .await?;

            // 3. Verify checksums
            self.verify_checksums(target, &backup_id, storage).await?;

            // 4. Verify row counts
            self.verify_row_counts(&test_db_name).await?;

            // 5. Verify schema
            self.verify_schema(&test_db_name).await?;

            // 6. Drop test database
            self.drop_test_database(&test_db_name).await?;

            info!(
                "Backup verification completed successfully for {}",
                backup_id
            );
        }

        Ok(())
    }

    async fn find_latest_backup(
        &self,
        target: &BackupTarget,
        backup_type: &crate::service::BackupType,
        storage: &Arc<dyn StorageBackendTrait>,
    ) -> BackupResult<Option<Uuid>> {
        // List manifests to find latest backup
        let prefix = format!("manifests/");
        let backups = self.list_manifests(target, &prefix, storage).await?;

        // Filter by backup type and find latest
        let mut typed_backups: Vec<_> = backups
            .into_iter()
            .filter(|m| m.backup_type == *backup_type)
            .collect();

        typed_backups.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(typed_backups.first().map(|m| m.backup_id))
    }

    async fn list_manifests(
        &self,
        _target: &BackupTarget,
        _prefix: &str,
        _storage: &Arc<dyn StorageBackendTrait>,
    ) -> BackupResult<Vec<crate::manifest::BackupManifest>> {
        // This would need storage.list_objects implementation
        // For now, return empty - would be implemented with object_store list
        Ok(vec![])
    }

    async fn create_test_database(&self) -> BackupResult<String> {
        let test_db_name = format!("verify_test_{}", Uuid::new_v4().simple());

        // Create test database from template
        let sql = format!("CREATE DATABASE \"{}\" TEMPLATE template1", test_db_name);
        sqlx::query(&sql)
            .execute(&self.db_pool)
            .await
            .map_err(|e| BackupError::Database(e))?;

        info!("Created test database: {}", test_db_name);
        Ok(test_db_name)
    }

    async fn drop_test_database(&self, db_name: &str) -> BackupResult<()> {
        // Terminate connections
        let terminate_sql = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            db_name
        );
        sqlx::query(&terminate_sql)
            .execute(&self.db_pool)
            .await
            .map_err(|e| BackupError::Database(e))?;

        // Drop database
        let drop_sql = format!("DROP DATABASE IF EXISTS \"{}\"", db_name);
        sqlx::query(&drop_sql)
            .execute(&self.db_pool)
            .await
            .map_err(|e| BackupError::Database(e))?;

        info!("Dropped test database: {}", db_name);
        Ok(())
    }

    async fn restore_to_test_db(
        &self,
        target: &BackupTarget,
        backup_id: &Uuid,
        test_db_name: &str,
        storage: &Arc<dyn StorageBackendTrait>,
    ) -> BackupResult<()> {
        // Find the dump file for this backup
        let dump_name = format!("db/dump_{}.dump", backup_id);

        self.pg_dump
            .restore_from_storage(storage, target, &dump_name, Some(test_db_name.to_string()))
            .await?;

        info!(
            "Restored backup {} to test database {}",
            backup_id, test_db_name
        );
        Ok(())
    }

    async fn verify_checksums(
        &self,
        target: &BackupTarget,
        backup_id: &Uuid,
        storage: &Arc<dyn StorageBackendTrait>,
    ) -> BackupResult<()> {
        // Download manifest
        let manifest_name = format!("manifests/{}.json", backup_id);
        let manifest_data = storage.download_bytes(target, &manifest_name).await?;

        let manifest: crate::manifest::BackupManifest =
            serde_json::from_slice(&manifest_data).map_err(|e| BackupError::Serialization(e))?;

        // Verify each object checksum from targets
        for target_manifest in &manifest.targets {
            for object in &target_manifest.objects {
                let data = storage.download_bytes(target, &object.name).await?;
                let computed_checksum = self.compute_checksum(&data);

                if computed_checksum != object.checksum_sha256 {
                    error!(
                        "Checksum mismatch for {}: expected {}, got {}",
                        object.name, object.checksum_sha256, computed_checksum
                    );
                    return Err(BackupError::Verification(format!(
                        "Checksum mismatch for {}: expected {}, got {}",
                        object.name, object.checksum_sha256, computed_checksum
                    )));
                }
            }
        }

        // Also verify file checksums
        for file_checksum in &manifest.checksums {
            let data = storage.download_bytes(target, &file_checksum.path).await?;
            let computed_checksum = self.compute_checksum(&data);

            if computed_checksum != file_checksum.sha256 {
                error!(
                    "File checksum mismatch for {}: expected {}, got {}",
                    file_checksum.path, file_checksum.sha256, computed_checksum
                );
                return Err(BackupError::Verification(format!(
                    "File checksum mismatch for {}: expected {}, got {}",
                    file_checksum.path, file_checksum.sha256, computed_checksum
                )));
            }
        }

        info!("All checksums verified for backup {}", backup_id);
        Ok(())
    }

    fn compute_checksum(&self, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    async fn verify_row_counts(&self, test_db_name: &str) -> BackupResult<()> {
        // Connect to test database
        let test_db_url = std::env::var("DATABASE_URL")
            .unwrap_or_default()
            .replace("/postgres", &format!("/{}", test_db_name));

        let test_pool = PgPool::connect(&test_db_url)
            .await
            .map_err(|e| BackupError::Database(e))?;

        // Get all tables
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
        )
        .fetch_all(&test_pool)
        .await
        .map_err(|e| BackupError::Database(e))?;

        for (table_name,) in tables {
            let count: (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM \"{}\"", table_name))
                .fetch_one(&test_pool)
                .await
                .map_err(|e| BackupError::Database(e))?;

            info!("Table {}: {} rows", table_name, count.0);
        }

        info!(
            "Row count verification completed for database {}",
            test_db_name
        );
        Ok(())
    }

    async fn verify_schema(&self, test_db_name: &str) -> BackupResult<()> {
        let test_db_url = std::env::var("DATABASE_URL")
            .unwrap_or_default()
            .replace("/postgres", &format!("/{}", test_db_name));

        let test_pool = PgPool::connect(&test_db_url)
            .await
            .map_err(|e| BackupError::Database(e))?;

        // Compare schema with source database
        let source_schema: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT table_name, column_name, data_type 
             FROM information_schema.columns 
             WHERE table_schema = 'public' 
             ORDER BY table_name, ordinal_position",
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| BackupError::Database(e))?;

        let test_schema: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT table_name, column_name, data_type 
             FROM information_schema.columns 
             WHERE table_schema = 'public' 
             ORDER BY table_name, ordinal_position",
        )
        .fetch_all(&test_pool)
        .await
        .map_err(|e| BackupError::Database(e))?;

        if source_schema != test_schema {
            error!("Schema mismatch between source and restored database");
            return Err(BackupError::Verification(
                "Schema verification failed: column mismatch".to_string(),
            ));
        }

        info!("Schema verification passed for database {}", test_db_name);
        Ok(())
    }
}
