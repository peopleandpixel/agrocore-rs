//! The LPIS response cache must actually be used (tasks.md I5).
//!
//! `LpisCache`, `LpisCache::new`, `CacheConfig` and `BaseClient::with_cache` all
//! existed and were correct. `BaseClient::new` and `BrpProvider::new` both set
//! `cache: None`, and `create_default_registry()` called only the constructors — so
//! nothing ever passed a cache, and the `if let Some(cache)` branch in
//! `get_cached_or_fetch` was unreachable in every deployment. Every SIGPAC and BRP
//! parcel listing went to the national WFS service on every call, on public
//! endpoints with rate limits, for data that does not change within an hour.
//!
//! The tests below use `wiremock`, so they can assert the effect rather than the
//! wiring: a second identical request must not produce a second HTTP call.

use agrocore_lpis_providers::cache::LpisCache;
use agrocore_lpis_providers::config::{CacheConfig, ProviderConfig, RateLimitConfig};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn provider_config(base_url: &str) -> ProviderConfig {
    ProviderConfig {
        base_url: base_url.to_string(),
        auth: None,
        timeout_seconds: 30,
        rate_limit: RateLimitConfig {
            requests_per_second: 0,
            burst_size: 0,
        },
        cache_ttl_seconds: 3600,
        enabled: true,
    }
}

/// A second identical request is served from the cache, so the mock server sees
/// exactly one call.
#[tokio::test]
async fn a_cached_client_fetches_once() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/parcels"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"features":[]}"#))
        .mount(&server)
        .await;

    let config = provider_config(&server.uri());
    let cache = LpisCache::memory_if_enabled(&CacheConfig::default()).expect("memory cache");
    let client =
        agrocore_lpis_providers::base::BaseClient::new(&config).with_cache(Arc::new(cache));

    let url = format!("{}/parcels", server.uri());

    let first = client
        .get_cached_or_fetch("key-1", &url)
        .await
        .expect("first");
    let second = client
        .get_cached_or_fetch("key-1", &url)
        .await
        .expect("second");

    assert_eq!(first, second, "the cached body must match the fetched one");
    assert_eq!(first, r#"{"features":[]}"#);
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "the second request must be served from the cache, not the network"
    );
}

/// Without a cache the same client calls the server twice. This is the condition
/// the registry was in, and it is asserted so the test above cannot pass by a
/// cache that is silently always-hit regardless of what was fetched.
#[tokio::test]
async fn an_uncached_client_fetches_twice() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/parcels"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"features":[]}"#))
        .mount(&server)
        .await;

    let config = provider_config(&server.uri());
    let client = agrocore_lpis_providers::base::BaseClient::new(&config);

    let url = format!("{}/parcels", server.uri());
    client
        .get_cached_or_fetch("key-1", &url)
        .await
        .expect("first");
    client
        .get_cached_or_fetch("key-1", &url)
        .await
        .expect("second");

    assert_eq!(
        server.received_requests().await.unwrap().len(),
        2,
        "without a cache both requests must reach the server — otherwise the cached \\
         test above proves nothing"
    );
}

/// Distinct keys do not share an entry.
#[tokio::test]
async fn the_cache_is_keyed_by_cache_key() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("body"))
        .mount(&server)
        .await;

    let config = provider_config(&server.uri());
    let cache = LpisCache::memory_if_enabled(&CacheConfig::default()).expect("memory cache");
    let client =
        agrocore_lpis_providers::base::BaseClient::new(&config).with_cache(Arc::new(cache));

    let url = format!("{}/anything", server.uri());
    client.get_cached_or_fetch("key-a", &url).await.expect("a");
    client.get_cached_or_fetch("key-b", &url).await.expect("b");

    assert_eq!(
        server.received_requests().await.unwrap().len(),
        2,
        "two different cache keys must both be fetched"
    );
}

/// A disabled configuration produces no cache at all, rather than a cache that is
/// written to but never read — which would look like the cache was working.
#[test]
fn a_disabled_config_produces_no_cache() {
    let config = CacheConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(
        LpisCache::memory_if_enabled(&config).is_none(),
        "a disabled cache config must yield None so the providers are registered \\
         without one"
    );
}

/// A Redis backend cannot be built from the synchronous registry constructor.
#[test]
fn a_redis_config_yields_no_memory_cache() {
    use agrocore_lpis_providers::config::CacheBackend;
    let config = CacheConfig {
        enabled: true,
        backend: CacheBackend::Redis,
        redis_url: Some("redis://localhost:6379".into()),
        ..Default::default()
    };
    assert!(
        LpisCache::memory_if_enabled(&config).is_none(),
        "the synchronous constructor must not pretend to build a Redis cache"
    );
}

/// The default configuration is the one that has to work, since that is what
/// `create_default_registry` uses.
#[test]
fn the_default_configuration_yields_a_memory_cache() {
    let cache = LpisCache::memory_if_enabled(&CacheConfig::default())
        .expect("the default config is enabled with the memory backend");
    let _ = cache;
}

/// The registry is where the wiring happened, and the mechanics tests above do not
/// cover it: they build their own client. This asserts the wiring directly, on the
/// source, because a registry that constructs successfully while attaching no cache
/// looks identical from the outside — which is exactly what was wrong.
///
/// Verified against a broken tree: replacing the `memory_if_enabled` call in
/// `create_default_registry` with `None` fails this test and leaves every other test
/// in this file green.
#[test]
fn the_default_registry_attaches_a_cache() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"),
    )
    .expect("read lpis-providers lib.rs");

    let code: String = src
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    let body = code
        .split("pub fn create_default_registry")
        .nth(1)
        .expect("create_default_registry must exist")
        .split("\npub fn ")
        .next()
        .unwrap_or_default();

    assert!(
        body.contains("memory_if_enabled"),
        "create_default_registry must build the cache from the configuration"
    );

    // Both providers with a cache path in their fetch code.
    for provider in ["sigpac::SigpacProvider", "brp::BrpProvider"] {
        assert!(
            body.contains(provider),
            "{provider} must still be registered"
        );
    }
    // Every provider registered in the default registry has a cache lookup in its
    // fetch path, so every one of them must receive the shared cache. A provider
    // left on `new` keeps `cache: None` and silently never reads it — which is
    // exactly how this defect stayed invisible: the code compiled, the tests were
    // green, and the cache was never consulted.
    let expected = 8;
    assert_eq!(
        body.matches(".with_cache(").count(),
        expected,
        "all {expected} registered providers must receive the shared cache"
    );

    for provider in [
        "sigpac::SigpacProvider",
        "brp::BrpProvider",
        "rpg::RpgProvider",
        "ilpis::IlpisProvider",
        "sian::SianProvider",
        "lpis_de::GermanLpisProvider",
        "lpis_pl::PolishLpisProvider",
        "invkos::InvekosProvider",
    ] {
        assert!(body.contains(provider), "{provider} must be registered");
    }
}
