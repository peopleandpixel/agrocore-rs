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
    /// False when no manifest was found and the id and type were inferred from
    /// the object name.
    pub manifest_backed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RestoreRequest {
    /// Kept for compatibility with clients that echo the id they just called
    /// with; the path segment is authoritative and the two must agree.
    pub backup_id: Uuid,
    /// Validate that the backup is restorable without writing to the database.
    #[serde(default)]
    pub dry_run: bool,
}

// There is deliberately no `target_database` field. `pg_restore` runs with
// `--clean`, so the destination is emptied before the dump is written; a target
// named by the client would let a restore wipe any database on the server. The
// destination is the configured `DATABASE_URL`, server-side, and is not
// something a request can influence.

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

/// Partial update: every field is optional, and only the ones present are
/// written. The retention and verification fields existed on the response but
/// not here, so an admin could read them and never change them.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateBackupConfigRequest {
    pub enabled: Option<bool>,
    #[validate(length(min = 9, max = 100))]
    pub schedule_db: Option<String>,
    #[validate(length(min = 9, max = 100))]
    pub schedule_config: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    /// 0 disables that retention tier rather than deleting everything: the
    /// retention sweep only removes entries older than the tier, so a zero
    /// window means "keep all".
    #[validate(range(min = 0, max = 3650))]
    pub retention_daily: Option<u32>,
    #[validate(range(min = 0, max = 520))]
    pub retention_weekly: Option<u32>,
    #[validate(range(min = 0, max = 120))]
    pub retention_monthly: Option<u32>,
    #[validate(range(min = 0, max = 30))]
    pub retention_yearly: Option<u32>,
    pub verification_enabled: Option<bool>,
}
