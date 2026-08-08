use agrocore_lpis_providers::config::LpisProvidersConfig;

fn main() {
    let config = LpisProvidersConfig::load().unwrap_or_default();
    println!("Providers count: {}", config.providers.len());
    for (k, v) in &config.providers {
        println!("  {}: {} (enabled: {})", k, v.base_url, v.enabled);
    }
    println!("Cache backend: {:?}", config.cache.backend);
    println!("Cache TTL: {}", config.cache.default_ttl_seconds);
}
