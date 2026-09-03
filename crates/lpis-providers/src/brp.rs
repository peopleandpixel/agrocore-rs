//! Netherlands BRP (Basisregistratie Percelen) Provider
//!
//! The Netherlands has excellent open data access via PDOK (Publieke Dienstverlening Op de Kaart).
//! BRP data is available via WFS (Web Feature Service) and WMS.
//!
//! WFS Endpoint: https://geodata.nationaalgeoregister.nl/brppercelen/wfs
//! GetCapabilities: https://geodata.nationaalgeoregister.nl/brppercelen/wfs?service=WFS&version=2.0.0&request=GetCapabilities
//!
//! The BRP contains all agricultural parcels in the Netherlands with:
//! - perceel_id (unique identifier)
//! - geometrie (polygon)
//! - gewascode (crop code)
//! - oppervlakte (area in m²)
//! - gemeente (municipality code)

use crate::{cache::LpisCache, config::ProviderConfig};
use agrocore_shared::lpis::{LpisCountry, LpisProvider};
use async_trait::async_trait;
use chrono::Datelike;
use geo::{Centroid, Geometry, Polygon};
use geojson::{GeoJson, Geometry as GeoJsonGeometry};
use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use quick_xml::de::from_str;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use agrocore_logging::warn;
#[allow(dead_code)]
const DEFAULT_BRP_WFS_URL: &str = "https://geodata.nationaalgeoregister.nl/brppercelen/wfs";

