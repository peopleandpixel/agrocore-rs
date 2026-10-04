//! Caching layer for LPIS providers

use agrocore_logging::{debug, warn};
use moka::future::Cache;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use redis::cmd;
use std::sync::Arc;
use std::time::Duration;

/// Unified cache interface supporting both in-memory (moka) and distributed (redis) caching
#[derive(Clone)]
pub struct LpisCache {
    memory: Option<Cache<String, Arc<[u8]>>>,
    redis: Option<ConnectionManager>,
    default_ttl: Duration,
    enabled: bool,
}

impl LpisCache {
    /// Builds an in-memory cache without touching the network.
    ///
    /// `new` is async because it may open a Redis connection. `create_default_registry`
    /// is a synchronous constructor called from `AppState::new`, so the memory backend
    /// — which is the default and needs no connection — gets its own constructor. A
    /// Redis-backed cache cannot be built here and is wired where an async context
    /// exists.
    ///
    /// Returns `None` when the configuration disables the cache, rather than a cache
    /// that silently never hits.
    pub fn memory_if_enabled(config: &crate::config::CacheConfig) -> Option<Self> {
        if !config.enabled || !matches!(config.backend, crate::config::CacheBackend::Memory) {
            return None;
        }
        Some(Self {
            memory: Some(
                Cache::builder()
                    .max_capacity(config.max_entries as u64)
                    .time_to_live(Duration::from_secs(config.default_ttl_seconds))
                    .build(),
            ),
            redis: None,
            default_ttl: Duration::from_secs(config.default_ttl_seconds),
            enabled: true,
        })
    }

    /// Create a new cache instance from configuration
    pub async fn new(
        config: &crate::config::CacheConfig,
    ) -> Result<Self, crate::base::BaseProviderError> {
        let memory =
            if config.enabled && matches!(config.backend, crate::config::CacheBackend::Memory) {
                Some(
                    Cache::builder()
                        .max_capacity(config.max_entries as u64)
                        .time_to_live(Duration::from_secs(config.default_ttl_seconds))
                        .build(),
                )
            } else {
                None
            };

        let redis =
            if config.enabled && matches!(config.backend, crate::config::CacheBackend::Redis) {
                if let Some(redis_url) = &config.redis_url {
                    let client = redis::Client::open(redis_url.as_str()).map_err(|e| {
                        crate::base::BaseProviderError::Configuration(format!(
                            "Redis client error: {}",
                            e
                        ))
                    })?;
                    let manager = ConnectionManager::new(client).await.map_err(|e| {
                        crate::base::BaseProviderError::Configuration(format!(
                            "Redis connection manager error: {}",
                            e
                        ))
                    })?;
                    Some(manager)
                } else {
                    warn!("Redis backend selected but no redis_url provided");
                    None
                }
            } else {
                None
            };

        let has_memory = memory.is_some();
        let has_redis = redis.is_some();

        if !has_memory && !has_redis && config.enabled {
            warn!("Cache enabled but no backend configured, falling back to in-memory");
        }

        Ok(Self {
            memory,
            redis,
            default_ttl: Duration::from_secs(config.default_ttl_seconds),
            enabled: config.enabled && (has_memory || has_redis),
        })
    }

    /// Get a value from cache
    pub async fn get(&self, key: &str) -> Option<Arc<[u8]>> {
        if !self.enabled {
            return None;
        }

        // Try memory cache first
        if let Some(mem) = &self.memory
            && let Some(value) = mem.get(key).await
        {
            debug!("Cache hit (memory): {}", key);
            return Some(value);
        }

        // Try Redis
        if let Some(conn) = &self.redis {
            let mut conn = conn.clone();
            if let Ok(value) = conn.get::<_, Vec<u8>>(key).await {
                debug!("Cache hit (redis): {}", key);
                return Some(Arc::from(value));
            }
        }

        debug!("Cache miss: {}", key);
        None
    }

