//! Settings DTOs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisProviderConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub rate_limit_requests_per_second: u32,
    pub rate_limit_burst_size: u32,
    pub enabled: bool,
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
