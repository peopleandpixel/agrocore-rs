use crate::entities::{Boundary, CropType, GeoPoint, Plot, RowConfig, SigpacData, Site, SiteProperty, SiteType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: crate::entities::CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    #[validate(range(min = 0.0))]
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<crate::entities::BbchStage>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub center: Option<crate::entities::GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Boundary>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
    pub is_temporary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportSitesRequest {
    #[validate(length(min = 1, max = 1000))]
    pub sites: Vec<ImportSiteDto>,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
    pub source: ImportSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ImportSource {
    GeoJSON,
    Shapefile,
    SIGPAC,
    REGEPAC,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportResult {
    pub total: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<ImportError>,
    pub warnings: Vec<ImportWarning>,
    pub duplicate_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportError {
    pub index: usize,
    pub label: String,
    pub error: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImportWarning {
    pub index: usize,
    pub label: String,
    pub warning: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DuplicateDetectionResult {
    pub is_duplicate: bool,
    pub existing_site_id: Option<Uuid>,
    pub match_type: DuplicateMatchType,
    pub similarity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DuplicateMatchType {
    ExactBoundary,
    SigpacMatch,
    RegepacMatch,
    HighSimilarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisValidationResult {
    pub is_valid: bool,
    pub sigpac_reference: Option<String>,
    pub area_difference: Option<f64>,
    pub boundary_difference: Option<f64>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoJsonImportRequest {
    pub features: Vec<GeoJsonFeature>,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: GeoJsonGeometry,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoJsonGeometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShapefileImportRequest {
    pub file_base64: String,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
    pub encoding: Option<String>,
}