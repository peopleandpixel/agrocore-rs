use crate::AppState;
use crate::dto::{ErrorResponse, LpisProviderConfig, LpisProviderConfigList};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use agrocore_shared::lpis::LpisCountry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use utoipa::ToSchema;
use validator::Validate;

use agrocore_logging::{info, warn};
/// Find the config file by searching from current directory up to project root
fn find_config_file() -> PathBuf {
    let mut current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    loop {
        let config_path = current_dir.join("config/lpis-providers.toml");
        if config_path.exists() {
            return config_path;
        }

        // Check if we're at the filesystem root
        if !current_dir.pop() {
            break;
        }
    }

    // Fallback to default relative path
    PathBuf::from("config/lpis-providers.toml")
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .service(
                web::resource("/lpis")
                    .route(web::get().to(get_lpis_settings))
                    .route(web::put().to(update_lpis_settings)),
            )
            .service(web::resource("/lpis/providers").route(web::get().to(list_lpis_providers)))
            // Typed key/value settings (tasks.md F1/H1). Every one of these
            // persists to the database; before this the company profile lived in
            // the browser's localStorage, so settings were per-device.
            .service(
                web::resource("")
                    .route(web::get().to(list_settings))
                    .route(web::put().to(update_settings)),
            )
            .service(web::resource("/keys").route(web::get().to(list_setting_keys)))
            .service(
                web::resource("/{key}")
                    .route(web::get().to(get_setting))
                    .route(web::put().to(update_setting))
                    .route(web::delete().to(reset_setting)),
            )
            .service(web::resource("/restore-defaults").route(web::post().to(restore_defaults))),
    );
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LpisSettingsResponse {
    pub providers: HashMap<String, LpisProviderConfig>,
    pub cache_backend: String,
    pub cache_default_ttl_seconds: u64,
    pub cache_max_entries: usize,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateLpisSettingsRequest {
    pub providers: HashMap<String, LpisProviderConfig>,
    pub cache_backend: Option<String>,
    pub cache_default_ttl_seconds: Option<u64>,
    pub cache_max_entries: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/lpis",
    responses(
        (status = 200, description = "LPIS provider settings", body = LpisSettingsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn get_lpis_settings(
    _state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;

    // Load from config or return defaults - find config file from project root
    let config_path = find_config_file();
    let config_result =
        agrocore_lpis_providers::config::LpisProvidersConfig::load_from_path(&config_path);
    let config = config_result.unwrap_or_else(|e| {
        warn!(
            "Failed to load LPIS config from {:?}: {:?}, using defaults",
            config_path, e
        );
        Default::default()
    });

    let mut providers = HashMap::new();
    for (country, provider_config) in config.providers {
        providers.insert(
            country,
            LpisProviderConfig {
                base_url: provider_config.base_url,
                timeout_seconds: provider_config.timeout_seconds,
                cache_ttl_seconds: provider_config.cache_ttl_seconds,
                rate_limit_requests_per_second: provider_config.rate_limit.requests_per_second,
                rate_limit_burst_size: provider_config.rate_limit.burst_size,
                enabled: provider_config.enabled,
            },
        );
    }

    let response = LpisSettingsResponse {
        providers,
        cache_backend: format!("{:?}", config.cache.backend).to_lowercase(),
        cache_default_ttl_seconds: config.cache.default_ttl_seconds,
        cache_max_entries: config.cache.max_entries,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/lpis",
    request_body = UpdateLpisSettingsRequest,
    responses(
        (status = 200, description = "LPIS settings updated"),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn update_lpis_settings(
    _state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<UpdateLpisSettingsRequest>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;

    // Load current config
    let config_path = find_config_file();
    let mut config =
        agrocore_lpis_providers::config::LpisProvidersConfig::load_from_path(&config_path)
            .unwrap_or_default();

    // Update providers from request
    for (country_code, provider_config) in dto.0.providers {
        config.providers.insert(
            country_code,
            agrocore_lpis_providers::config::ProviderConfig {
                base_url: provider_config.base_url,
                auth: None,
                timeout_seconds: provider_config.timeout_seconds,
                rate_limit: agrocore_lpis_providers::config::RateLimitConfig {
                    requests_per_second: provider_config.rate_limit_requests_per_second,
                    burst_size: provider_config.rate_limit_burst_size,
                },
                cache_ttl_seconds: provider_config.cache_ttl_seconds,
                enabled: provider_config.enabled,
            },
        );
    }

    // Update cache config if provided
    if let Some(cache_backend) = dto.0.cache_backend {
        config.cache.backend = match cache_backend.to_lowercase().as_str() {
            "redis" => agrocore_lpis_providers::config::CacheBackend::Redis,
            _ => agrocore_lpis_providers::config::CacheBackend::Memory,
        };
    }
    if let Some(ttl) = dto.0.cache_default_ttl_seconds {
        config.cache.default_ttl_seconds = ttl;
    }
    if let Some(max_entries) = dto.0.cache_max_entries {
        config.cache.max_entries = max_entries;
    }

    // Save to file
    config
        .save_to_path(&config_path)
        .map_err(|e| SharedError::Internal(format!("Failed to save LPIS config: {}", e)))?;

    info!("LPIS settings updated by user: {}", auth.0.user_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "LPIS settings updated successfully",
        "restart_required": true
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/lpis/providers",
    responses(
        (status = 200, description = "List of available LPIS providers", body = LpisProviderConfigList),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn list_lpis_providers(
    _state: web::Data<AppState>,
    _auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    let mut providers = Vec::new();

    for country in [
        LpisCountry::Es,
        LpisCountry::Nl,
        LpisCountry::Fr,
        LpisCountry::Pt,
        LpisCountry::It,
        LpisCountry::De,
        LpisCountry::Pl,
        LpisCountry::At,
    ] {
        providers.push(LpisProviderConfig {
            base_url: format!(
                "https://{}.example.com/wfs",
                country.to_string().to_lowercase()
            ),
            timeout_seconds: 30,
            cache_ttl_seconds: 3600,
            rate_limit_requests_per_second: 10,
            rate_limit_burst_size: 20,
            enabled: true,
        });
    }

    Ok(HttpResponse::Ok().json(LpisProviderConfigList { providers }))
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
