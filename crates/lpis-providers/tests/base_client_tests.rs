#[cfg(test)]
mod tests {
    use agrocore_lpis_providers::base::BaseClient;
    use agrocore_lpis_providers::config::ProviderConfig;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_base_client_execute_request_mock() {
        // Start a mock server
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            base_url: mock_server.uri(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: agrocore_lpis_providers::config::RateLimitConfig {
                requests_per_second: 10,
                burst_size: 20,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);
        let response = client
            .execute_request(&format!("{}/test", mock_server.uri()))
            .await;
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_base_client_get_cached_or_fetch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/cached"))
            .respond_with(ResponseTemplate::new(200).set_body_string("cached response"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            base_url: mock_server.uri(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: agrocore_lpis_providers::config::RateLimitConfig {
                requests_per_second: 10,
                burst_size: 20,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);
        let result = client
            .get_cached_or_fetch("test_key", &format!("{}/cached", mock_server.uri()))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("cached response"));
    }

    #[tokio::test]
    async fn test_base_client_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rate"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let config = ProviderConfig {
            base_url: mock_server.uri(),
            auth: None,
            timeout_seconds: 30,
            rate_limit: agrocore_lpis_providers::config::RateLimitConfig {
                requests_per_second: 100,
                burst_size: 10,
            },
            cache_ttl_seconds: 3600,
            enabled: true,
        };

        let client = BaseClient::new(&config);

        // Make multiple rapid requests to test rate limiting
        let start = std::time::Instant::now();
        for _ in 0..3 {
            let _ = client
                .execute_request(&format!("{}/rate", mock_server.uri()))
                .await;
        }
        let elapsed = start.elapsed();

        // With 100 req/s, 3 requests should complete quickly
        assert!(elapsed < Duration::from_secs(5));
    }
}
