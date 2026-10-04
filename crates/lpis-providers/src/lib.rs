//! LPIS Providers Module
//!
//! This crate contains implementations of LPIS (Land Parcel Identification System)
//! providers for different European countries. Each provider implements the
//! `LpisProvider` trait from `agrocore_shared`.

pub mod base; // Base client with caching, rate limiting, retry
pub mod brp; // Netherlands - Basisregistratie Percelen
pub mod cache; // Caching layer
pub mod config; // Configuration
pub mod ilpis; // Portugal - iLPIS
pub mod invkos; // Austria - INVEKOS
pub mod lpis_de; // Germany - LPIS
pub mod lpis_pl; // Poland - LPIS
pub mod rpg; // France - Registre Parcellaire Graphique
pub mod sian; // Italy - SIAN
pub mod sigpac; // Spain - SIGPAC

use agrocore_shared::lpis::{LpisCountry, LpisProvider, LpisRegistry};
use std::sync::Arc;

/// Create and populate the default LPIS registry with all available providers
pub fn create_default_registry() -> LpisRegistry {
    let mut registry = LpisRegistry::new();

    // The response cache, built once and shared by every provider that has a cache
    // lookup. `BaseClient::new` and `BrpProvider::new` both set `cache: None`, and
    // this function used to call only the constructors — so `get_cached_or_fetch`
    // always took the network branch and neither `with_cache` had a caller. Every
    // SIGPAC and BRP parcel listing hit the national WFS service directly, on a
    // public endpoint with a rate limit, for data that does not change within the
    // hour.
    //
    // `CacheConfig::default()` is enabled with the memory backend and a one-hour
    // TTL, which is what `memory_if_enabled` builds from. It returns `None` when the
    // configuration disables caching, and the providers are then registered exactly
    // as before — a disabled cache must not become a cache that is written to but
    // never read.
    let cache_config = crate::config::CacheConfig::default();
    let cache = crate::cache::LpisCache::memory_if_enabled(&cache_config).map(Arc::new);

    // Netherlands - BRP (Basisregistratie Percelen) - Best open data access
    {
        let provider = brp::BrpProvider::new(crate::config::ProviderConfig {
            base_url: "https://geodata.nationaalgeoregister.nl/brppercelen/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Spain - SIGPAC
    {
        let provider = sigpac::SigpacProvider::new(crate::config::ProviderConfig {
            base_url: "https://sigpac.mapa.gob.es/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // France - RPG
    {
        let provider = rpg::RpgProvider::new(crate::config::ProviderConfig {
            base_url: "https://geoservices.ign.fr/rpg/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Portugal - iLPIS
    {
        let provider = ilpis::IlpisProvider::new(crate::config::ProviderConfig {
            base_url: "https://ide.ifap.pt/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Italy - SIAN
    {
        let provider = sian::SianProvider::new(crate::config::ProviderConfig {
            base_url: "https://www.sian.it/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Germany - LPIS
    {
        let provider = lpis_de::GermanLpisProvider::new(crate::config::ProviderConfig {
            base_url: "https://geodienste.bfn.de/lpis/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Poland - LPIS
    {
        let provider = lpis_pl::PolishLpisProvider::new(crate::config::ProviderConfig {
            base_url: "https://geoportal.arrim.gov.pl/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    // Austria - INVEKOS
    {
        let provider = invkos::InvekosProvider::new(crate::config::ProviderConfig {
            base_url: "https://data.gv.at/wfs".to_string(),
            ..Default::default()
        });
        registry.register(Arc::new(match &cache {
            Some(c) => provider.with_cache(c.clone()),
            None => provider,
        }));
    }

    registry
}

/// Get a provider for a specific country
pub fn get_provider_for_country(
    registry: &LpisRegistry,
    country: LpisCountry,
) -> Option<Arc<dyn LpisProvider + Send + Sync>> {
    registry.get_arc(country)
}

// Re-export config and cache types
pub use base::BaseProviderError;
pub use cache::{CacheError, LpisCache};
pub use config::{
    CacheBackend, CacheConfig, LpisProvidersConfig, ProviderConfig, RateLimitConfig, RetryConfig,
};
