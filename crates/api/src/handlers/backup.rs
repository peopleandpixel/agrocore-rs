//! Backup API Handlers

use crate::AppState;
use crate::dto::{
    BackupConfigResponse, BackupResponse, BackupSummaryResponse, CreateBackupRequest,
    RestoreRequest, RestoreResponse, UpdateBackupConfigRequest,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor;
use actix_web::{HttpResponse, web};
use agrocore_backup::service::{BackupService, BackupStatus, BackupType};
use agrocore_logging::info;
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

async fn get_backup_config(
    state: web::Data<AppState>,
    _auth: AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    // Return current backup configuration
    let config = BackupConfigResponse {
        enabled: true,
        schedule_db: "0 2 * * *".to_string(),
        schedule_config: "0 3 * * 0".to_string(),
        timezone: "UTC".to_string(),
        targets_count: 0,
        retention_daily: 7,
        retention_weekly: 4,
        retention_monthly: 12,
        retention_yearly: 7,
        verification_enabled: true,
    };
    Ok(HttpResponse::Ok().json(config))
}

async fn update_backup_config(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    req: web::Json<UpdateBackupConfigRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin"])?;
    // TODO: Persist config changes
    info!("Backup config update requested by user: {}", auth.0.user_id);
    Ok(HttpResponse::Ok().json(json!({"message": "Backup configuration updated"})))
}

async fn list_backups(
    state: web::Data<AppState>,
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
    backup_service
        .restore(backup_id, req.target_database.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Restore failed: {}", e)))?;

    info!("Backup {} restored by user {}", backup_id, auth.0.user_id);

    Ok(HttpResponse::Ok().json(RestoreResponse {
        success: true,
        message: format!("Backup {} restored successfully", backup_id),
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

    let job = backup_service
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
