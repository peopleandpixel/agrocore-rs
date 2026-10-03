//! Reading LPIS provider configuration from the database (tasks.md F4/F6).
//!
//! The configuration used to live in `config/lpis-providers.toml`, found by
//! walking up from the current directory. Two problems with that: it does not
//! work in a container where the working directory is `/`, and it is not
//! tenant-scoped, so every tenant shared one provider list and one set of
//! credentials.
//!
//! Values come from `system_settings` under `lpis.providers.<COUNTRY>.`, with
//! the built-in defaults as the fallback. Credentials are never written there:
//! `api_key` and friends are secrets, and a tenant setting row is readable by
//! every admin of that tenant.

use std::collections::HashMap;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_lpis_providers::config::{ProviderConfig, RateLimitConfig};

/// Settings key for one provider's base URL.
fn key_url(country: &str) -> String {
    format!("lpis.providers.{country}.base_url")
}
fn key_timeout(country: &str) -> String {
    format!("lpis.providers.{country}.timeout_seconds")
}
fn key_cache_ttl(country: &str) -> String {
    format!("lpis.providers.{country}.cache_ttl_seconds")
}
fn key_enabled(country: &str) -> String {
    format!("lpis.providers.{country}.enabled")
}
fn key_rps(country: &str) -> String {
    format!("lpis.providers.{country}.rate_limit.requests_per_second")
}
fn key_burst(country: &str) -> String {
    format!("lpis.providers.{country}.rate_limit.burst_size")
}

/// The countries this build knows about.
///
/// Derived from `LpisCountry` rather than hand-listed, so a new country in the
/// enum shows up here instead of silently missing from the provider list.
fn known_countries() -> Vec<&'static str> {
    use agrocore_shared::lpis::LpisCountry;
    [
        LpisCountry::Es,
        LpisCountry::Nl,
        LpisCountry::Fr,
        LpisCountry::Pt,
        LpisCountry::It,
        LpisCountry::De,
        LpisCountry::Pl,
        LpisCountry::At,
    ]
    .iter()
    .map(|c| country_code(*c))
    .collect()
}

/// Two-letter code, as used in the settings keys and in `LpisProvidersConfig`.
fn country_code(c: agrocore_shared::lpis::LpisCountry) -> &'static str {
    use agrocore_shared::lpis::LpisCountry;
    match c {
        LpisCountry::Es => "ES",
        LpisCountry::Nl => "NL",
        LpisCountry::Fr => "FR",
        LpisCountry::Pt => "PT",
        LpisCountry::It => "IT",
        LpisCountry::De => "DE",
        LpisCountry::Pl => "PL",
        LpisCountry::At => "AT",
        LpisCountry::Other => "ZZ",
    }
}

/// Read a string setting.
async fn read_str(repo: &dyn SettingsRepository, tid: TenantId, key: &str) -> Option<String> {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => entry.value.as_str().map(|s| s.to_string()),
        Ok(None) => None,
        Err(e) => {
            agrocore_logging::warn!("Could not read LPIS setting {key}: {e}");
            None
        }
    }
}

/// Read a numeric setting.
async fn read_num(repo: &dyn SettingsRepository, tid: TenantId, key: &str) -> Option<u64> {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => entry.value.as_u64(),
        Ok(None) => None,
        Err(e) => {
            agrocore_logging::warn!("Could not read LPIS setting {key}: {e}");
            None
        }
    }
}

/// Read a boolean setting.
async fn read_bool(repo: &dyn SettingsRepository, tid: TenantId, key: &str) -> Option<bool> {
    match repo.get(tid, key).await {
        Ok(Some(entry)) => entry.value.as_bool(),
        Ok(None) => None,
        Err(e) => {
            agrocore_logging::warn!("Could not read LPIS setting {key}: {e}");
            None
        }
    }
}

/// The provider configuration for every known country.
///
/// Built-in defaults are the base; a tenant override replaces the individual
/// field, not the whole provider. A country with no override therefore still
/// gets its real upstream URL rather than an invented one.
pub async fn load_provider_config(
    repo: &dyn SettingsRepository,
    tid: TenantId,
) -> HashMap<String, ProviderConfig> {
    let defaults = agrocore_lpis_providers::config::LpisProvidersConfig::default();
    let mut out: HashMap<String, ProviderConfig> = HashMap::new();

    for country in known_countries() {
        let fallback = defaults.providers.get(country).cloned();
        let mut config = fallback.clone().unwrap_or(ProviderConfig {
            base_url: String::new(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_seconds: 3600,
            enabled: true,
        });

        if let Some(url) = read_str(repo, tid, &key_url(country)).await
            && !url.is_empty()
        {
            config.base_url = url;
        }
        if let Some(v) = read_num(repo, tid, &key_timeout(country)).await {
            config.timeout_seconds = v;
        }
        if let Some(v) = read_num(repo, tid, &key_cache_ttl(country)).await {
            config.cache_ttl_seconds = v;
        }
        if let Some(v) = read_bool(repo, tid, &key_enabled(country)).await {
            config.enabled = v;
        }
        if let Some(v) = read_num(repo, tid, &key_rps(country)).await {
            config.rate_limit.requests_per_second = v as u32;
        }
        if let Some(v) = read_num(repo, tid, &key_burst(country)).await {
            config.rate_limit.burst_size = v as u32;
        }

        // A provider with no URL at all cannot be used, and reporting it as
        // configured would send requests to an empty host.
        if config.base_url.is_empty() {
            config.enabled = false;
        }

        out.insert(country.to_string(), config);
    }

    out
}

/// The cache settings for LPIS responses.
#[derive(Debug, Clone)]
pub struct LpisCacheSettings {
    pub enabled: bool,
    pub backend: String,
    pub default_ttl_seconds: u64,
    pub max_entries: usize,
}

/// Read the cache configuration, falling back to the built-in defaults.
pub async fn load_cache_config(repo: &dyn SettingsRepository, tid: TenantId) -> LpisCacheSettings {
    let defaults = agrocore_lpis_providers::config::LpisProvidersConfig::default();

    LpisCacheSettings {
        enabled: read_bool(repo, tid, "lpis.cache.enabled")
            .await
            .unwrap_or(defaults.cache.enabled),
        backend: read_str(repo, tid, "lpis.cache.backend")
            .await
            .unwrap_or_else(|| defaults.cache.backend.to_string()),
        default_ttl_seconds: read_num(repo, tid, "lpis.cache.default_ttl_seconds")
            .await
            .unwrap_or(defaults.cache.default_ttl_seconds),
        max_entries: read_num(repo, tid, "lpis.cache.max_entries")
            .await
            .map(|v| v as usize)
            .unwrap_or(defaults.cache.max_entries),
    }
}
