//! Grouped settings endpoints (tasks.md F3).
//!
//! The typed key/value API from F1 works, but every caller had to know the key
//! names and the value types. This module adds one endpoint per resource, so a
//! client reads `GET /api/v1/settings/backup` instead of assembling six keys,
//! and cannot silently change a field that does not exist.
//!
//! Each group is backed by the same `system_settings` rows as the key/value API.
//! That is deliberate: there is one storage path, so a value written through a
//! group endpoint is visible through the key/value API and vice versa.
//!
//! What is stored here is configuration only. Credentials are not part of any
//! group: they belong in a secret store, and `is_sensitive` on a setting exists
//! so a value can be redacted from responses instead of being exposed by a
//! group endpoint that does not know what it is handling.

use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::AppState;
use crate::error::ApiError;
use crate::middleware::AuthExtractor;

use agrocore_domain::entities::setting::UpdateSetting;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_logging::{error, warn};

/// Register the grouped settings endpoints.
///
/// They live under the same `/settings` scope as the key/value API, so a client
/// already talking to `/api/v1/settings` finds them next to it.
///
/// Register the group resources.
///
/// Added to the existing `/settings` scope rather than opening a second one:
/// actix resolves two scopes with the same prefix to the first one only, so a
/// separate scope here would silently drop the key/value routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/lpis/providers").route(web::get().to(list_lpis_providers)))
        .service(web::resource("/groups").route(web::get().to(list_setting_groups)))
        .service(
            web::resource("/backup")
                .route(web::get().to(get_backup_group))
                .route(web::put().to(update_backup_group)),
        )
        .service(
            web::resource("/notification")
                .route(web::get().to(get_notification_group))
                .route(web::put().to(update_notification_group)),
        )
        .service(
            web::resource("/weather")
                .route(web::get().to(get_weather_group))
                .route(web::put().to(update_weather_group)),
        )
        .service(
            web::resource("/locale")
                .route(web::get().to(get_locale_group))
                .route(web::put().to(update_locale_group)),
        )
        .service(
            web::resource("/company")
                .route(web::get().to(get_company_group))
                .route(web::put().to(update_company_group)),
        );
}

/// Namespaces this module serves, with the fields each one owns.
///
/// A group declares its fields so the generic read/write below cannot drift from
/// the struct it fills: an unknown field in a request is rejected instead of
/// being written under a key nothing reads back.
struct GroupSpec {
    namespace: &'static str,
    /// (field name as in the DTO, settings key, declared type)
    fields: &'static [(&'static str, &'static str, FieldType)],
}

#[derive(Clone, Copy, PartialEq)]
enum FieldType {
    Str,
    Bool,
    Num,
    Arr,
}

impl FieldType {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(_) => Some(FieldType::Str),
            serde_json::Value::Bool(_) => Some(FieldType::Bool),
            serde_json::Value::Number(_) => Some(FieldType::Num),
            serde_json::Value::Array(_) => Some(FieldType::Arr),
            // An object under a scalar field is a client bug; null is handled
            // separately as a reset.
            serde_json::Value::Object(_) | serde_json::Value::Null => None,
        }
    }

    fn matches(self, value: &serde_json::Value) -> bool {
        FieldType::from_json(value) == Some(self)
    }
}

const BACKUP_GROUP: &[(&str, &str, FieldType)] = &[
    ("enabled", "backup.enabled", FieldType::Bool),
    ("schedule_db", "backup.schedule_db", FieldType::Str),
    ("schedule_config", "backup.schedule_config", FieldType::Str),
    ("timezone", "backup.timezone", FieldType::Str),
    ("retention_daily", "backup.retention.daily", FieldType::Num),
    (
        "retention_weekly",
        "backup.retention.weekly",
        FieldType::Num,
    ),
    (
        "retention_monthly",
        "backup.retention.monthly",
        FieldType::Num,
    ),
    (
        "retention_yearly",
        "backup.retention.yearly",
        FieldType::Num,
    ),
    (
        "verification_enabled",
        "backup.verification_enabled",
        FieldType::Bool,
    ),
];

const NOTIFICATION_GROUP: &[(&str, &str, FieldType)] = &[
    (
        "email_enabled",
        "notification.email.enabled",
        FieldType::Bool,
    ),
    ("push_enabled", "notification.push.enabled", FieldType::Bool),
];

const WEATHER_GROUP: &[(&str, &str, FieldType)] = &[
    ("provider", "weather.provider", FieldType::Str),
    (
        "cache_ttl_seconds",
        "weather.cache_ttl_seconds",
        FieldType::Num,
    ),
];

