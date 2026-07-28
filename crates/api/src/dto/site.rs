//! Site DTOs

use agrocore_domain::entities::site::{Boundary, GeoPoint, SiteProperty};
use agrocore_domain::entities::{BbchStage, CropType, SiteType};
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
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub bbch_stage: Option<BbchStage>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::site::Site> for SiteDto {
    fn from(s: agrocore_domain::entities::site::Site) -> Self {
        Self {
            id: s.id,
            tenant_id: s.tenant_id.into(),
            label: s.label,
            site_type: s.site_type,
            crop_type: s.crop_type,
            variety: s.variety,
            area: s.area,
            gross_area: s.gross_area,
            bbch_stage: s.bbch_stage,
            soil_type: s.soil_type,
            slope: s.slope,
            altitude: s.altitude,
            organic: s.organic,
            center: s.center,
            boundary: s.boundary.map(|b| b.0),
            properties: s.properties,
            is_active: s.is_active,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
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
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub plots: Option<Vec<agrocore_domain::entities::site::Plot>>,
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
            center: dto.center,
            boundary: dto.boundary.map(Boundary),
            plots: dto.plots,
            properties: dto.properties,
            row_config: None,
            bbch_stage: None,
            planted_date: None,
            soil_type: None,
            slope: None,
            slope_facing: None,
            altitude: None,
            organic: None,
            sigpac_data: None,
            regepac_id: None,
            custom_fields: None,
            note1: None,
            note2: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: Option<f64>,
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<agrocore_domain::entities::site::Plot>>,
    pub row_config: Option<agrocore_domain::entities::site::RowConfig>,
    pub bbch_stage: Option<BbchStage>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<agrocore_domain::entities::site::SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
}

impl From<UpdateSiteDto> for agrocore_domain::entities::site::UpdateSiteDto {
    fn from(dto: UpdateSiteDto) -> Self {
        Self {
            label: dto.label,
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
            sigpac_data: dto.sigpac_data,
            regepac_id: dto.regepac_id,
            boundary: dto.boundary.map(Boundary),
            properties: dto.properties,
            custom_fields: dto.custom_fields,
            note1: dto.note1,
            note2: dto.note2,
            is_active: dto.is_active,
        }
    }
}