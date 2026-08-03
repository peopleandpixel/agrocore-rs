//! LPIS Providers Module
//!
//! This crate contains implementations of LPIS (Land Parcel Identification System)
//! providers for different European countries. Each provider implements the
//! `LpisProvider` trait from `agrocore_shared`.

pub mod brp; // Netherlands - Basisregistratie Percelen
pub mod ilpis; // Portugal - iLPIS
pub mod invkos;
pub mod lpis_de; // Germany - LPIS
pub mod lpis_pl; // Poland - LPIS
pub mod rpg; // France - Registre Parcellaire Graphique
pub mod sian; // Italy - SIAN
pub mod sigpac; // Spain - SIGPAC // Austria - INVEKOS

use agrocore_shared::lpis::{LpisCountry, LpisProvider, LpisRegistry};
use std::sync::Arc;

/// Create and populate the default LPIS registry with all available providers
pub fn create_default_registry() -> LpisRegistry {
    let mut registry = LpisRegistry::new();

    // Netherlands - BRP (Basisregistratie Percelen) - Best open data access
    registry.register(Arc::new(brp::BrpProvider::new()));

    // Spain - SIGPAC
    registry.register(Arc::new(sigpac::SigpacProvider::new()));

    // France - RPG
    registry.register(Arc::new(rpg::RpgProvider::new()));

    // Portugal - iLPIS
    registry.register(Arc::new(ilpis::IlpisProvider::new()));

    // Italy - SIAN
    registry.register(Arc::new(sian::SianProvider::new()));

    // Germany - LPIS
    registry.register(Arc::new(lpis_de::GermanLpisProvider::new()));

    // Poland - LPIS
    registry.register(Arc::new(lpis_pl::PolishLpisProvider::new()));

    // Austria - INVEKOS
    registry.register(Arc::new(invkos::InvekosProvider::new()));

    registry
}

/// Get a provider for a specific country
pub fn get_provider_for_country(
    registry: &LpisRegistry,
    country: LpisCountry,
) -> Option<Arc<dyn LpisProvider + Send + Sync>> {
    registry.get_arc(country)
}