const LOCALE_GROUP: &[(&str, &str, FieldType)] = &[
    (
        "default_language",
        "locale.default_language",
        FieldType::Str,
    ),
    (
        "supported_languages",
        "locale.supported_languages",
        FieldType::Arr,
    ),
    ("timezone", "locale.timezone", FieldType::Str),
    ("date_format", "locale.date_format", FieldType::Str),
];

const COMPANY_GROUP: &[(&str, &str, FieldType)] = &[
    ("name", "company.name", FieldType::Str),
    ("tax_id", "company.tax_id", FieldType::Str),
    ("email", "company.email", FieldType::Str),
    ("phone", "company.phone", FieldType::Str),
    ("address", "company.address", FieldType::Str),
    ("website", "company.website", FieldType::Str),
    ("country", "company.country", FieldType::Str),
];

const ALL_GROUPS: &[GroupSpec] = &[
    GroupSpec {
        namespace: "backup",
        fields: BACKUP_GROUP,
    },
    GroupSpec {
        namespace: "notification",
        fields: NOTIFICATION_GROUP,
    },
    GroupSpec {
        namespace: "weather",
        fields: WEATHER_GROUP,
    },
    GroupSpec {
        namespace: "locale",
        fields: LOCALE_GROUP,
    },
    GroupSpec {
        namespace: "company",
        fields: COMPANY_GROUP,
    },
];

fn spec_for(namespace: &str) -> Option<&'static GroupSpec> {
    ALL_GROUPS.iter().find(|g| g.namespace == namespace)
}

/// Read one group into a JSON object.
///
/// Keys that were never written are omitted rather than filled with a made-up
/// default: the shipped defaults live in `system_settings`, so `list_effective`
/// already returns them, and a client cannot tell an invented default from a
/// configured one.
async fn read_group(
    repo: &dyn SettingsRepository,
    tid: TenantId,
    spec: &'static GroupSpec,
) -> Result<serde_json::Map<String, serde_json::Value>, ApiError> {
    let mut out = serde_json::Map::new();

    for (field, key, _) in spec.fields {
        match repo.get(tid, key).await {
            Ok(Some(entry)) => {
                // A sensitive setting is not returned through a group endpoint:
                // the group has no idea what the value means and cannot redact it
                // selectively.
                if entry.is_sensitive {
                    continue;
                }
                out.insert((*field).to_string(), entry.value);
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Could not read setting {key}: {e}");
            }
        }
    }

    Ok(out)
}

/// Write the present fields of a group.
///
/// `null` clears the tenant override, matching the key/value endpoint. A value
/// whose type does not match the declaration is rejected: storing a string under
/// a numeric field would be accepted by the database and then silently ignored by
/// every reader using `as_u64()`.
async fn write_group(
    repo: &dyn SettingsRepository,
    tid: TenantId,
    user_id: uuid::Uuid,
    spec: &'static GroupSpec,
    body: serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Map<String, serde_json::Value>, ApiError> {
    let mut reset_keys: Vec<String> = Vec::new();
    let mut updates: Vec<UpdateSetting> = Vec::new();

    for (field, value) in body {
        let Some((_, key, field_type)) = spec
            .fields
            .iter()
            .find(|(name, _, _)| *name == field.as_str())
        else {
            return Err(ApiError::validation(format!(
                "Unknown field '{field}' for settings group '{}'",
                spec.namespace
            )));
        };

        if value.is_null() {
            reset_keys.push((*key).to_string());
            continue;
        }

        if !field_type.matches(&value) {
            return Err(ApiError::validation(format!(
                "Field '{field}' expects {expected}, got {actual}",
                expected = type_name(*field_type),
                actual = value_type_name(&value),
            )));
        }

        updates.push(UpdateSetting {
            key: (*key).to_string(),
            value,
        });
    }

    for key in &reset_keys {
        repo.reset(tid, key).await.map_err(|e| {
            error!("Could not reset setting {key}: {e}");
            ApiError::internal("Settings could not be saved")
        })?;
    }
    if !updates.is_empty() {
        repo.set_many(tid, user_id, updates).await.map_err(|e| {
            error!("Could not save settings group {}: {e}", spec.namespace);
            ApiError::internal("Settings could not be saved")
        })?;
    }

    read_group(repo, tid, spec).await
}

fn type_name(t: FieldType) -> &'static str {
    match t {
        FieldType::Str => "a string",
        FieldType::Bool => "a boolean",
        FieldType::Num => "a number",
        FieldType::Arr => "an array",
    }
}

