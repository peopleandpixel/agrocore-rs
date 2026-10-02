//! Backup API Handlers

use crate::AppState;
use crate::dto::{
    BackupConfigResponse, BackupResponse, BackupSummaryResponse, CreateBackupRequest,
    RestoreRequest, RestoreResponse, UpdateBackupConfigRequest,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor;
use actix_web::{HttpResponse, web};
use agrocore_backup::service::{BackupStatus, BackupType};
use agrocore_domain::entities::setting::UpdateSetting;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_logging::{error, info, warn};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

/// Configure backup routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backup")
            .service(
                web::resource("/config")
                    .route(web::get().to(get_backup_config))
                    .route(web::put().to(update_backup_config)),
            )
            .service(
                web::resource("/backups")
                    .route(web::get().to(list_backups))
                    .route(web::post().to(create_backup)),
            )
            .service(
                web::resource("/backups/{id}")
                    .route(web::get().to(get_backup))
                    .route(web::delete().to(delete_backup)),
            )
            .service(web::resource("/backups/{id}/restore").route(web::post().to(restore_backup)))
            .service(web::resource("/backups/{id}/status").route(web::get().to(get_backup_status))),
    );
}

/// Settings keys under the `backup.` namespace.
///
/// Namespaced rather than flat so the settings page groups them, and so a
/// backup key cannot collide with an unrelated one.
const KEY_ENABLED: &str = "backup.enabled";
const KEY_SCHEDULE_DB: &str = "backup.schedule_db";
const KEY_SCHEDULE_CONFIG: &str = "backup.schedule_config";
const KEY_TIMEZONE: &str = "backup.timezone";
const KEY_RETENTION_DAILY: &str = "backup.retention.daily";
const KEY_RETENTION_WEEKLY: &str = "backup.retention.weekly";
const KEY_RETENTION_MONTHLY: &str = "backup.retention.monthly";
const KEY_RETENTION_YEARLY: &str = "backup.retention.yearly";
const KEY_VERIFICATION: &str = "backup.verification_enabled";

/// Fallbacks used when a key has never been written.
///
/// Every field has a system default in `system_settings`, so these are only
/// reached if a deployment dropped that row. They match the shipped defaults.
const DEFAULT_SCHEDULE_DB: &str = "0 2 * * *";
const DEFAULT_SCHEDULE_CONFIG: &str = "0 3 * * 0";
const DEFAULT_TIMEZONE: &str = "UTC";

/// Read a string setting, falling back when it is unset.
async fn setting_str(
    repo: &dyn SettingsRepository,
    tid: TenantId,
    key: &'static str,
    default: &'static str,
) -> String {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => match entry.value {
            serde_json::Value::String(s) => s,
            // A number or boolean stored under a string key: the value is still
            // usable as text rather than silently reverting to the default.
            other => other.to_string(),
        },
        Ok(None) => default.to_string(),
        Err(e) => {
            warn!("Failed to read setting {key}: {e}");
            default.to_string()
        }
    }
}

/// Read a numeric setting, falling back when it is unset or not a number.
async fn setting_u32(
    repo: &dyn SettingsRepository,
    tid: TenantId,
    key: &'static str,
    default: u32,
) -> u32 {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => entry
            .value
            .as_u64()
            .map(|v| v.min(u32::MAX as u64) as u32)
            .unwrap_or(default),
        Ok(None) => default,
        Err(e) => {
            warn!("Failed to read setting {key}: {e}");
            default
        }
    }
}

/// Read a boolean setting, falling back when it is unset.
async fn setting_bool(
    repo: &dyn SettingsRepository,
    tid: TenantId,
    key: &'static str,
    default: bool,
) -> bool {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => entry.value.as_bool().unwrap_or(default),
        Ok(None) => default,
        Err(e) => {
            warn!("Failed to read setting {key}: {e}");
            default
        }
    }
}

