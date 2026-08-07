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

    // Netherlands - BRP (Basisregistratie Percelen) - Best open data access
    registry.register(Arc::new(brp::BrpProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://geodata.nationaalgeoregister.nl/brppercelen/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Spain - SIGPAC
    registry.register(Arc::new(sigpac::SigpacProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://sigpac.mapa.gob.es/wfs".to_string(),
            ..Default::default()
        },
    )));

    // France - RPG
    registry.register(Arc::new(rpg::RpgProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://geoservices.ign.fr/rpg/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Portugal - iLPIS
    registry.register(Arc::new(ilpis::IlpisProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://ide.ifap.pt/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Italy - SIAN
    registry.register(Arc::new(sian::SianProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://www.sian.it/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Germany - LPIS
    registry.register(Arc::new(lpis_de::GermanLpisProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://geodienste.bfn.de/lpis/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Poland - LPIS
    registry.register(Arc::new(lpis_pl::PolishLpisProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://geoportal.arrim.gov.pl/wfs".to_string(),
            ..Default::default()
        },
    )));

    // Austria - INVEKOS
    registry.register(Arc::new(invkos::InvekosProvider::new(
        crate::config::ProviderConfig {
            base_url: "https://data.gv.at/wfs".to_string(),
            ..Default::default()
        },
    )));

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