fn value_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::Array(_) => "an array",
        serde_json::Value::Object(_) => "an object",
        serde_json::Value::Null => "null",
    }
}

// ---------------------------------------------------------------------------
// Typed DTOs
//
// The typed structs exist so the schema documents the shape and so a client gets
// a compile-time error against a wrong field. The generic path above is what
// actually reads and writes; the handler converts between them.
// ---------------------------------------------------------------------------

macro_rules! group_dto {
    (
        $(#[$meta:meta])*
        $name:ident, $namespace:literal
        { $( $field:ident : $ty:ty ),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, Default)]
        pub struct $name {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field: Option<$ty>,
            )*
        }

        impl $name {
            /// The present fields, keyed as the DTO names them.
            fn to_body(&self) -> serde_json::Map<String, serde_json::Value> {
                let mut map = serde_json::Map::new();
                $(
                    if let Some(v) = &self.$field {
                        map.insert(
                            stringify!($field).to_string(),
                            serde_json::to_value(v).unwrap_or(serde_json::Value::Null),
                        );
                    }
                )*
                map
            }

            /// Rebuild the struct from a stored group.
            fn from_stored(
                stored: &serde_json::Map<String, serde_json::Value>,
            ) -> Self {
                // Deserializing the map directly works because the field names in
                // the DTO match the names used by `GroupSpec`; a field the map
                // does not carry stays `None`.
                serde_json::from_value(serde_json::Value::Object(stored.clone()))
                    .unwrap_or_default()
            }
        }
    };
}

group_dto!(
    /// Backup schedule, retention and verification.
    BackupGroupDto, "backup" {
    enabled: bool,
    schedule_db: String,
    schedule_config: String,
    timezone: String,
    retention_daily: u32,
    retention_weekly: u32,
    retention_monthly: u32,
    retention_yearly: u32,
    verification_enabled: bool,
});

group_dto!(
    /// Which notification channels are active.
    NotificationGroupDto, "notification" {
    email_enabled: bool,
    push_enabled: bool,
});

group_dto!(
    /// Weather data source and cache lifetime.
    WeatherGroupDto, "weather" {
    provider: String,
    cache_ttl_seconds: u64,
});

group_dto!(
    /// Language, timezone and date format.
    LocaleGroupDto, "locale" {
    default_language: String,
    supported_languages: Vec<String>,
    timezone: String,
    date_format: String,
});

group_dto!(
    /// Company profile printed on documents.
    ///
    /// This group is what the AdminUI previously kept in the browser's
    /// localStorage, which meant the profile was per device and lost on a tenant
    /// switch.
    CompanyGroupDto, "company" {
    name: String,
    tax_id: String,
    email: String,
    phone: String,
    address: String,
    website: String,
    country: String,
});

/// One settings group, for the group index.
#[derive(Serialize, ToSchema)]
pub struct SettingsGroupInfo {
    pub namespace: String,
    pub fields: Vec<SettingsGroupField>,
}

#[derive(Serialize, ToSchema)]
pub struct SettingsGroupField {
    pub name: String,
    pub key: String,
    pub value_type: String,
}