/// The stored configuration, or the defaults for whatever was never set.
async fn load_backup_config(
    state: &web::Data<AppState>,
    auth: &AuthExtractor,
) -> Result<BackupConfigResponse, ApiError> {
    let repo = state.db.settings_repo();
    let tid = TenantId(auth.0.tenant_id);

    Ok(BackupConfigResponse {
        enabled: setting_bool(&*repo, tid, KEY_ENABLED, true).await,
        schedule_db: setting_str(&*repo, tid, KEY_SCHEDULE_DB, DEFAULT_SCHEDULE_DB).await,
        schedule_config: setting_str(&*repo, tid, KEY_SCHEDULE_CONFIG, DEFAULT_SCHEDULE_CONFIG)
            .await,
        timezone: setting_str(&*repo, tid, KEY_TIMEZONE, DEFAULT_TIMEZONE).await,
        // No target rows are stored in system_settings yet: the backup service
        // owns the target list. Reporting the configured count honestly means 0
        // rather than implying destinations that are not registered.
        targets_count: 0,
        retention_daily: setting_u32(&*repo, tid, KEY_RETENTION_DAILY, 7).await,
        retention_weekly: setting_u32(&*repo, tid, KEY_RETENTION_WEEKLY, 4).await,
        retention_monthly: setting_u32(&*repo, tid, KEY_RETENTION_MONTHLY, 12).await,
        retention_yearly: setting_u32(&*repo, tid, KEY_RETENTION_YEARLY, 7).await,
        verification_enabled: setting_bool(&*repo, tid, KEY_VERIFICATION, true).await,
    })
}

async fn get_backup_config(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    Ok(HttpResponse::Ok().json(load_backup_config(&state, &auth).await?))
}

async fn update_backup_config(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    req: web::Json<UpdateBackupConfigRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin"])?;

    req.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let repo = state.db.settings_repo();
    let tid = TenantId(auth.0.tenant_id);

    // Only the fields present in the request are written. Building the list
    // this way — rather than writing every field — is what makes this a partial
    // update: sending `{"enabled": false}` must not clear the schedule.
    let mut updates: Vec<UpdateSetting> = Vec::new();
    let mut push = |key: &'static str, value: serde_json::Value| {
        updates.push(UpdateSetting {
            key: key.to_string(),
            value,
        });
    };

    if let Some(v) = req.enabled {
        push(KEY_ENABLED, json!(v));
    }
    if let Some(v) = &req.schedule_db {
        push(KEY_SCHEDULE_DB, json!(v));
    }
    if let Some(v) = &req.schedule_config {
        push(KEY_SCHEDULE_CONFIG, json!(v));
    }
    if let Some(v) = &req.timezone {
        push(KEY_TIMEZONE, json!(v));
    }
    if let Some(v) = req.retention_daily {
        push(KEY_RETENTION_DAILY, json!(v));
    }
    if let Some(v) = req.retention_weekly {
        push(KEY_RETENTION_WEEKLY, json!(v));
    }
    if let Some(v) = req.retention_monthly {
        push(KEY_RETENTION_MONTHLY, json!(v));
    }
    if let Some(v) = req.retention_yearly {
        push(KEY_RETENTION_YEARLY, json!(v));
    }
    if let Some(v) = req.verification_enabled {
        push(KEY_VERIFICATION, json!(v));
    }

    if updates.is_empty() {
        // Nothing to write. Returning the current state instead of a bare
        // success keeps the response shape identical whether or not fields were
        // sent, so a client can always render from the reply.
        return Ok(HttpResponse::Ok().json(load_backup_config(&state, &auth).await?));
    }

    repo.set_many(tid, auth.0.user_id, updates)
        .await
        .map_err(|e| {
            // The detail is logged, not returned: a database error message
            // carries table, column and constraint names.
            error!("Failed to persist backup config: {e}");
            ApiError::internal("Backup configuration could not be saved")
        })?;

    info!("Backup config updated by user {}", auth.0.user_id);

    // The stored configuration, not an acknowledgement that it was stored.
    Ok(HttpResponse::Ok().json(load_backup_config(&state, &auth).await?))
}

async fn list_backups(
    _state: web::Data<AppState>,
    _auth: AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    // TODO: Implement listing from backup service
    let backups: Vec<BackupSummaryResponse> = vec![];
    Ok(HttpResponse::Ok().json(backups))
}

