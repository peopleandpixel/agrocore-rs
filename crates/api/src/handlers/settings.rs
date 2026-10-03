use crate::AppState;

use crate::dto::ErrorResponse;
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            // Groups first: `/{key}` matches any single segment, so the group
            // resources have to be registered before it or `/backup` is read as
            // the setting "backup". actix matches in registration order.
            .configure(crate::handlers::settings_groups::configure)
            // Literal paths are registered before `/{key}` on purpose: the key
            // resource matches any single segment, so `/lpis/providers` would be
            // swallowed by `/{key}` = "lpis" if it came second. actix matches in
            // registration order.
            .service(web::resource("/restore-defaults").route(web::post().to(restore_defaults)))
            // Typed key/value settings (tasks.md F1/H1). Every one of these
            // persists to the database; before this the company profile lived in
            // the browser's localStorage, so settings were per-device.
            .service(
                web::resource("")
                    .route(web::get().to(list_settings))
                    .route(web::put().to(update_settings)),
            )
            .service(web::resource("/keys").route(web::get().to(list_setting_keys)))
            // The key resource has to come last for the reason above.
            .service(
                web::resource("/{key}")
                    .route(web::get().to(get_setting))
                    .route(web::put().to(update_setting))
                    .route(web::delete().to(reset_setting)),
            ),
    );
}

// ---------------------------------------------------------------------------
// Typed settings (tasks.md F1/H1)
// ---------------------------------------------------------------------------

/// The value as stored, plus whether it is inherited.
///
/// `is_default` is what lets the UI show a field as inherited and offer a
/// reset, instead of silently displaying a value the tenant never chose.
#[derive(Serialize, ToSchema)]
pub struct SettingDto {
    pub key: String,
    pub value: serde_json::Value,
    pub value_type: agrocore_domain::entities::setting::SettingValueType,
    pub is_default: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<uuid::Uuid>,
}

impl From<agrocore_domain::entities::setting::SettingWithDefault> for SettingDto {
    fn from(w: agrocore_domain::entities::setting::SettingWithDefault) -> Self {
        Self {
            key: w.entry.key.clone(),
            value: w.entry.value.clone(),
            value_type: w.entry.value_type,
            is_default: w.entry.is_default,
            default_value: w.default_value.clone(),
            description: w.entry.description.clone(),
            is_sensitive: w.entry.is_sensitive,
            updated_at: w.entry.updated_at,
            updated_by: w.entry.updated_by,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct SettingsListResponse {
    pub settings: Vec<SettingDto>,
}

/// A key/value pair to write. A `null` value removes the tenant's override,
/// which is what a reset from the UI sends.
#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateSettingRequest {
    pub value: serde_json::Value,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct SettingKeyDto {
    pub key: String,
    pub value_type: agrocore_domain::entities::setting::SettingValueType,
    pub description: Option<String>,
    pub is_sensitive: bool,
}

#[derive(Serialize, ToSchema)]
pub struct RestoreDefaultsResponse {
    pub restored_keys: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/settings",
    responses(
        (status = 200, description = "Effective settings for the tenant", body = SettingsListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn list_settings(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let rows = state
        .db
        .settings_repo()
        .list_effective(agrocore_domain::entities::tenant::TenantId(
            auth.0.tenant_id,
        ))
        .await?;

    Ok(HttpResponse::Ok().json(SettingsListResponse {
        settings: rows.into_iter().map(SettingDto::from).collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/keys",
    responses(
        (status = 200, description = "Known setting keys", body = Vec<SettingKeyDto>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn list_setting_keys(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let keys = state.db.settings_repo().list_keys().await?;
    Ok(HttpResponse::Ok().json(
        keys.into_iter()
            .map(
                |(key, value_type, description, is_sensitive)| SettingKeyDto {
                    key,
                    value_type,
                    description,
                    is_sensitive,
                },
            )
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/{key}",
    responses(
        (status = 200, description = "One effective setting", body = SettingDto),
        (status = 404, description = "Key unknown", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn get_setting(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let entry = state
        .db
        .settings_repo()
        .get(
            agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id),
            &path,
        )
        .await?
        .ok_or_else(|| ApiError::not_found(format!("Unknown setting: {}", path.into_inner())))?;

    Ok(HttpResponse::Ok().json(SettingDto {
        key: entry.key,
        value: entry.value,
        value_type: entry.value_type,
        is_default: entry.is_default,
        default_value: None,
        description: entry.description,
        is_sensitive: entry.is_sensitive,
        updated_at: entry.updated_at,
        updated_by: entry.updated_by,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings written", body = SettingsListResponse),
        (status = 400, description = "Value does not match its declared type", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn update_settings(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<UpdateSettingsRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;

    let repo = state.db.settings_repo();
    let tid = agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id);
    let user_id = auth.0.user_id;
    // A `null` value removes the override rather than storing JSON null,
    // which would look like a real setting that simply has no value.
    let mut reset_keys: Vec<String> = Vec::new();
    let mut updates: Vec<agrocore_domain::entities::setting::UpdateSetting> = Vec::new();

    for (key, value) in dto.0.settings {
        if value.is_null() {
            reset_keys.push(key);
        } else {
            updates.push(agrocore_domain::entities::setting::UpdateSetting { key, value });
        }
    }

    for key in &reset_keys {
        repo.reset(tid, key).await?;
    }
    if !updates.is_empty() {
        repo.set_many(tid, user_id, updates).await?;
    }

    let rows = repo.list_effective(tid).await?;
    Ok(HttpResponse::Ok().json(SettingsListResponse {
        settings: rows.into_iter().map(SettingDto::from).collect(),
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/{key}",
    request_body = UpdateSettingRequest,
    responses(
        (status = 200, description = "Setting written", body = SettingsListResponse),
        (status = 400, description = "Value does not match its declared type", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn update_setting(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
    dto: web::Json<UpdateSettingRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let key = path.into_inner();

    if dto.0.value.is_null() {
        state
            .db
            .settings_repo()
            .reset(
                agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id),
                &key,
            )
            .await?;
    } else {
        state
            .db
            .settings_repo()
            .set(
                agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id),
                auth.0.user_id,
                agrocore_domain::entities::setting::UpdateSetting {
                    key: key.clone(),
                    value: dto.0.value.clone(),
                },
            )
            .await?;
    }

    let rows = state
        .db
        .settings_repo()
        .list_effective(agrocore_domain::entities::tenant::TenantId(
            auth.0.tenant_id,
        ))
        .await?;
    Ok(HttpResponse::Ok().json(SettingsListResponse {
        settings: rows.into_iter().map(SettingDto::from).collect(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/settings/{key}",
    responses(
        (status = 200, description = "Override removed", body = RestoreDefaultsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn reset_setting(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let key = path.into_inner();
    let removed = state
        .db
        .settings_repo()
        .reset(
            agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id),
            &key,
        )
        .await?;

    Ok(HttpResponse::Ok().json(RestoreDefaultsResponse {
        restored_keys: if removed { vec![key] } else { Vec::new() },
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/restore-defaults",
    responses(
        (status = 200, description = "All tenant overrides removed", body = RestoreDefaultsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn restore_defaults(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    // Deletes every tenant override in the system, so this is admin-only by
    // design and not merely by role check.
    let keys = state.db.settings_repo().restore_defaults().await?;
    Ok(HttpResponse::Ok().json(RestoreDefaultsResponse {
        restored_keys: keys,
    }))
}
