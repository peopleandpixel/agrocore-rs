//! Typed key/value settings (tasks.md F1).
//!
//! The schema had no settings table, so the AdminUI kept the company profile in
//! the browser's localStorage. That made every setting per-browser: a second
//! device saw the defaults, and the backend never learned what was configured.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

use super::tenant::TenantId;

/// One setting row.
///
/// `value` is a JSONB string in the database and stays one here; parsing it into
/// a typed variant is the repository's job, so a malformed value surfaces there
/// with the key that caused it rather than as an opaque decode error in a
/// handler.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemSetting {
    pub id: Uuid,
    /// `None` is a system-wide default that every tenant inherits.
    pub tenant_id: Option<TenantId>,
    pub key: String,
    /// The raw JSON text as stored.
    pub value: String,
    pub value_type: SettingValueType,
    pub description: Option<String>,
    /// Sensitive values are never returned to non-admin callers.
    pub is_sensitive: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// Drives validation and which input the AdminUI renders.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SettingValueType {
    #[default]
    String,
    Number,
    Boolean,
    Json,
    Array,
}

impl SettingValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Json => "json",
            Self::Array => "array",
        }
    }
}

impl std::str::FromStr for SettingValueType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(Self::String),
            "number" => Ok(Self::Number),
            "boolean" => Ok(Self::Boolean),
            "json" => Ok(Self::Json),
            "array" => Ok(Self::Array),
            other => Err(format!("unknown setting value_type: {other}")),
        }
    }
}

impl std::fmt::Display for SettingValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A setting with its value already parsed.
///
/// This is what handlers and the UI see: a typed value plus the metadata needed
/// to render an input and to decide whether the field may be displayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    /// `None` means the value came from the system-wide default.
    pub is_default: bool,
    pub value_type: SettingValueType,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

/// A setting plus, where the tenant overrides it, the inherited default.
///
/// The UI needs both: showing the effective value while making it obvious which
/// fields are inherited and can therefore be reset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingWithDefault {
    pub entry: SettingEntry,
    /// The system default, absent when the tenant already overrides it or when
    /// no default exists.
    pub default_value: Option<serde_json::Value>,
}

/// A key with a value to write, or `None` to delete the tenant's override.
#[derive(Debug, Clone)]
pub struct UpdateSetting {
    pub key: String,
    pub value: serde_json::Value,
}

/// Known keys.
///
/// A string constant rather than an enum: settings are added without a code
/// change, and the table is the source of truth for what exists. This is a
/// convenience for the keys the application itself reads.
pub mod keys {
    pub const COMPANY_NAME: &str = "company.name";
    pub const COMPANY_TAX_ID: &str = "company.tax_id";
    pub const COMPANY_EMAIL: &str = "company.email";
    pub const COMPANY_PHONE: &str = "company.phone";
    pub const COMPANY_ADDRESS: &str = "company.address";
    pub const COMPANY_WEBSITE: &str = "company.website";
    pub const COMPANY_COUNTRY: &str = "company.country";

    pub const DEFAULT_LANGUAGE: &str = "locale.default_language";
    pub const SUPPORTED_LANGUAGES: &str = "locale.supported_languages";
    pub const TIMEZONE: &str = "locale.timezone";
    pub const DATE_FORMAT: &str = "locale.date_format";

    pub const BACKUP_ENABLED: &str = "backup.enabled";
    pub const BACKUP_SCHEDULE: &str = "backup.schedule";
    pub const BACKUP_RETENTION_DAYS: &str = "backup.retention_days";
    pub const BACKUP_TARGETS: &str = "backup.targets";
    pub const BACKUP_VERIFY: &str = "backup.verify";

    pub const NOTIFICATION_EMAIL_ENABLED: &str = "notification.email.enabled";
    pub const NOTIFICATION_PUSH_ENABLED: &str = "notification.push.enabled";

    pub const WEATHER_PROVIDER: &str = "weather.provider";
    pub const WEATHER_CACHE_TTL: &str = "weather.cache_ttl_seconds";
}

/// Settings grouped for the UI, so the page does not have to know the key layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSections {
    pub company: HashMap<String, SettingEntry>,
    pub locale: HashMap<String, SettingEntry>,
    pub backup: HashMap<String, SettingEntry>,
    pub notification: HashMap<String, SettingEntry>,
    pub weather: HashMap<String, SettingEntry>,
}

impl SettingsSections {
    pub fn from_entries(entries: &[SettingWithDefault]) -> Self {
        let mut out = Self {
            company: HashMap::new(),
            locale: HashMap::new(),
            backup: HashMap::new(),
            notification: HashMap::new(),
            weather: HashMap::new(),
        };

        for item in entries {
            let target = match item.entry.key.split_once('.').map(|(ns, _)| ns) {
                Some("company") => &mut out.company,
                Some("locale") => &mut out.locale,
                Some("backup") => &mut out.backup,
                Some("notification") => &mut out.notification,
                Some("weather") => &mut out.weather,
                // An unknown namespace is not an error: settings are
                // extensible, and hiding them would make the feature useless.
                _ => continue,
            };
            target.insert(item.entry.key.clone(), item.entry.clone());
        }

        out
    }
}
