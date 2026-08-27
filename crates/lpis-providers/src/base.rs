//! Base Provider Implementation with Caching and Rate Limiting
//!
//! This module provides common functionality for all LPIS providers:
//! - HTTP client with timeout
//! - Caching via LpisCache (Memory/Redis)
//! - Rate limiting via governor

use crate::{cache::LpisCache, config::ProviderConfig};
use agrocore_shared::with_retry;
use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaseProviderError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Cache error: {0}")]
    Cache(#[from] crate::cache::CacheError),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

/// Base client with caching, rate limiting, and retry support
#[derive(Clone)]
pub struct BaseClient {
    pub client: Client,
    pub base_url: String,
    pub cache: Option<Arc<LpisCache>>,
    pub rate_limiter:
        Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>>,
    pub cache_ttl: Duration,
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl BaseClient {
    /// Create a new base client from provider config
    pub fn new(config: &ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        // Initialize rate limiter from config
        let rate_limiter = if config.rate_limit.requests_per_second > 0 {
            let quota = Quota::per_second(
                NonZeroU32::new(config.rate_limit.requests_per_second)
                    .unwrap_or(NonZeroU32::new(10).unwrap()),
            )
            .allow_burst(
                NonZeroU32::new(config.rate_limit.burst_size)
                    .unwrap_or(NonZeroU32::new(20).unwrap()),
            );
            Some(Arc::new(RateLimiter::direct(quota)))
        } else {
            None
        };

        Self {
            client,
            base_url: config.base_url.clone(),
            cache: None,
            rate_limiter,
            cache_ttl: Duration::from_secs(config.cache_ttl_seconds),
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }

    /// Add cache to the client
    pub fn with_cache(mut self, cache: Arc<LpisCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Execute HTTP request with rate limiting and retries
    pub async fn execute_request(&self, url: &str) -> Result<reqwest::Response, BaseProviderError> {
        // Apply rate limiting
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let url_owned = url.to_string();
        let response = with_retry(
            "execute HTTP request",
            self.max_retries,
            self.base_delay.as_secs(),
            || {
                let url = url_owned.clone();
                let client = self.client.clone();
                Box::pin(async move {
                    client
                        .get(&url)
                        .send()
                        .await
                        .map_err(BaseProviderError::Http)
                })
            },
        )
        .await?;

        Ok(response)
    }

    /// Get cached response or fetch and cache with retries
    pub async fn get_cached_or_fetch(
        &self,
        cache_key: &str,
        url: &str,
    ) -> Result<String, BaseProviderError> {
        // Try cache first
        #[allow(clippy::collapsible_if)]
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(cache_key).await {
                tracing::debug!("Cache hit: {}", cache_key);
                return Ok(String::from_utf8_lossy(&cached).to_string());
            }
        }

        // Fetch from network with retries
        let mut attempt = 0;
        let mut delay = self.base_delay;

        loop {
            if let Some(limiter) = &self.rate_limiter {
                limiter.until_ready().await;
            }

            let response = self.client.get(url).send().await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let text = resp.text().await.map_err(BaseProviderError::Http)?;

                        // Store in cache
                        if let Some(cache) = &self.cache {
                            cache
                                .set_with_ttl(
                                    cache_key.to_string(),
                                    text.as_bytes().to_vec(),
                                    self.cache_ttl,
                                )
                                .await?;
                        }

                        return Ok(text);
                    } else {
                        attempt += 1;
                        if attempt >= self.max_retries {
                            return Err(BaseProviderError::Configuration(format!(
                                "API error: {}",
                                resp.status()
                            )));
                        }
                        // Wait before retry with exponential backoff
                        tokio::time::sleep(delay).await;
                        delay = Duration::from_millis(
                            (delay.as_millis() as f64 * self.backoff_multiplier) as u64,
                        )
                        .min(self.max_delay);
                    }
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.max_retries {
                        return Err(BaseProviderError::Http(e));
                    }
                    tokio::time::sleep(delay).await;
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.backoff_multiplier) as u64,
                    )
                    .min(self.max_delay);
                }
            }
        }
    }
}

/// Trait for providers that want to use the base client
pub trait BaseProvider: Send + Sync {
    fn base_client(&self) -> &BaseClient;
    fn cache_ttl(&self) -> Duration;
}