#[derive(Serialize, ToSchema)]
pub struct SettingsGroupsResponse {
    pub groups: Vec<SettingsGroupInfo>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/api/v1/settings/groups",
    responses(
        (status = 200, description = "Available settings groups and their fields", body = SettingsGroupsResponse),
        (status = 401, description = "Unauthorized", body = crate::dto::ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = crate::dto::ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn list_setting_groups(
    _state: web::Data<AppState>,
    auth: AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;

    let groups = ALL_GROUPS
        .iter()
        .map(|g| SettingsGroupInfo {
            namespace: g.namespace.to_string(),
            fields: g
                .fields
                .iter()
                .map(|(name, key, ty)| SettingsGroupField {
                    name: (*name).to_string(),
                    key: (*key).to_string(),
                    value_type: type_name(*ty).to_string(),
                })
                .collect(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(SettingsGroupsResponse { groups }))
}

/// GET/PUT for one group, described by its spec.
macro_rules! group_handlers {
    ($get_fn:ident, $put_fn:ident, $dto:ty, $namespace:literal) => {
        #[utoipa::path(
            get,
            path = concat!("/api/v1/settings/", $namespace),
            responses(
                (status = 200, description = concat!("Stored ", $namespace, " settings"), body = $dto),
                (status = 401, description = "Unauthorized", body = crate::dto::ErrorResponse),
                (status = 403, description = "Forbidden - Admin only", body = crate::dto::ErrorResponse)
            ),
            tag = "settings",
            security(("bearer_auth" = []))
        )]
        pub async fn $get_fn(
            state: web::Data<AppState>,
            auth: AuthExtractor,
        ) -> Result<HttpResponse, ApiError> {
            auth.require_admin()?;

            let spec = spec_for($namespace).ok_or_else(|| {
                ApiError::internal("Settings group is not registered")
            })?;
            let repo = state.db.settings_repo();
            let stored = read_group(&*repo, TenantId(auth.0.tenant_id), spec).await?;

            Ok(HttpResponse::Ok().json(<$dto>::from_stored(&stored)))
        }

        #[utoipa::path(
            put,
            path = concat!("/api/v1/settings/", $namespace),
            request_body = $dto,
            responses(
                (status = 200, description = concat!("Stored ", $namespace, " settings"), body = $dto),
                (status = 400, description = "Unknown field or wrong type", body = crate::dto::ErrorResponse),
                (status = 401, description = "Unauthorized", body = crate::dto::ErrorResponse),
                (status = 403, description = "Forbidden - Admin only", body = crate::dto::ErrorResponse)
            ),
            tag = "settings",
            security(("bearer_auth" = []))
        )]
        pub async fn $put_fn(
            state: web::Data<AppState>,
            auth: AuthExtractor,
            body: web::Json<$dto>,
        ) -> Result<HttpResponse, ApiError> {
            auth.require_admin()?;

            body.validate()
                .map_err(|e| ApiError::validation(e.to_string()))?;

            let spec = spec_for($namespace).ok_or_else(|| {
                ApiError::internal("Settings group is not registered")
            })?;
            let repo = state.db.settings_repo();
            let stored = write_group(
                &*repo,
                TenantId(auth.0.tenant_id),
                auth.0.user_id,
                spec,
                body.to_body(),
            )
            .await?;

            // The stored state, not an acknowledgement: a clamped or partially
            // rejected write has to be visible to the caller.
            Ok(HttpResponse::Ok().json(<$dto>::from_stored(&stored)))
        }
    };
}

group_handlers!(
    get_backup_group,
    update_backup_group,
    BackupGroupDto,
    "backup"
);
group_handlers!(
    get_notification_group,
    update_notification_group,
    NotificationGroupDto,
    "notification"
);
group_handlers!(
    get_weather_group,
    update_weather_group,
    WeatherGroupDto,
    "weather"
);
group_handlers!(
    get_locale_group,
    update_locale_group,
    LocaleGroupDto,
    "locale"
);
group_handlers!(
    get_company_group,
    update_company_group,
    CompanyGroupDto,
    "company"
);

/// The LPIS providers as configured, read from the database.
///
/// Replaces an endpoint that returned `https://{country}.example.com/wfs` for
/// every country: those were invented domains presented as configuration. A
/// country with no configured provider is reported as such rather than given a
/// plausible-looking URL.
#[utoipa::path(
    get,
    path = "/api/v1/settings/lpis/providers",
    responses(
        (status = 200, description = "Configured LPIS providers", body = crate::dto::LpisProviderConfigList),
        (status = 401, description = "Unauthorized", body = crate::dto::ErrorResponse),
        (status = 403, description = "Forbidden - Admin only", body = crate::dto::ErrorResponse)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn list_lpis_providers(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;

    let repo = state.db.settings_repo();
    let tid = TenantId(auth.0.tenant_id);

    let configured = crate::lpis_settings::load_provider_config(&*repo, tid).await;

    let providers = configured
        .into_iter()
        .map(|(country, cfg)| {
            use crate::dto::LpisProviderConfig;
            LpisProviderConfig {
                base_url: cfg.base_url,
                timeout_seconds: cfg.timeout_seconds,
                cache_ttl_seconds: cfg.cache_ttl_seconds,
                rate_limit_requests_per_second: cfg.rate_limit.requests_per_second,
                rate_limit_burst_size: cfg.rate_limit.burst_size,
                enabled: cfg.enabled,
                // Recorded so the UI can tell a configured provider from a
                // built-in default without comparing URLs.
                configured: Some(country),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(crate::dto::LpisProviderConfigList { providers }))
}

/// Placeholder to keep the module import list honest if the LPIS loader moves.
#[allow(dead_code)]
fn _uses_country() {
    let _ = agrocore_shared::lpis::LpisCountry::Es;
}
