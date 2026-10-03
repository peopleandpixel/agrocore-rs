use actix_web::{App, http::StatusCode, test};
use agrocore_api::handlers::configure;

/// Every settings route must be registered.
///
/// Asserts "not 404", because a missing route answers 404 while a registered
/// one answers 401 or 403 without a token. This is what caught the LPIS routes
/// disappearing when the file-based handlers were replaced by the database-backed
/// ones.
#[actix_web::test]
async fn test_settings_routes_configured() {
    let app = test::init_service(App::new().configure(configure)).await;

    let registered = [
        ("GET", "/api/v1/settings"),
        ("PUT", "/api/v1/settings"),
        ("GET", "/api/v1/settings/keys"),
        ("POST", "/api/v1/settings/restore-defaults"),
        ("GET", "/api/v1/settings/groups"),
        ("GET", "/api/v1/settings/lpis/providers"),
        ("GET", "/api/v1/settings/backup"),
        ("PUT", "/api/v1/settings/backup"),
        ("GET", "/api/v1/settings/notification"),
        ("PUT", "/api/v1/settings/notification"),
        ("GET", "/api/v1/settings/weather"),
        ("PUT", "/api/v1/settings/weather"),
        ("GET", "/api/v1/settings/locale"),
        ("PUT", "/api/v1/settings/locale"),
        ("GET", "/api/v1/settings/company"),
        ("PUT", "/api/v1/settings/company"),
    ];

    for (method, uri) in registered {
        let req = match method {
            "GET" => test::TestRequest::get().uri(uri),
            "PUT" => test::TestRequest::put().uri(uri),
            "POST" => test::TestRequest::post().uri(uri),
            _ => unreachable!("unsupported method in the list"),
        };

        let resp = test::call_service(&app, req.to_request()).await;
        assert_ne!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{method} {uri} is not registered"
        );
    }
}

/// The key-scoped routes use a path segment, so they are checked separately.
#[actix_web::test]
async fn test_setting_key_routes_configured() {
    let app = test::init_service(App::new().configure(configure)).await;

    for (method, uri) in [
        ("GET", "/api/v1/settings/company.name"),
        ("PUT", "/api/v1/settings/company.name"),
        ("DELETE", "/api/v1/settings/company.name"),
    ] {
        let req = match method {
            "GET" => test::TestRequest::get().uri(uri),
            "PUT" => test::TestRequest::put().uri(uri),
            "DELETE" => test::TestRequest::delete().uri(uri),
            _ => unreachable!("unsupported method in the list"),
        };

        let resp = test::call_service(&app, req.to_request()).await;
        assert_ne!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{method} {uri} is not registered"
        );
    }
}

#[cfg(test)]
mod config_serialization_tests {
    use agrocore_api::dto::LpisProviderConfig;

    #[test]
    fn test_provider_config_roundtrip() {
        let config = LpisProviderConfig {
            base_url: "https://test.example.com/wfs".to_string(),
            timeout_seconds: 45,
            cache_ttl_seconds: 7200,
            rate_limit_requests_per_second: 20,
            rate_limit_burst_size: 30,
            enabled: true,
            configured: Some(String::from("PT")),
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: LpisProviderConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.base_url, parsed.base_url);
        assert_eq!(config.timeout_seconds, parsed.timeout_seconds);
        assert_eq!(config.cache_ttl_seconds, parsed.cache_ttl_seconds);
        assert_eq!(
            config.rate_limit_requests_per_second,
            parsed.rate_limit_requests_per_second
        );
        assert_eq!(config.rate_limit_burst_size, parsed.rate_limit_burst_size);
        assert_eq!(config.enabled, parsed.enabled);
        assert_eq!(config.configured, parsed.configured);
    }

    #[test]
    fn test_providers_config_roundtrip() {
        let config = agrocore_lpis_providers::config::LpisProvidersConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: agrocore_lpis_providers::config::LpisProvidersConfig =
            toml::from_str(&toml_str).unwrap();

        assert_eq!(config.providers.len(), parsed.providers.len());
        assert_eq!(config.cache.enabled, parsed.cache.enabled);
        assert_eq!(config.cache.backend, parsed.cache.backend);
        assert_eq!(
            config.cache.default_ttl_seconds,
            parsed.cache.default_ttl_seconds
        );
        assert_eq!(config.cache.max_entries, parsed.cache.max_entries);
    }

    #[test]
    fn test_cache_backend_enum() {
        let memory = agrocore_lpis_providers::config::CacheBackend::Memory;
        assert_eq!(memory.to_string(), "memory");

        let redis = agrocore_lpis_providers::config::CacheBackend::Redis;
        assert_eq!(redis.to_string(), "redis");
    }
}
