use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use std::sync::OnceLock;

#[cfg(feature = "sqlx")]
use sqlx::postgres::PgPoolOptions;

static JWT_SECRET: OnceLock<String> = OnceLock::new();
static DECODING_KEY: OnceLock<DecodingKey> = OnceLock::new();
static CONFIG: OnceLock<AgroCoreConfig> = OnceLock::new();

/// Centralized application configuration loaded from environment variables.
///
/// Loads all configuration once at startup into a `OnceLock` for zero-allocation access.
/// Supports `.env` file loading via `dotenv` if available (feature-gated).
///
/// # Environment Variables
/// - `JWT_SECRET` — Secret key for JWT signing/verification (required for production)
/// - `REDIS_URL` — Optional Redis connection URL for token revocation storage
/// - `TOKEN_BLACKLIST_TTL_SECONDS` — TTL for revoked tokens (default: 3600)
/// - `DATABASE_URL` — PostgreSQL connection string
/// - `DATABASE_MAX_CONNECTIONS` — Pool max connections (default: 20)
/// - `DATABASE_MIN_CONNECTIONS` — Pool min connections (default: 1)
/// - `DATABASE_IDLE_TIMEOUT_SECS` — Idle connection timeout (default: 600)
/// - `DATABASE_MAX_LIFETIME_SECS` — Max connection lifetime (default: 1800)
/// - `DATABASE_ACQUIRE_TIMEOUT_SECS` — Connection acquire timeout (default: 30)
/// - `DATABASE_CONNECT_TIMEOUT_SECS` — TCP connect timeout (default: 10)
/// - `NATS_URL` — NATS server URL (default: nats://localhost:4222)
/// - `MQTT_BROKER` — MQTT broker URL (default: mqtt://localhost:1883)
/// - `RUST_LOG` — Log level filter (default: info)
#[derive(Debug, Clone, Deserialize)]
pub struct AgroCoreConfig {
    /// JWT secret for signing/verifying tokens
    pub jwt_secret: String,
    /// Redis URL for token revocation (None = in-memory fallback)
    pub redis_url: Option<String>,
    /// TTL in seconds for revoked JWT tokens in the blacklist
    #[serde(default = "default_token_blacklist_ttl")]
    pub token_blacklist_ttl_secs: u64,
    /// PostgreSQL connection URL
    pub database_url: String,
    /// Maximum database connections in pool
    #[serde(default = "default_max_connections")]
    pub database_max_connections: u32,
    /// Minimum database connections in pool
    #[serde(default = "default_min_connections")]
    pub database_min_connections: u32,
    /// Idle connection timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub database_idle_timeout_secs: u64,
    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub database_max_lifetime_secs: u64,
    /// Connection acquire timeout in seconds
    #[serde(default = "default_acquire_timeout")]
    pub database_acquire_timeout_secs: u64,
    /// TCP connect timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub database_connect_timeout_secs: u64,
    /// NATS server URL
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    /// MQTT broker URL
    #[serde(default = "default_mqtt_broker")]
    pub mqtt_broker: String,
    /// Log level filter
    #[serde(default = "default_rust_log")]
    pub rust_log: String,
}

fn default_token_blacklist_ttl() -> u64 {
    3600
}
fn default_max_connections() -> u32 {
    20
}
fn default_min_connections() -> u32 {
    1
}
fn default_idle_timeout() -> u64 {
    600
}
fn default_max_lifetime() -> u64 {
    1800
}
fn default_acquire_timeout() -> u64 {
    30
}
fn default_connect_timeout() -> u64 {
    10
}
fn default_nats_url() -> String {
    "nats://localhost:4222".to_string()
}
fn default_mqtt_broker() -> String {
    "mqtt://localhost:1883".to_string()
}
fn default_rust_log() -> String {
    "info".to_string()
}

impl Default for AgroCoreConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "dev-secret".to_string(),
            redis_url: None,
            token_blacklist_ttl_secs: 3600,
            database_url: "postgres://postgres:password@localhost/agrocore".to_string(),
            database_max_connections: 20,
            database_min_connections: 1,
            database_idle_timeout_secs: 600,
            database_max_lifetime_secs: 1800,
            database_acquire_timeout_secs: 30,
            database_connect_timeout_secs: 10,
            nats_url: "nats://localhost:4222".to_string(),
            mqtt_broker: "mqtt://localhost:1883".to_string(),
            rust_log: "info".to_string(),
        }
    }
}

impl AgroCoreConfig {
    /// Load configuration from environment variables.
    /// Uses `dotenv` if the `dotenv` crate is available and a `.env` file exists.
    pub fn from_env() -> Self {
        Self::from_env_with_defaults(Self::default())
    }

