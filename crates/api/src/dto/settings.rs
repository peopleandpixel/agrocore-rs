//! Settings DTOs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisProviderAuthDto {
    /// API key for authentication (encrypted in DB).
    #[serde(default)]
    pub api_key: Option<String>,
    /// Username for basic auth (encrypted in DB).
    #[serde(default)]
    pub username: Option<String>,
    /// Password for basic auth (encrypted in DB).
    #[serde(default)]
    pub password: Option<String>,
    /// Bearer token for authentication (encrypted in DB).
    #[serde(default)]
    pub bearer_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisProviderConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub rate_limit_requests_per_second: u32,
    pub rate_limit_burst_size: u32,
    pub enabled: bool,
    /// The country this entry describes, and whether the tenant configured it or
    /// the value is the built-in default.
    ///
    /// Needed because the listing now returns the real upstream URLs instead of
    /// invented ones, so a client cannot tell a configured provider from a
    /// default by comparing domains.
    pub configured: Option<String>,
    /// Whether credentials are configured for this provider (read-only indicator).
    #[serde(default)]
    pub has_credentials: bool,
    /// Authentication credentials (only when saving, never returned in responses).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<LpisProviderAuthDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisProviderConfigList {
    pub providers: Vec<LpisProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisSettingsResponse {
    pub providers: std::collections::HashMap<String, LpisProviderConfig>,
    pub cache_backend: String,
    pub cache_default_ttl_seconds: u64,
    pub cache_max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateLpisSettingsRequest {
    pub providers: std::collections::HashMap<String, LpisProviderConfig>,
    pub cache_backend: Option<String>,
    pub cache_default_ttl_seconds: Option<u64>,
    pub cache_max_entries: Option<usize>,
}