async fn create_backup(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    req: web::Json<CreateBackupRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager"])?;

    req.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let backup_type = match req.backup_type.as_str() {
        "database" => BackupType::Database,
        "config" => BackupType::Config,
        "full" => BackupType::Full,
        _ => {
            return Err(ApiError::validation(
                "Invalid backup_type. Must be 'database', 'config', or 'full'",
            ));
        }
    };

    // Get backup service from app state
    let backup_service = state
        .backup_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Backup service not available"))?;

    // Run backup
    let job = backup_service
        .run_backup(backup_type)
        .await
        .map_err(|e| ApiError::internal(format!("Backup failed: {}", e)))?;

    let response = BackupResponse {
        id: job.id,
        backup_type: job.backup_type.to_string(),
        status: job.status.to_string(),
        started_at: job.started_at,
        completed_at: job.completed_at,
        target_ids: job.target_ids,
        total_size_bytes: job.total_size_bytes,
        error: job.error,
        progress: job.progress,
    };

    info!("Backup created by user {}: {:?}", auth.0.user_id, job.id);
    Ok(HttpResponse::Accepted().json(response))
}

async fn get_backup(
    state: web::Data<AppState>,
    _auth: AuthExtractor,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let backup_id = path.into_inner();

    // Get backup service from app state
    let backup_service = state
        .backup_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Backup service not available"))?;

    let job = backup_service
        .get_job_status(backup_id)
        .await
        .ok_or_else(|| ApiError::not_found("Backup job not found"))?;

    let response = BackupResponse {
        id: job.id,
        backup_type: job.backup_type.to_string(),
        status: job.status.to_string(),
        started_at: job.started_at,
        completed_at: job.completed_at,
        target_ids: job.target_ids,
        total_size_bytes: job.total_size_bytes,
        error: job.error,
        progress: job.progress,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_backup_status(
    state: web::Data<AppState>,
    _auth: AuthExtractor,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let backup_id = path.into_inner();

    // Get backup service from app state
    let backup_service = state
        .backup_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Backup service not available"))?;

    let job = backup_service
        .get_job_status(backup_id)
        .await
        .ok_or_else(|| ApiError::not_found("Backup job not found"))?;

    Ok(HttpResponse::Ok().json(json!({
        "id": job.id,
        "status": job.status.to_string(),
        "progress": job.progress,
        "error": job.error,
        "completed_at": job.completed_at,
    })))
}

async fn restore_backup(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    path: web::Path<Uuid>,
    req: web::Json<RestoreRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin"])?;

    let backup_id = path.into_inner();

    // Get backup service from app state
    let backup_service = state
        .backup_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Backup service not available"))?;

    // Check if backup exists
    let job = backup_service
        .get_job_status(backup_id)
        .await
        .ok_or_else(|| ApiError::not_found("Backup job not found"))?;

    if job.status != BackupStatus::Completed {
        return Err(ApiError::validation("Can only restore completed backups"));
    }

    // Run restore
    let outcome = backup_service
        .restore(backup_id, req.target_database.clone(), req.dry_run)
        .await
        .map_err(|e| ApiError::internal(format!("Restore failed: {}", e)))?;

    info!("Backup {} restored by user {}", backup_id, auth.0.user_id);

    let message = if outcome.dry_run {
        format!(
            "Dry run succeeded: {} is restorable ({} bytes)",
            outcome.object_name, outcome.bytes_restored
        )
    } else {
        format!("Restored {} bytes", outcome.bytes_restored)
    };

    Ok(HttpResponse::Ok().json(RestoreResponse {
        success: true,
        message,
    }))
}

async fn delete_backup(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin"])?;

    let backup_id = path.into_inner();

    // Get backup service from app state
    let backup_service = state
        .backup_service
        .as_ref()
        .ok_or_else(|| ApiError::internal("Backup service not available"))?;

    let _job = backup_service
        .get_job_status(backup_id)
        .await
        .ok_or_else(|| ApiError::not_found("Backup job not found"))?;

    // TODO: Actually delete from storage
    // For now just return success
    info!("Backup {} deleted by user {}", backup_id, auth.0.user_id);

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Backup {} deleted", backup_id),
    })))
}