    /// Set a value in cache with default TTL
    pub async fn set(
        &self,
        key: String,
        value: Vec<u8>,
    ) -> Result<(), crate::base::BaseProviderError> {
        if !self.enabled {
            return Ok(());
        }

        // Set in memory cache — Arc::from avoids clone of Vec<u8> bytes
        if let Some(mem) = &self.memory {
            mem.insert(key.clone(), Arc::from(value.as_slice())).await;
        }

        // Set in Redis
        if let Some(conn) = &self.redis {
            let mut conn = conn.clone();
            let _: () = conn
                .set_ex(&key, &value, self.default_ttl.as_secs())
                .await
                .map_err(|e| {
                    crate::base::BaseProviderError::Cache(crate::cache::CacheError::Redis(e))
                })?;
        }

        Ok(())
    }

    /// Set a value with custom TTL
    pub async fn set_with_ttl(
        &self,
        key: String,
        value: Vec<u8>,
        ttl: Duration,
    ) -> Result<(), crate::base::BaseProviderError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(mem) = &self.memory {
            mem.insert(key.clone(), Arc::from(value.as_slice())).await;
        }

        if let Some(conn) = &self.redis {
            let mut conn = conn.clone();
            let _: () = conn
                .set_ex(&key, &value, ttl.as_secs())
                .await
                .map_err(|e| {
                    crate::base::BaseProviderError::Cache(crate::cache::CacheError::Redis(e))
                })?;
        }

        Ok(())
    }

    /// Invalidate a key
    pub async fn invalidate(&self, key: &str) -> Result<(), crate::base::BaseProviderError> {
        if let Some(mem) = &self.memory {
            mem.invalidate(key).await;
        }

        if let Some(conn) = &self.redis {
            let mut conn = conn.clone();
            let _: () = conn.del(key).await.map_err(|e| {
                crate::base::BaseProviderError::Cache(crate::cache::CacheError::Redis(e))
            })?;
        }

        Ok(())
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<(), crate::base::BaseProviderError> {
        if let Some(mem) = &self.memory {
            mem.invalidate_all();
        }

        if let Some(conn) = &self.redis {
            let mut conn = conn.clone();
            let _: () = cmd("FLUSHALL").query_async(&mut conn).await.map_err(|e| {
                crate::base::BaseProviderError::Cache(crate::cache::CacheError::Redis(e))
            })?;
        }

        Ok(())
    }
}

/// Error types for cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// A cached GET for the WFS providers that do not use `BaseClient`.
///
/// Six providers — RPG, iLPIS, SIAN, the German and Polish LPIS services and
/// INVEKOS — each held a bare `reqwest::Client` and issued `.get(url).send()` per
/// request. `LpisCache` existed and `BaseClient` and `BrpProvider` used it, but
/// those six had no cache field at all, so every call went to the national WFS
/// service. They are read-only reference datasets that change on the order of days,
/// so the hourly default TTL is appropriate.
///
/// Deliberately not on `BaseClient`: those six carry their own error type and their
/// own status handling, and routing them through `BaseClient` would have meant
/// rewriting six providers to change one cache lookup. This is the narrow change.
///
/// `key_prefix` keeps the namespaces apart — a French RPG parcel and a German LPIS
/// parcel for the same bounding box must not collide, since the cache key would
/// otherwise be identical while the responses are not.
pub(crate) async fn cached_get(
    client: &reqwest::Client,
    cache: Option<&Arc<LpisCache>>,
    key_prefix: &str,
    url: &str,
) -> Result<String, String> {
    if let Some(cache) = cache {
        let key = format!("{key_prefix}:{}", urlencoding::encode(url));
        if let Some(hit) = cache.get(&key).await {
            debug!("LPIS cache hit: {}", key);
            return Ok(String::from_utf8_lossy(&hit).to_string());
        }
    }

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("provider returned {}", response.status()));
    }

    let text = response.text().await.map_err(|e| e.to_string())?;

    if let Some(cache) = cache {
        let key = format!("{key_prefix}:{}", urlencoding::encode(url));
        // A cache write that fails must not fail the request: the data was fetched
        // successfully, and the caller has no way to act on a cache failure. The
        // next request simply misses and fetches again.
        if let Err(e) = cache.set(key, text.as_bytes().to_vec()).await {
            warn!("LPIS cache write failed: {}", e);
        }
    }

    Ok(text)
}
