//! LPIS (Land Parcel Identification System) Abstraction Layer
//!
//! This module provides a unified interface for different countries' LPIS implementations.
//! Each EU member state has its own LPIS implementation (SIGPAC in Spain, iLPIS in Portugal,
//! etc.) but they all serve the same purpose under the EU Common Agricultural Policy.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Country code for LPIS systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LpisCountry {
    /// Spain - SIGPAC (Sistema de Información Geográfica de Parcelas Agrícolas)
    Es,
    /// Portugal - iLPIS (integrated Land Parcel Identification System)
    Pt,
    /// France - RPG (Registre Parcellaire Graphique)
    Fr,
    /// Italy - SIAN (Sistema Informativo Agricolo Nazionale)
    It,
    /// Netherlands - BRP (Basisregistratie Percelen)
    Nl,
    /// Germany - LPIS
    De,
    /// Poland - LPIS
    Pl,
    /// Austria - INVEKOS
    At,
    /// Generic/Other EU country
    Other,
}

impl std::fmt::Display for LpisCountry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LpisCountry::Es => write!(f, "ES"),
            LpisCountry::Pt => write!(f, "PT"),
            LpisCountry::Fr => write!(f, "FR"),
            LpisCountry::It => write!(f, "IT"),
            LpisCountry::Nl => write!(f, "NL"),
            LpisCountry::De => write!(f, "DE"),
            LpisCountry::Pl => write!(f, "PL"),
            LpisCountry::At => write!(f, "AT"),
            LpisCountry::Other => write!(f, "OTHER"),
        }
    }
}

impl std::str::FromStr for LpisCountry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ES" => Ok(LpisCountry::Es),
            "PT" => Ok(LpisCountry::Pt),
            "FR" => Ok(LpisCountry::Fr),
            "IT" => Ok(LpisCountry::It),
            "NL" => Ok(LpisCountry::Nl),
            "DE" => Ok(LpisCountry::De),
            "PL" => Ok(LpisCountry::Pl),
            "AT" => Ok(LpisCountry::At),
            _ => Ok(LpisCountry::Other),
        }
    }
}

// ... rest of file unchanged ...

/// LPIS parcel reference format specification
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisReferenceFormat {
    pub country: LpisCountry,
    pub pattern: String,
    pub description: String,
    pub example: String,
}

impl LpisReferenceFormat {
    pub fn for_country(country: LpisCountry) -> Self {
        match country {
            LpisCountry::Es => Self {
                country: LpisCountry::Es,
                pattern: r"^ES\d{2}\d{3}\d{3}\d{3}\d{5}\d{3}$".to_string(),
                description: "SIGPAC reference: ES + province(2) + municipality(3) + aggregate(3) + zone(3) + parcel(5) + enclosure(3)".to_string(),
                example: "ES411234567890123".to_string(),
            },
            LpisCountry::Pt => Self {
                country: LpisCountry::Pt,
                pattern: r"^PT\d{2}\d{3}\d{4}\d{4}$".to_string(),
                description: "iLPIS reference: PT + NUTS2(2) + municipality(3) + freguesia(4) + parcel(4)".to_string(),
                example: "PT1112345678".to_string(),
            },
            LpisCountry::Fr => Self {
                country: LpisCountry::Fr,
                pattern: r"^FR\d{2}\d{3}\d{4}$".to_string(),
                description: "RPG reference: FR + department(2) + commune(3) + section+parcel(4)".to_string(),
                example: "FR131234567".to_string(),
            },
            LpisCountry::It => Self {
                country: LpisCountry::It,
                pattern: r"^IT\d{2}\d{3}\d{4}$".to_string(),
                description: "SIAN reference: IT + province(2) + municipality(3) + parcel(4)".to_string(),
                example: "IT123456789".to_string(),
            },
            LpisCountry::Nl => Self {
                country: LpisCountry::Nl,
                pattern: r"^NL\d{14}$".to_string(),
                description: "BRP reference: NL + 14-digit perceel_id".to_string(),
                example: "NL12345678901234".to_string(),
            },
            LpisCountry::De => Self {
                country: LpisCountry::De,
                pattern: r"^DE\d{16}$".to_string(),
                description: "German LPIS reference: DE + 16-digit parcel ID".to_string(),
                example: "DE1234567890123456".to_string(),
            },
            LpisCountry::Pl => Self {
                country: LpisCountry::Pl,
                pattern: r"^PL\d{24}$".to_string(),
                description: "Polish LPIS reference: PL + 24-digit parcel ID".to_string(),
                example: "PL123456789012345678901234".to_string(),
            },
            LpisCountry::At => Self {
                country: LpisCountry::At,
                pattern: r"^AT\d{14}$".to_string(),
                description: "INVEKOS reference: AT + 14-digit parcel ID".to_string(),
                example: "AT12345678901234".to_string(),
            },
            LpisCountry::Other => Self {
                country: LpisCountry::Other,
                pattern: r".*".to_string(),
                description: "Generic LPIS reference format".to_string(),
                example: "REF123456".to_string(),
            },
        }
    }
}

