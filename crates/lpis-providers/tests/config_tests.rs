#[cfg(test)]
mod tests {
    use agrocore_lpis_providers::base::BaseClient;
    use agrocore_lpis_providers::config::{
        CacheBackend, CacheConfig, LpisProvidersConfig, ProviderConfig, RateLimitConfig,
        RetryConfig,
    };
    use std::time::Duration;

    #[test]
    fn test_base_client_creation() {
        let config = ProviderConfig {
            base_url: "https://example.com/wfs".to_string(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig::default(),
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);
        assert_eq!(client.base_url, "https://example.com/wfs");
        assert_eq!(client.cache_ttl, Duration::from_secs(3600));
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.base_delay, Duration::from_millis(100));
        assert_eq!(client.max_delay, Duration::from_secs(5));
        assert_eq!(client.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_base_client_with_rate_limit() {
        let config = ProviderConfig {
            base_url: "https://example.com/wfs".to_string(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig {
                requests_per_second: 5,
                burst_size: 10,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);
        assert!(client.rate_limiter.is_some());
    }

    #[test]
    fn test_base_client_without_rate_limit() {
        let config = ProviderConfig {
            base_url: "https://example.com/wfs".to_string(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig {
                requests_per_second: 0,
                burst_size: 0,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);
        assert!(client.rate_limiter.is_none());
    }

    #[test]
    fn test_lpis_providers_config_default() {
        let config = LpisProvidersConfig::default();

        // Check all 8 providers exist
        assert!(config.providers.contains_key("NL"));
        assert!(config.providers.contains_key("ES"));
        assert!(config.providers.contains_key("FR"));
        assert!(config.providers.contains_key("PT"));
        assert!(config.providers.contains_key("IT"));
        assert!(config.providers.contains_key("DE"));
        assert!(config.providers.contains_key("PL"));
        assert!(config.providers.contains_key("AT"));

        // Check cache config
        assert!(config.cache.enabled);
        assert_eq!(config.cache.backend, CacheBackend::Memory);
        assert_eq!(config.cache.default_ttl_seconds, 3600);
        assert_eq!(config.cache.max_entries, 10000);

        // Check rate limit config
        assert_eq!(config.rate_limit.requests_per_second, 10);
        assert_eq!(config.rate_limit.burst_size, 20);

        // Check retry config
        assert_eq!(config.retry.max_attempts, 3);
        assert_eq!(config.retry.base_delay_ms, 100);
        assert_eq!(config.retry.max_delay_ms, 5000);
        assert_eq!(config.retry.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.base_url, "");
        assert!(config.auth.is_none());
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.cache_ttl_seconds, 3600);
        assert!(config.enabled);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.backend, CacheBackend::Memory);
        assert!(config.redis_url.is_none());
        assert_eq!(config.default_ttl_seconds, 3600);
        assert_eq!(config.max_entries, 10000);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig {
            base_url: "https://example.com/wfs".to_string(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: RateLimitConfig {
                requests_per_second: 5,
                burst_size: 10,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("base_url"));
        assert!(toml_str.contains("timeout_seconds"));
        assert!(toml_str.contains("rate_limit"));

        let parsed: ProviderConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.base_url, config.base_url);
        assert_eq!(parsed.timeout_seconds, config.timeout_seconds);
        assert_eq!(
            parsed.rate_limit.requests_per_second,
            config.rate_limit.requests_per_second
        );
    }

    #[test]
    fn test_lpis_providers_config_serialization() {
        let config = LpisProvidersConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("NL"));
        assert!(toml_str.contains("ES"));
        assert!(toml_str.contains("FR"));
        assert!(toml_str.contains("providers"));
        assert!(toml_str.contains("cache"));
        assert!(toml_str.contains("rate_limit"));
    }
}