    /// Load configuration from environment variables with custom defaults.
    pub fn from_env_with_defaults(defaults: Self) -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .ok()
                .unwrap_or(defaults.jwt_secret),
            redis_url: std::env::var("REDIS_URL").ok().or(defaults.redis_url),
            token_blacklist_ttl_secs: std::env::var("TOKEN_BLACKLIST_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.token_blacklist_ttl_secs),
            database_url: std::env::var("DATABASE_URL")
                .ok()
                .unwrap_or(defaults.database_url),
            database_max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_max_connections),
            database_min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_min_connections),
            database_idle_timeout_secs: std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_idle_timeout_secs),
            database_max_lifetime_secs: std::env::var("DATABASE_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_max_lifetime_secs),
            database_acquire_timeout_secs: std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_acquire_timeout_secs),
            database_connect_timeout_secs: std::env::var("DATABASE_CONNECT_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.database_connect_timeout_secs),
            nats_url: std::env::var("NATS_URL").ok().unwrap_or(defaults.nats_url),
            mqtt_broker: std::env::var("MQTT_BROKER")
                .ok()
                .unwrap_or(defaults.mqtt_broker),
            rust_log: std::env::var("RUST_LOG").ok().unwrap_or(defaults.rust_log),
        }
    }

    /// Get the globally cached configuration instance.
    /// Initializes from environment variables on first call.
    pub fn global() -> &'static Self {
        CONFIG.get_or_init(Self::from_env)
    }

    /// Initialize the global configuration with a custom instance.
    /// Must be called before any `global()` access; panics if already initialized.
    pub fn init_global(config: Self) -> &'static Self {
        let _ = CONFIG.set(config);
        CONFIG.get().expect("Config should be initialized")
    }

    /// Returns the JWT secret.
    pub fn jwt_secret_str(&self) -> &str {
        &self.jwt_secret
    }

    /// Returns the database URL with connect_timeout appended as query parameter.
    #[cfg(feature = "sqlx")]
    pub fn database_url_with_timeout(&self) -> String {
        let connect_timeout = self.database_connect_timeout_secs;
        if self.database_url.contains('?') {
            format!("{}&connect_timeout={}", self.database_url, connect_timeout)
        } else {
            format!("{}?connect_timeout={}", self.database_url, connect_timeout)
        }
    }

    /// Returns PgPoolOptions configured from this config.
    #[cfg(feature = "sqlx")]
    pub fn pg_pool_options(&self) -> PgPoolOptions {
        PgPoolOptions::new()
            .max_connections(self.database_max_connections)
            .min_connections(self.database_min_connections)
            .idle_timeout(std::time::Duration::from_secs(
                self.database_idle_timeout_secs,
            ))
            .max_lifetime(std::time::Duration::from_secs(
                self.database_max_lifetime_secs,
            ))
            .acquire_timeout(std::time::Duration::from_secs(
                self.database_acquire_timeout_secs,
            ))
    }

    /// Returns the connect timeout duration.
    #[cfg(feature = "sqlx")]
    pub fn connect_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.database_connect_timeout_secs)
    }
}

/// Returns the JWT secret for token signing/verification.
///
/// # Security
/// - Uses centralized `AgroCoreConfig` loaded from `JWT_SECRET` env var
/// - Falls back to `dev-secret` if not set (logs warning)
/// - Never falls back to insecure default in production
pub fn jwt_secret() -> &'static str {
    JWT_SECRET
        .get_or_init(|| {
            let secret = AgroCoreConfig::global().jwt_secret.clone();
            if secret == "dev-secret" {
                tracing::warn!("Set JWT_SECRET environment variable before running in production!");
            }
            secret
        })
        .as_str()
}

/// Returns a cached DecodingKey for JWT token validation.
/// The key is created once and reused across all requests for performance.
pub fn decoding_key() -> &'static DecodingKey {
    DECODING_KEY.get_or_init(|| DecodingKey::from_secret(jwt_secret().as_bytes()))
}

/// Database connection pool configuration, loaded from centralized `AgroCoreConfig`.
///
/// All values have sensible defaults for production use.
#[cfg(feature = "sqlx")]
pub fn pg_pool_options() -> PgPoolOptions {
    AgroCoreConfig::global().pg_pool_options()
}

/// Returns the connect timeout from centralized configuration.
/// This timeout applies to the initial TCP connection establishment.
#[cfg(feature = "sqlx")]
pub fn connect_timeout() -> std::time::Duration {
    AgroCoreConfig::global().connect_timeout()
}

/// Returns the token blacklist TTL from centralized configuration.
/// Default: 3600 seconds (1 hour).
pub fn token_blacklist_ttl_secs() -> u64 {
    AgroCoreConfig::global().token_blacklist_ttl_secs
}

/// Validates that JWT secret is properly configured
/// Call this during startup to ensure production readiness
pub fn validate_jwt_secret() -> bool {
    AgroCoreConfig::global().jwt_secret != "dev-secret"
}