#[derive(Debug, Error)]
pub enum BrpError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XML parsing failed: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("GeoJSON parsing failed: {0}")]
    GeoJson(#[from] geojson::Error),
    #[error("Geometry conversion failed: {0}")]
    Geometry(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Base provider error: {0}")]
    BaseProvider(#[from] crate::base::BaseProviderError),
}

#[derive(Debug, Deserialize, Serialize)]
struct WfsFeatureCollection {
    #[serde(rename = "featureMember")]
    features: Vec<WfsFeature>,
    #[serde(rename = "numberMatched")]
    number_matched: Option<u64>,
    #[serde(rename = "numberReturned")]
    number_returned: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WfsFeature {
    #[serde(rename = "BRPpercelen")]
    parcels: Vec<BrpParcel>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BrpParcel {
    #[serde(rename = "perceel_id")]
    perceel_id: String,
    #[serde(rename = "gewascode")]
    gewascode: Option<String>,
    #[serde(rename = "oppervlakte")]
    oppervlakte: Option<f64>,
    #[serde(rename = "gemeente")]
    gemeente: Option<String>,
    #[serde(rename = "geometrie")]
    geometrie: GmlGeometry,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmlGeometry {
    #[serde(rename = "Polygon")]
    polygon: GmlPolygon,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmlPolygon {
    #[serde(rename = "exterior")]
    exterior: GmlLinearRing,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmlLinearRing {
    #[serde(rename = "LinearRing")]
    linear_ring: GmlCoordinates,
}

#[derive(Debug, Deserialize, Serialize)]
struct GmlCoordinates {
    #[serde(rename = "posList")]
    pos_list: String,
}

#[derive(Clone)]
pub struct BrpProvider {
    client: Client,
    base_url: String,
    config: ProviderConfig,
    cache: Option<Arc<LpisCache>>,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>>,
}

impl BrpProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        // Initialize rate limiter
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
            config,
            cache: None,
            rate_limiter,
        }
    }

    pub fn with_cache(mut self, cache: Arc<LpisCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Build WFS GetFeature request URL
    fn build_query_url(&self, query: &agrocore_shared::lpis::LpisQuery) -> String {
        let mut url = format!(
            "{}?service=WFS&version=2.0.0&request=GetFeature&typeNames=BRPpercelen&outputFormat=application/json",
            self.base_url
        );

        // Add CQL filter for filtering
        let mut filters = Vec::new();

        if let Some(gemeente) = &query.municipality {
            filters.push(format!("gemeente='{}'", gemeente));
        }

        if let Some(gewascode) = &query.reference {
            filters.push(format!("gewascode='{}'", gewascode));
        }

        if !filters.is_empty() {
            let cql_filter = filters.join(" AND ");
            url.push_str(&format!("&CQL_FILTER={}", urlencoding::encode(&cql_filter)));
        }

        // Pagination
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(1000);
        let start_index = (page - 1) * per_page;

        url.push_str(&format!("&startIndex={}&count={}", start_index, per_page));

        url
    }

    /// Execute HTTP request with rate limiting
    async fn execute_request(&self, url: &str) -> Result<reqwest::Response, BrpError> {
        // Apply rate limiting
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self.client.get(url).send().await.map_err(BrpError::Http)?;

        Ok(response)
    }

    /// Get cached response or fetch and cache
    async fn get_cached_or_fetch(
        &self,
        cache_key: &str,
        url: &str,
        ttl: Duration,
    ) -> Result<String, BrpError> {
        // Try cache first
        if let Some(cache) = &self.cache
            && let Some(cached) = cache.get(cache_key).await
        {
            return Ok(String::from_utf8_lossy(&cached).to_string());
        }

        // Fetch from network
        let response = self.execute_request(url).await?;
        let text = response.text().await.map_err(BrpError::Http)?;

        // Store in cache
        if let Some(cache) = &self.cache {
            cache
                .set_with_ttl(cache_key.to_string(), text.as_bytes().to_vec(), ttl)
                .await?;
        }

        Ok(text)
    }

    /// Convert GML coordinates to GeoJSON polygon
    fn gml_to_polygon(&self, gml: &GmlGeometry) -> Result<Polygon<f64>, BrpError> {
        let coords_str = &gml.polygon.exterior.linear_ring.pos_list;
        let coords: Vec<f64> = coords_str
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if !coords.len().is_multiple_of(2) {
            return Err(BrpError::Geometry("Invalid coordinate count".to_string()));
        }

        let mut points = Vec::new();
        for chunk in coords.chunks(2) {
            points.push(geo::coord! { x: chunk[0], y: chunk[1] });
        }

        // Ensure ring is closed
        #[allow(clippy::collapsible_if)]
        if points.first() != points.last() {
            if let Some(first) = points.first().copied() {
                points.push(first);
            }
        }

        let line_string = geo::LineString::new(points);
        let polygon = Polygon::new(line_string, vec![]);
        Ok(polygon)
    }

    /// Convert BRP parcel to LpisParcel
    fn parcel_to_lpis(
        &self,
        parcel: &BrpParcel,
        tenant_id: Uuid,
    ) -> Result<agrocore_shared::lpis::LpisParcel, BrpError> {
        let polygon = self.gml_to_polygon(&parcel.geometrie)?;
        let geometry = Geometry::Polygon(polygon);

        // Convert geometry to GeoJSON value
        let geo_json: GeoJson = GeoJsonGeometry::from(&geometry).into();
        let geometry_value =
            serde_json::to_value(geo_json).map_err(|e| BrpError::Geometry(e.to_string()))?;

        // Parse area from m² to hectares
        let area_ha = parcel.oppervlakte.map(|a| a / 10000.0);

        // Generate a UUID from perceel_id
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, parcel.perceel_id.as_bytes());

        Ok(agrocore_shared::lpis::LpisParcel {
            id,
            tenant_id,
            country: LpisCountry::Nl,
            reference: parcel.perceel_id.clone(),
            province: None,
            municipality: parcel.gemeente.clone(),
            aggregate: None,
            zone: None,
            polygon: None,
            parcel: Some(parcel.perceel_id.clone()),
            enclosure: None,
            usage_code: parcel.gewascode.clone(),
            usage_description: None,
            geometry: geometry_value,
            area_hectares: area_ha,
            official_area_ha: area_ha,
            source_dataset: Some("BRP".to_string()),
            source_year: Some(chrono::Utc::now().year() as i16),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[async_trait]
impl LpisProvider for BrpProvider {
    fn country(&self) -> LpisCountry {
        LpisCountry::Nl
    }

    async fn list_parcels(
        &self,
        tenant_id: Uuid,
        query: agrocore_shared::lpis::LpisQuery,
    ) -> Result<agrocore_shared::lpis::PaginatedLpisResponse, String> {
        let url = self.build_query_url(&query);
        let cache_key = format!("brp:list:{}", urlencoding::encode(&url));

        let text = self
            .get_cached_or_fetch(
                &cache_key,
                &url,
                Duration::from_secs(self.config.cache_ttl_seconds),
            )
            .await
            .map_err(|e| format!("BRP request failed: {}", e))?;

        // Parse WFS response
        let wfs_fc: WfsFeatureCollection =
            from_str(&text).map_err(|e| format!("XML parsing failed: {}", e))?;

        let mut parcels = Vec::new();
        for feature in wfs_fc.features {
            for parcel in feature.parcels {
                match self.parcel_to_lpis(&parcel, tenant_id) {
                    Ok(lpis_parcel) => parcels.push(lpis_parcel),
                    Err(e) => warn!("Failed to convert parcel: {}", e),
                }
            }
        }

        let total = wfs_fc.number_matched.unwrap_or(parcels.len() as u64);
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50);
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u64;

        Ok(agrocore_shared::lpis::PaginatedLpisResponse {
            data: parcels,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    async fn get_parcel(
        &self,
        tenant_id: Uuid,
        parcel_id: Uuid,
    ) -> Result<agrocore_shared::lpis::LpisParcel, String> {
        let query = agrocore_shared::lpis::LpisQuery {
            country: Some(LpisCountry::Nl),
            reference: Some(parcel_id.to_string()),
            ..Default::default()
        };

        let response = self.list_parcels(tenant_id, query).await?;
        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| "Parcel not found".to_string())
    }

    async fn search_near_point(
        &self,
        tenant_id: Uuid,
        query: agrocore_shared::lpis::LpisNearPointQuery,
    ) -> Result<agrocore_shared::lpis::PaginatedLpisResponse, String> {
        let radius_km = query.radius_m.unwrap_or(1000.0) / 1000.0;
        let bbox = format!(
            "BBOX(geometrie,{},{},{},{})",
            query.lng - radius_km / 111.0,
            query.lat - radius_km / 111.0,
            query.lng + radius_km / 111.0,
            query.lat + radius_km / 111.0,
        );

        let url = format!(
            "{}?service=WFS&version=2.0.0&request=GetFeature&typeNames=BRPpercelen&outputFormat=application/json&CQL_FILTER={}&count=100",
            self.base_url,
            urlencoding::encode(&bbox)
        );

        let cache_key = format!("brp:near:{}", urlencoding::encode(&bbox));

        let text = self
            .get_cached_or_fetch(
                &cache_key,
                &url,
                Duration::from_secs(self.config.cache_ttl_seconds),
            )
            .await
            .map_err(|e| format!("BRP spatial search failed: {}", e))?;

        let wfs_fc: WfsFeatureCollection =
            from_str(&text).map_err(|e| format!("XML parsing failed: {}", e))?;

        let mut parcels = Vec::new();
        for feature in wfs_fc.features {
            for parcel in feature.parcels {
                if let Ok(lpis_parcel) = self.parcel_to_lpis(&parcel, tenant_id) {
                    parcels.push(lpis_parcel);
                }
            }
        }

        let total_count = parcels.len();

        Ok(agrocore_shared::lpis::PaginatedLpisResponse {
            data: parcels,
            total: total_count as u64,
            page: 1,
            per_page: 100,
            total_pages: 1,
        })
    }

    async fn validate_geometry(
        &self,
        _tenant_id: Uuid,
        geometry: &geo::Geometry<f64>,
    ) -> Result<agrocore_shared::lpis::LpisValidationResult, String> {
        let matched = if let geo::Geometry::Polygon(poly) = geometry {
            let centroid = poly.centroid();
            let centroid_coord = centroid.expect("Polygon should have centroid");
            let query = agrocore_shared::lpis::LpisNearPointQuery {
                country: LpisCountry::Nl,
                lng: centroid_coord.x(),
                lat: centroid_coord.y(),
                radius_m: Some(100.0),
            };

            let response = self.search_near_point(Uuid::nil(), query).await?;
            response.data
        } else {
            vec![]
        };

        let is_empty = matched.is_empty();
        Ok(agrocore_shared::lpis::LpisValidationResult {
            is_valid: !is_empty,
            matched_parcels: matched,
            area_difference_ha: None,
            warnings: if is_empty {
                vec!["No matching BRP parcels found".to_string()]
            } else {
                vec![]
            },
        })
    }

    async fn import_parcels(
        &self,
        tenant_id: Uuid,
        _source_path: &str,
    ) -> Result<agrocore_shared::lpis::LpisImportResult, String> {
        let query = agrocore_shared::lpis::LpisQuery {
            country: Some(LpisCountry::Nl),
            per_page: Some(1000),
            ..Default::default()
        };

        let response = self.list_parcels(tenant_id, query).await?;

        Ok(agrocore_shared::lpis::LpisImportResult {
            total: response.total,
            created: response.data.len() as u64,
            updated: 0,
            skipped: 0,
            errors: vec![],
        })
    }
}

impl Default for BrpProvider {
    fn default() -> Self {
        Self::new(ProviderConfig {
            base_url: DEFAULT_BRP_WFS_URL.to_string(),
            ..Default::default()
        })
    }
}