/// Query parameters for LPIS parcel search
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct LpisQuery {
    pub country: Option<LpisCountry>,
    pub province: Option<String>,
    pub municipality: Option<String>,
    pub aggregate: Option<String>,
    pub zone: Option<String>,
    pub polygon: Option<String>,
    pub parcel: Option<String>,
    pub enclosure: Option<String>,
    pub reference: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

/// LPIS parcel data structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisParcel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub country: LpisCountry,
    pub reference: String,
    pub province: Option<String>,
    pub municipality: Option<String>,
    pub aggregate: Option<String>,
    pub zone: Option<String>,
    pub polygon: Option<String>,
    pub parcel: Option<String>,
    pub enclosure: Option<String>,
    pub usage_code: Option<String>,
    pub usage_description: Option<String>,
    pub geometry: serde_json::Value,
    pub area_hectares: Option<f64>,
    pub official_area_ha: Option<f64>,
    pub source_dataset: Option<String>,
    pub source_year: Option<i16>,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated response for LPIS parcels
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedLpisResponse {
    pub data: Vec<LpisParcel>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// Spatial search query for nearby parcels
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisNearPointQuery {
    pub country: LpisCountry,
    pub lng: f64,
    pub lat: f64,
    pub radius_m: Option<f64>,
}

/// Trait for LPIS data providers
#[async_trait::async_trait]
pub trait LpisProvider: Send + Sync {
    /// Get the country this provider supports
    fn country(&self) -> LpisCountry;

    /// Get the reference format for this provider
    fn reference_format(&self) -> LpisReferenceFormat {
        LpisReferenceFormat::for_country(self.country())
    }

    /// List parcels with filters
    async fn list_parcels(
        &self,
        tenant_id: Uuid,
        query: LpisQuery,
    ) -> Result<PaginatedLpisResponse, String>;

    /// Get a single parcel by ID
    async fn get_parcel(&self, tenant_id: Uuid, parcel_id: Uuid) -> Result<LpisParcel, String>;

    /// Search parcels near a point
    async fn search_near_point(
        &self,
        tenant_id: Uuid,
        query: LpisNearPointQuery,
    ) -> Result<PaginatedLpisResponse, String>;

    /// Validate a parcel geometry against this LPIS
    async fn validate_geometry(
        &self,
        tenant_id: Uuid,
        geometry: &geo::Geometry<f64>,
    ) -> Result<LpisValidationResult, String>;

    /// Import parcels from source data (GeoParquet, Shapefile, etc.)
    async fn import_parcels(
        &self,
        tenant_id: Uuid,
        source_path: &str,
    ) -> Result<LpisImportResult, String>;
}

/// Validation result for LPIS geometry validation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisValidationResult {
    pub is_valid: bool,
    pub matched_parcels: Vec<LpisParcel>,
    pub area_difference_ha: Option<f64>,
    pub warnings: Vec<String>,
}

/// Import result for LPIS data import
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisImportResult {
    pub total: u64,
    pub created: u64,
    pub updated: u64,
    pub skipped: u64,
    pub errors: Vec<String>,
}

/// Registry for LPIS providers
pub struct LpisRegistry {
    providers:
        std::collections::HashMap<LpisCountry, std::sync::Arc<dyn LpisProvider + Send + Sync>>,
}

impl Default for LpisRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LpisRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: std::sync::Arc<dyn LpisProvider + Send + Sync>) {
        self.providers.insert(provider.country(), provider);
    }

    pub fn get(&self, country: LpisCountry) -> Option<&(dyn LpisProvider + Send + Sync)> {
        self.providers.get(&country).map(|p| p.as_ref())
    }

    pub fn get_for_tenant(
        &self,
        tenant_country: LpisCountry,
    ) -> Option<&(dyn LpisProvider + Send + Sync)> {
        self.get(tenant_country)
    }

    pub fn get_arc(
        &self,
        country: LpisCountry,
    ) -> Option<std::sync::Arc<dyn LpisProvider + Send + Sync>> {
        self.providers.get(&country).cloned()
    }

    pub fn available_countries(&self) -> Vec<LpisCountry> {
        self.providers.keys().copied().collect()
    }
}

/// Configuration for an LPIS provider (serializable for API/Settings)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisProviderConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub rate_limit_requests_per_second: u32,
    pub rate_limit_burst_size: u32,
    pub enabled: bool,
}

impl Default for LpisProviderConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            timeout_seconds: 30,
            cache_ttl_seconds: 3600,
            rate_limit_requests_per_second: 10,
            rate_limit_burst_size: 20,
            enabled: true,
        }
    }
}
