use std::sync::OnceLock;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn jwt_secret() -> &'static str {
    JWT_SECRET
        .get_or_init(|| std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into()))
        .as_str()
}
