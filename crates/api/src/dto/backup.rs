//! Backup API Data Transfer Objects

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateBackupRequest {
    #[validate(length(min = 1))]
    pub backup_type: String, // "database", "config", "full"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BackupResponse {
    pub id: Uuid,
    pub backup_type: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub target_ids: Vec<String>,
    pub total_size_bytes: u64,
    pub error: Option<String>,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BackupSummaryResponse {
    pub id: Uuid,
    pub backup_type: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_size_bytes: u64,
    pub target_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RestoreRequest {
    pub backup_id: Uuid,
    pub target_database: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RestoreResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BackupConfigResponse {
    pub enabled: bool,
    pub schedule_db: String,
    pub schedule_config: String,
    pub timezone: String,
    pub targets_count: usize,
    pub retention_daily: u32,
    pub retention_weekly: u32,
    pub retention_monthly: u32,
    pub retention_yearly: u32,
    pub verification_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateBackupConfigRequest {
    pub enabled: Option<bool>,
    pub schedule_db: Option<String>,
    pub schedule_config: Option<String>,
    pub timezone: Option<String>,
}
