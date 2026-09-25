//! Site DTOs

use agrocore_domain::entities::{BbchStage, CropType, SiteType};
use agrocore_domain::entities::{
    Boundary, GeoPoint, LpisData, Plot, RowConfig, SigpacData, SiteProperty,
};
use agrocore_shared::lpis::LpisCountry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedSiteResponse {
    pub data: Vec<SiteDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SiteDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    #[serde(rename = "site_type")]
    pub site_type: SiteType,
    #[serde(rename = "crop_type")]
    pub crop_type: CropType,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub plots: serde_json::Value,
    pub row_config: Option<serde_json::Value>,
    pub bbch_stage: Option<serde_json::Value>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Boundary>,
    pub properties: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: bool,
    pub is_temporary: bool,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl From<agrocore_domain::entities::site::Site> for SiteDto {
    fn from(s: agrocore_domain::entities::site::Site) -> Self {
        Self {
            id: s.id,
            tenant_id: s.tenant_id,
            label: s.label,
            site_type: s.site_type,
            crop_type: s.crop_type,
            variety: s.variety,
            area: s.area,
            gross_area: s.gross_area,
            plots: s.plots,
            row_config: s.row_config,
            bbch_stage: s.bbch_stage,
            planted_date: s.planted_date,
            cleared_date: s.cleared_date,
            soil_type: s.soil_type,
            slope: s.slope,
            slope_facing: s.slope_facing,
            altitude: s.altitude,
            organic: s.organic,
            organic_eligible: s.organic_eligible,
            center: s.center,
            boundary: s.boundary,
            properties: s.properties,
            custom_fields: s.custom_fields,
            note1: s.note1,
            note2: s.note2,
            is_active: s.is_active,
            is_temporary: s.is_temporary,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
            created_by: s.created_by,
            updated_by: s.updated_by,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<String>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Boundary>,
    pub properties: Option<Vec<SiteProperty>>,
}

impl From<CreateSiteDto> for agrocore_domain::entities::site::CreateSiteDto {
    fn from(dto: CreateSiteDto) -> Self {
        Self {
            label: dto.label,
            site_type: dto.site_type,
            crop_type: dto.crop_type,
            variety: dto.variety,
            area: dto.area,
            gross_area: dto.gross_area,
            plots: dto.plots,
            row_config: dto.row_config,
            bbch_stage: dto.bbch_stage,
            planted_date: dto.planted_date,
            cleared_date: dto.cleared_date,
            soil_type: dto.soil_type,
            slope: dto.slope,
            slope_facing: dto.slope_facing,
            altitude: dto.altitude,
            organic: dto.organic,
            center: dto.center,
            boundary: dto.boundary,
            properties: dto.properties,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateSiteDto {
    pub label: Option<String>,
    pub site_type: Option<SiteType>,
    pub crop_type: Option<CropType>,
    pub variety: Option<String>,
    pub area: Option<f64>,
    pub gross_area: Option<f64>,
    pub plots: Option<serde_json::Value>,
    pub row_config: Option<serde_json::Value>,
    pub bbch_stage: Option<String>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub lpis_country: Option<LpisCountry>,
    pub lpis_data: Option<LpisData>,
    pub properties: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
    pub is_temporary: Option<bool>,
    pub boundary: Option<Boundary>,
}

impl From<UpdateSiteDto> for agrocore_domain::entities::site::UpdateSiteDto {
    fn from(dto: UpdateSiteDto) -> Self {
        Self {
            label: dto.label,
            site_type: dto.site_type,
            crop_type: dto.crop_type,
            variety: dto.variety,
            area: dto.area,
            gross_area: dto.gross_area,
            plots: dto.plots,
            row_config: dto.row_config,
            bbch_stage: dto.bbch_stage,
            planted_date: dto.planted_date,
            cleared_date: dto.cleared_date,
            soil_type: dto.soil_type,
            slope: dto.slope,
            slope_facing: dto.slope_facing,
            altitude: dto.altitude,
            organic: dto.organic,
            organic_eligible: dto.organic_eligible,
            center: dto.center,
            sigpac_data: dto.sigpac_data,
            regepac_id: dto.regepac_id,
            lpis_country: dto.lpis_country,
            lpis_data: dto.lpis_data,
            properties: dto.properties,
            custom_fields: dto.custom_fields,
            note1: dto.note1,
            note2: dto.note2,
            is_active: dto.is_active,
            is_temporary: dto.is_temporary,
            boundary: dto.boundary,
        }
    }
}
