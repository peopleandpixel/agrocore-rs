//! Configuration for LPIS Providers

use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LpisProvidersConfig {
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub retry: RetryConfig,
}

impl Default for LpisProvidersConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // Default BRP (Netherlands) configuration
        providers.insert(
            "NL".to_string(),
            ProviderConfig {
                base_url: "https://geodata.nationaalgeoregister.nl/brppercelen/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default SIGPAC (Spain) configuration
        providers.insert(
            "ES".to_string(),
            ProviderConfig {
                base_url: "https://sigpac.mapa.gob.es/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default RPG (France) configuration
        providers.insert(
            "FR".to_string(),
            ProviderConfig {
                base_url: "https://geoservices.ign.fr/rpg/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default iLPIS (Portugal) configuration
        providers.insert(
            "PT".to_string(),
            ProviderConfig {
                base_url: "https://ide.ifap.pt/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default SIAN (Italy) configuration
        providers.insert(
            "IT".to_string(),
            ProviderConfig {
                base_url: "https://www.sian.it/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default LPIS-DE (Germany) configuration
        providers.insert(
            "DE".to_string(),
            ProviderConfig {
                base_url: "https://geodienste.bfn.de/lpis/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default LPIS-PL (Poland) configuration
        providers.insert(
            "PL".to_string(),
            ProviderConfig {
                base_url: "https://geoportal.arrim.gov.pl/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        // Default INVEKOS (Austria) configuration
        providers.insert(
            "AT".to_string(),
            ProviderConfig {
                base_url: "https://data.gv.at/wfs".to_string(),
                auth: None,
                timeout_seconds: 30,
                rate_limit: RateLimitConfig::default(),
                cache_ttl_seconds: 3600,
                enabled: true,
            },
        );

        Self {
            providers,
            cache: CacheConfig::default(),
            rate_limit: RateLimitConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl LpisProvidersConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::builder();

        // Load from file if exists
        config =
            config.add_source(File::new("config/lpis-providers", FileFormat::Toml).required(false));

        // Load from environment variables
        config = config.add_source(config::Environment::with_prefix("LPIS").separator("__"));

        config.build()?.try_deserialize()
    }

    /// Save the configuration to the TOML file
    pub fn save(&self) -> Result<(), ConfigError> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Message(format!("Failed to serialize config: {}", e)))?;

        // Ensure config directory exists
        if let Some(parent) = std::path::Path::new("config/lpis-providers").parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::Message(format!("Failed to create config directory: {}", e))
            })?;
        }

        std::fs::write("config/lpis-providers.toml", toml_string)
            .map_err(|e| ConfigError::Message(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    pub fn get_provider_config(&self, country: &str) -> Option<&ProviderConfig> {
        self.providers.get(country).filter(|c| c.enabled)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub auth: Option<AuthConfig>,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    pub cache_ttl_seconds: u64,
    pub enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_seconds: 3600,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub backend: CacheBackend,
    pub redis_url: Option<String>,
    pub default_ttl_seconds: u64,
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: CacheBackend::Memory,
            redis_url: None,
            default_ttl_seconds: 3600,
            max_entries: 10000,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    Memory,
    Redis,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}
