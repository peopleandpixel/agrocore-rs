use crate::config::ProviderConfig;
use agrocore_logging::warn;
use agrocore_shared::lpis::{LpisCountry, LpisProvider};
use async_trait::async_trait;
use chrono::Datelike;
use geo::{Centroid, Geometry, Polygon};
use geojson::{GeoJson, Geometry as GeoJsonGeometry};
use reqwest::Client;
use serde::{Deserialize, Serialize};
/// Italy SIAN (Sistema Informativo Agricolo Nazionale) Provider
///
/// Italy's LPIS system managed by AGEA and regional bodies.
/// Data available via regional geoportals and dati.gov.it.
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[allow(dead_code)]
const SIAN_WFS_URL: &str = "https://www.sian.it/wfs";

#[derive(Debug, Error)]
pub enum SianError {
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
    #[serde(rename = "SIAN")]
    parcels: Vec<SianParcel>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SianParcel {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "provincia")]
    provincia: Option<String>,
    #[serde(rename = "comune")]
    comune: Option<String>,
    #[serde(rename = "particella")]
    particella: Option<String>,
    #[serde(rename = "foglio")]
    foglio: Option<String>,
    #[serde(rename = "coltura")]
    coltura: Option<String>,
    #[serde(rename = "superficie")]
    superficie: Option<f64>,
    #[serde(rename = "geometria")]
    geometria: GmlGeometry,
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
pub struct SianProvider {
    client: Client,
    base_url: String,
}

impl SianProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url.clone(),
        }
    }

    fn build_query_url(&self, query: &agrocore_shared::lpis::LpisQuery) -> String {
        let mut url = format!(
            "{}?service=WFS&version=2.0.0&request=GetFeature&typeNames=SIAN&outputFormat=application/json",
            self.base_url
        );

        let mut filters = Vec::new();

        if let Some(prov) = &query.province {
            filters.push(format!("provincia='{}'", prov));
        }

        if let Some(comune) = &query.municipality {
            filters.push(format!("comune='{}'", comune));
        }

        if let Some(coltura) = &query.reference {
            filters.push(format!("coltura='{}'", coltura));
        }

        if !filters.is_empty() {
            let cql_filter = filters.join(" AND ");
            url.push_str(&format!("&CQL_FILTER={}", urlencoding::encode(&cql_filter)));
        }

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(1000);
        let start_index = (page - 1) * per_page;

        url.push_str(&format!("&startIndex={}&count={}", start_index, per_page));

        url
    }

    fn gml_to_polygon(&self, gml: &GmlGeometry) -> Result<Polygon<f64>, SianError> {
        let coords_str = &gml.polygon.exterior.linear_ring.pos_list;
        let coords: Vec<f64> = coords_str
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if !coords.len().is_multiple_of(2) {
            return Err(SianError::Geometry("Invalid coordinate count".to_string()));
        }

        let mut points = Vec::new();
        for chunk in coords.chunks(2) {
            points.push(geo::coord! { x: chunk[0], y: chunk[1] });
        }

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

    fn parcel_to_lpis(
        &self,
        parcel: &SianParcel,
        tenant_id: Uuid,
    ) -> Result<agrocore_shared::lpis::LpisParcel, SianError> {
        let polygon = self.gml_to_polygon(&parcel.geometria)?;
        let geometry = Geometry::Polygon(polygon);

        let geo_json: GeoJson = GeoJsonGeometry::from(&geometry).into();
        let geometry_value =
            serde_json::to_value(geo_json).map_err(|e| SianError::Geometry(e.to_string()))?;

        let area_ha = parcel.superficie.map(|a| a / 10000.0);

        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, parcel.id.as_bytes());

        Ok(agrocore_shared::lpis::LpisParcel {
            id,
            tenant_id,
            country: LpisCountry::It,
            reference: parcel.id.clone(),
            province: parcel.provincia.clone(),
            municipality: parcel.comune.clone(),
            aggregate: parcel.foglio.clone(),
            zone: None,
            polygon: parcel.particella.clone(),
            parcel: parcel.particella.clone(),
            enclosure: None,
            usage_code: parcel.coltura.clone(),
            usage_description: None,
            geometry: geometry_value,
            area_hectares: area_ha,
            official_area_ha: area_ha,
            source_dataset: Some("SIAN".to_string()),
            source_year: Some(chrono::Utc::now().year() as i16),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[async_trait]
impl LpisProvider for SianProvider {
    fn country(&self) -> LpisCountry {
        LpisCountry::It
    }

    async fn list_parcels(
        &self,
        tenant_id: Uuid,
        query: agrocore_shared::lpis::LpisQuery,
    ) -> Result<agrocore_shared::lpis::PaginatedLpisResponse, String> {
        let url = self.build_query_url(&query);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("SIAN request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("SIAN API error: {}", response.status()));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;

        let wfs_fc: WfsFeatureCollection =
            quick_xml::de::from_str(&text).map_err(|e| format!("XML parsing failed: {}", e))?;

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
            country: Some(LpisCountry::It),
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
            "BBOX(geometria,{},{},{},{})",
            query.lng - radius_km / 111.0,
            query.lat - radius_km / 111.0,
            query.lng + radius_km / 111.0,
            query.lat + radius_km / 111.0,
        );

        let url = format!(
            "{}?service=WFS&version=2.0.0&request=GetFeature&typeNames=SIAN&outputFormat=application/json&CQL_FILTER={}&count=100",
            self.base_url,
            urlencoding::encode(&bbox)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("SIAN spatial search failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("SIAN API error: {}", response.status()));
        }

        let text = response.text().await.map_err(|e| e.to_string())?;

        let wfs_fc: WfsFeatureCollection =
            quick_xml::de::from_str(&text).map_err(|e| format!("XML parsing failed: {}", e))?;

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
                country: LpisCountry::It,
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
                vec!["No matching SIAN parcels found".to_string()]
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
            country: Some(LpisCountry::It),
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

impl Default for SianProvider {
    fn default() -> Self {
        Self::new(ProviderConfig {
            base_url: "https://example.com/wfs".to_string(),
            ..Default::default()
        })
    }
}
