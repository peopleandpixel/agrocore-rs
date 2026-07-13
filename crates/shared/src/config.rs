use std::sync::OnceLock;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

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
