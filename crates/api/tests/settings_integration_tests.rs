use actix_web::{App, http::StatusCode, test};
use agrocore_api::handlers::configure;

#[actix_web::test]
async fn test_settings_routes_configured() {
    // Just verify the routes are configured correctly
    let app = test::init_service(App::new().configure(configure)).await;

    // Test that the routes exist (they return 401/403/404 instead of 404 Not Found for unknown routes)
    let req = test::TestRequest::get()
        .uri("/api/v1/settings/lpis/providers")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Route exists but returns 401/403 because no auth - not 404
    assert_ne!(resp.status(), StatusCode::NOT_FOUND);

    let req = test::TestRequest::get()
        .uri("/api/v1/settings/lpis")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), StatusCode::NOT_FOUND);

    let req = test::TestRequest::put()
        .uri("/api/v1/settings/lpis")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
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
