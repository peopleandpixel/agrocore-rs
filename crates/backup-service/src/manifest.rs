use crate::config::{BackupConfig, BackupMetadataConfig, BackupTarget};
use crate::error::{BackupError, BackupResult};
use crate::service::{BackupStatus, BackupType};
use crate::storage::StorageBackendTrait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub backup_id: Uuid,
    pub backup_type: crate::service::BackupType,
    pub status: crate::service::BackupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub targets: Vec<TargetManifest>,
    pub total_size_bytes: u64,
    pub schema_version: Option<String>,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub app_version: String,
    pub checksums: Vec<FileChecksum>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetManifest {
    pub target_id: String,
    pub target_type: String,
    pub objects: Vec<ObjectManifest>,
    pub size_bytes: u64,
    pub encryption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectManifest {
    pub name: String,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecksum {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

pub struct ManifestManager {
    config: BackupMetadataConfig,
}

impl ManifestManager {
    pub fn new(config: BackupMetadataConfig) -> Self {
        Self { config }
    }

    pub async fn create_manifest(
        &self,
        backup_id: Uuid,
        backup_type: &crate::service::BackupType,
        target_ids: &[String],
        total_size_bytes: u64,
        started_at: chrono::DateTime<Utc>,
        completed_at: chrono::DateTime<Utc>,
    ) -> BackupResult<BackupManifest> {
        let git_commit = if self.config.include_git_info {
            Self::get_git_commit().await.ok()
        } else {
            None
        };

        let git_branch = if self.config.include_git_info {
            Self::get_git_branch().await.ok()
        } else {
            None
        };

        let schema_version = if self.config.include_schema_version {
            Self::get_schema_version().await.ok()
        } else {
            None
        };

        let manifest = BackupManifest {
            backup_id,
            backup_type: backup_type.clone(),
            status: crate::service::BackupStatus::Completed,
            started_at,
            completed_at,
            targets: vec![], // Would be populated with actual target data
            total_size_bytes,
            schema_version,
            git_commit,
            git_branch,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            checksums: vec![],
            metadata: self.config.custom_fields.clone(),
        };

        Ok(manifest)
    }

    pub async fn save_manifest(
        &self,
        storage: &Arc<dyn StorageBackendTrait>,
        target: &crate::config::BackupTarget,
        manifest: &BackupManifest,
    ) -> crate::error::BackupResult<()> {
        // TODO: Serialize and save manifest to storage
        Ok(())
    }

    async fn get_git_commit() -> anyhow::Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn get_git_branch() -> anyhow::Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn get_schema_version() -> anyhow::Result<String> {
        // Could query database for migration version
        Ok("unknown".to_string())
    }
}
