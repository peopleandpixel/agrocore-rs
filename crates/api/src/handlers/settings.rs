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
            .service(web::resource("/lpis/providers").route(web::get().to(list_lpis_providers))),
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
    eprintln!("DEBUG HANDLER: get_lpis_settings called!");
    auth.require_admin()?;

    // Load from config or return defaults - find config file from project root
    let config_path = find_config_file();
    eprintln!(
        "DEBUG HANDLER: Config path: {:?}, exists: {}",
        config_path,
        config_path.exists()
    );
    let config_result =
        agrocore_lpis_providers::config::LpisProvidersConfig::load_from_path(&config_path);
    eprintln!(
        "DEBUG HANDLER: Config load result: {:?}",
        config_result.is_ok()
    );
    if let Err(ref e) = config_result {
        eprintln!("DEBUG HANDLER: Config error: {:?}", e);
    }
    let config = config_result.unwrap_or_else(|e| {
        tracing::warn!(
            "Failed to load LPIS config from {:?}: {:?}, using defaults",
            config_path,
            e
        );
        Default::default()
    });

    eprintln!(
        "DEBUG HANDLER: Config loaded, providers count: {}",
        config.providers.len()
    );

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

    tracing::info!("LPIS settings updated by user: {}", auth.0.user_id);

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
    eprintln!("DEBUG HANDLER: list_lpis_providers called - START");

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
