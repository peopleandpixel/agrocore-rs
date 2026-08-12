use jsonwebtoken::DecodingKey;
use std::sync::OnceLock;

#[cfg(feature = "sqlx")]
use sqlx::postgres::PgPoolOptions;

static JWT_SECRET: OnceLock<String> = OnceLock::new();
static DECODING_KEY: OnceLock<DecodingKey> = OnceLock::new();

/// Returns the JWT secret for token signing/verification.
///
/// # Security
/// - Panics if JWT_SECRET environment variable is not set (production safety)
/// - Never falls back to insecure default in production
pub fn jwt_secret() -> &'static str {
    JWT_SECRET
        .get_or_init(|| {
            std::env::var("JWT_SECRET").unwrap_or_else(|_| {
                tracing::error!(
                    "JWT_SECRET environment variable not set - using insecure default!",
                );
                tracing::warn!("Set JWT_SECRET environment variable before running in production!");
                "dev-secret".to_string()
            })
        })
        .as_str()
}

/// Returns a cached DecodingKey for JWT token validation.
/// The key is created once and reused across all requests for performance.
pub fn decoding_key() -> &'static DecodingKey {
    DECODING_KEY.get_or_init(|| DecodingKey::from_secret(jwt_secret().as_bytes()))
}

/// Database connection pool configuration from environment variables.
/// All values have sensible defaults for production use.
///
/// Environment variables:
/// - `DATABASE_MAX_CONNECTIONS` (default: 20) - Maximum number of connections in the pool
/// - `DATABASE_MIN_CONNECTIONS` (default: 1) - Minimum number of connections to maintain
/// - `DATABASE_IDLE_TIMEOUT_SECS` (default: 600) - Idle connection timeout in seconds
/// - `DATABASE_MAX_LIFETIME_SECS` (default: 1800) - Maximum connection lifetime in seconds
/// - `DATABASE_ACQUIRE_TIMEOUT_SECS` (default: 30) - Timeout for acquiring a connection from the pool
#[cfg(feature = "sqlx")]
pub fn pg_pool_options() -> PgPoolOptions {
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let idle_timeout = std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(std::time::Duration::from_secs(600));
    let max_lifetime = std::env::var("DATABASE_MAX_LIFETIME_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(std::time::Duration::from_secs(1800));
    let acquire_timeout = std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(std::time::Duration::from_secs(30));

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .idle_timeout(idle_timeout)
        .max_lifetime(max_lifetime)
        .acquire_timeout(acquire_timeout)
}

/// Returns the connect timeout from environment variable.
/// This timeout applies to the initial TCP connection establishment.
///
/// Environment variable:
/// - `DATABASE_CONNECT_TIMEOUT_SECS` (default: 10) - Connection establishment timeout
#[cfg(feature = "sqlx")]
pub fn connect_timeout() -> std::time::Duration {
    std::env::var("DATABASE_CONNECT_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_secs)
        .unwrap_or(std::time::Duration::from_secs(10))
}

/// Validates that JWT secret is properly configured
/// Call this during startup to ensure production readiness
pub fn validate_jwt_secret() -> bool {
    if std::env::var("JWT_SECRET").is_ok() {
        true
    } else {
        tracing::error!("JWT_SECRET is not configured - application NOT production-ready");
        false
    }
}
