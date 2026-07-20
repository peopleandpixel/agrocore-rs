//! Specialized Culture DTOs (Vineyard, Olive, Kelter Delivery)

use agrocore_domain::entities::olive::{
    CreateOliveGroveDto as DomainCreateOliveGroveDto,
    CreateOliveOilRecordDto as DomainCreateOliveOilRecordDto, OilGrade, OliveGrove, OliveOilRecord,
    UpdateOliveGroveDto as DomainUpdateOliveGroveDto,
    UpdateOliveOilRecordDto as DomainUpdateOliveOilRecordDto,
};
use agrocore_domain::entities::vineyard::{
    CreateKelterDeliveryDto as DomainCreateKelterDeliveryDto,
    CreateVineyardDto as DomainCreateVineyardDto, KelterDelivery,
    UpdateKelterDeliveryDto as DomainUpdateKelterDeliveryDto,
    UpdateVineyardDto as DomainUpdateVineyardDto, Vineyard,
};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Vineyard DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedVineyardResponse {
    pub data: Vec<VineyardDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct VineyardDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<QualityGrade>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Vineyard> for VineyardDto {
    fn from(v: Vineyard) -> Self {
        Self {
            id: v.id,
            tenant_id: v.tenant_id.into(),
            site_id: v.site_id,
            doc_area: v.doc_area,
            vintage: v.vintage,
            grape_variety: v.grape_variety,
            brix_at_harvest: v.brix_at_harvest,
            ph_at_harvest: v.ph_at_harvest,
            acidity: v.acidity,
            yield_tons: v.yield_tons,
            quality_grade: v.quality_grade,
            slope_percent: v.slope_percent,
            altitude_m: v.altitude_m,
            is_organic: v.is_organic,
            certification_body: v.certification_body,
            certification_number: v.certification_number,
            created_at: v.created_at.to_rfc3339(),
            updated_at: v.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateVineyardDto {
    pub site_id: Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<QualityGrade>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

impl From<CreateVineyardDto> for DomainCreateVineyardDto {
    fn from(dto: CreateVineyardDto) -> Self {
        Self {
            site_id: dto.site_id,
            doc_area: dto.doc_area,
            vintage: dto.vintage,
            grape_variety: dto.grape_variety,
            brix_at_harvest: dto.brix_at_harvest,
            ph_at_harvest: dto.ph_at_harvest,
            acidity: dto.acidity,
            yield_tons: dto.yield_tons,
            quality_grade: dto.quality_grade,
            slope_percent: dto.slope_percent,
            altitude_m: dto.altitude_m,
            is_organic: dto.is_organic,
            certification_body: dto.certification_body,
            certification_number: dto.certification_number,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateVineyardDto {
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<QualityGrade>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

impl From<UpdateVineyardDto> for DomainUpdateVineyardDto {
    fn from(dto: UpdateVineyardDto) -> Self {
        Self {
            doc_area: dto.doc_area,
            vintage: dto.vintage,
            grape_variety: dto.grape_variety,
            brix_at_harvest: dto.brix_at_harvest,
            ph_at_harvest: dto.ph_at_harvest,
            acidity: dto.acidity,
            yield_tons: dto.yield_tons,
            quality_grade: dto.quality_grade,
            slope_percent: dto.slope_percent,
            altitude_m: dto.altitude_m,
            is_organic: dto.is_organic,
            certification_body: dto.certification_body,
            certification_number: dto.certification_number,
            kelter_delivery: None,
        }
    }
}

// =============================================================================
// Kelter Delivery DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedKelterDeliveryResponse {
    pub data: Vec<KelterDeliveryDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct KelterDeliveryDto {
    pub id: Uuid,
    pub vineyard_id: Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub lot_number: String,
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

impl From<agrocore_domain::entities::vineyard::KelterDelivery> for KelterDeliveryDto {
    fn from(k: KelterDelivery) -> Self {
        Self {
            id: k.id,
            vineyard_id: k.vineyard_id,
            delivery_date: k.delivery_date.to_rfc3339(),
            gross_weight_kg: k.gross_weight_kg,
            net_weight_kg: k.net_weight_kg,
            lot_number: k.lot_number,
            kelter_name: k.kelter_name,
            transport_company: k.transport_company,
            temperature_c: k.temperature_c,
            notes: k.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateKelterDeliveryDto {
    pub vineyard_id: Uuid,
    pub delivery_date: String,
    #[validate(range(min = 0.0))]
    pub gross_weight_kg: f64,
    #[validate(range(min = 0.0))]
    pub net_weight_kg: f64,
    #[validate(length(min = 1))]
    pub lot_number: String,
    #[validate(length(min = 1))]
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

impl From<CreateKelterDeliveryDto> for DomainCreateKelterDeliveryDto {
    fn from(dto: CreateKelterDeliveryDto) -> Self {
        Self {
            vineyard_id: dto.vineyard_id,
            delivery_date: chrono::DateTime::parse_from_rfc3339(&dto.delivery_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            gross_weight_kg: dto.gross_weight_kg,
            net_weight_kg: dto.net_weight_kg,
            lot_number: dto.lot_number,
            kelter_name: dto.kelter_name,
            transport_company: dto.transport_company,
            temperature_c: dto.temperature_c,
            notes: dto.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateKelterDeliveryDto {
    pub vineyard_id: Option<Uuid>,
    pub delivery_date: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub net_weight_kg: Option<f64>,
    pub lot_number: Option<String>,
    pub kelter_name: Option<String>,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

impl From<UpdateKelterDeliveryDto> for DomainUpdateKelterDeliveryDto {
    fn from(dto: UpdateKelterDeliveryDto) -> Self {
        Self {
            vineyard_id: dto.vineyard_id,
            delivery_date: dto.delivery_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            gross_weight_kg: dto.gross_weight_kg,
            net_weight_kg: dto.net_weight_kg,
            lot_number: dto.lot_number,
            kelter_name: dto.kelter_name,
            transport_company: dto.transport_company,
            temperature_c: dto.temperature_c,
            notes: dto.notes,
        }
    }
}

// =============================================================================
// Olive Grove DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedOliveGroveResponse {
    pub data: Vec<OliveGroveDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OliveGroveDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub label: String,
    pub variety: String,
    pub planting_year: Option<i32>,
    pub area_ha: f64,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<OliveGrove> for OliveGroveDto {
    fn from(o: OliveGrove) -> Self {
        Self {
            id: o.id,
            tenant_id: o.tenant_id.into(),
            site_id: o.site_id,
            label: o.label,
            variety: o.variety,
            planting_year: o.planting_year,
            area_ha: o.area_ha,
            tree_count: o.tree_count,
            spacing_m: o.spacing_m,
            irrigation_type: o.irrigation_type,
            is_organic: o.is_organic,
            certification_body: o.certification_body,
            certification_number: o.certification_number,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateOliveGroveDto {
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[validate(length(min = 1))]
    pub variety: String,
    pub planting_year: Option<i32>,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

impl From<CreateOliveGroveDto> for DomainCreateOliveGroveDto {
    fn from(dto: CreateOliveGroveDto) -> Self {
        Self {
            site_id: dto.site_id,
            label: dto.label,
            variety: dto.variety,
            planting_year: dto.planting_year,
            area_ha: dto.area_ha,
            tree_count: dto.tree_count,
            spacing_m: dto.spacing_m,
            irrigation_type: dto.irrigation_type,
            is_organic: dto.is_organic.unwrap_or(false),
            certification_body: dto.certification_body,
            certification_number: dto.certification_number,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateOliveGroveDto {
    pub label: Option<String>,
    pub variety: Option<String>,
    pub planting_year: Option<i32>,
    pub area_ha: Option<f64>,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

impl From<UpdateOliveGroveDto> for DomainUpdateOliveGroveDto {
    fn from(dto: UpdateOliveGroveDto) -> Self {
        Self {
            label: dto.label,
            variety: dto.variety,
            planting_year: dto.planting_year,
            area_ha: dto.area_ha,
            tree_count: dto.tree_count,
            spacing_m: dto.spacing_m,
            irrigation_type: dto.irrigation_type,
            is_organic: dto.is_organic,
            certification_body: dto.certification_body,
            certification_number: dto.certification_number,
        }
    }
}

// =============================================================================
// Olive Oil Record DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedOliveOilRecordResponse {
    pub data: Vec<OliveOilRecordDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OliveOilRecordDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub olive_grove_id: Uuid,
    pub harvest_date: String,
    pub quantity_kg: f64,
    pub oil_yield_kg: f64,
    pub oil_yield_percent: f64,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<OliveOilRecord> for OliveOilRecordDto {
    fn from(o: OliveOilRecord) -> Self {
        Self {
            id: o.id,
            tenant_id: o.tenant_id.into(),
            olive_grove_id: o.grove_id,
            harvest_date: o.harvest_date.to_rfc3339(),
            quantity_kg: o.quantity_kg.unwrap_or(0.0),
            oil_yield_kg: o.oil_yield_kg.unwrap_or(0.0),
            oil_yield_percent: o.oil_yield_percent.unwrap_or(0.0),
            acidity_percent: o.acidity_pct,
            peroxide_value: o.peroxide_value,
            k232: o.k232,
            k270: o.k270,
            quality_grade: o.quality_grade,
            notes: o.notes,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateOliveOilRecordDto {
    pub olive_grove_id: Uuid,
    pub harvest_date: String,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub oil_yield_kg: f64,
    #[validate(range(min = 0.0, max = 100.0))]
    pub oil_yield_percent: f64,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}

impl From<CreateOliveOilRecordDto> for DomainCreateOliveOilRecordDto {
    fn from(dto: CreateOliveOilRecordDto) -> Self {
        Self {
            grove_id: dto.olive_grove_id,
            harvest_year: chrono::DateTime::parse_from_rfc3339(&dto.harvest_date)
                .map(|dt| dt.with_timezone(&chrono::Utc).year())
                .unwrap_or_else(|_| chrono::Utc::now().year()),
            oil_grade: dto.quality_grade.as_ref().and_then(|q| q.parse().ok()).unwrap_or(OilGrade::ExtraVirgin),
            acidity_pct: dto.acidity_percent,
            peroxide_value: dto.peroxide_value,
            sensory_score: None,
            liters_produced: None,
            mill_name: None,
            lot_number: None,
            harvest_date: chrono::DateTime::parse_from_rfc3339(&dto.harvest_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            quantity_kg: Some(dto.quantity_kg),
            oil_yield_kg: Some(dto.oil_yield_kg),
            oil_yield_percent: Some(dto.oil_yield_percent),
            k232: dto.k232,
            k270: dto.k270,
            quality_grade: dto.quality_grade,
            notes: dto.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateOliveOilRecordDto {
    pub olive_grove_id: Option<Uuid>,
    pub harvest_date: Option<String>,
    pub quantity_kg: Option<f64>,
    pub oil_yield_kg: Option<f64>,
    pub oil_yield_percent: Option<f64>,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}

impl From<UpdateOliveOilRecordDto> for DomainUpdateOliveOilRecordDto {
    fn from(dto: UpdateOliveOilRecordDto) -> Self {
        Self {
            grove_id: dto.olive_grove_id,
            harvest_year: dto.harvest_date.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc).year())
                    .ok()
            }),
            oil_grade: dto.quality_grade.as_ref().and_then(|q| q.parse().ok()),
            acidity_pct: dto.acidity_percent,
            peroxide_value: dto.peroxide_value,
            sensory_score: None,
            liters_produced: None,
            mill_name: None,
            lot_number: None,
            harvest_date: dto.harvest_date.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            quantity_kg: dto.quantity_kg,
            oil_yield_kg: dto.oil_yield_kg,
            oil_yield_percent: dto.oil_yield_percent,
            k232: dto.k232,
            k270: dto.k270,
            quality_grade: dto.quality_grade,
            notes: dto.notes,
        }
    }
}
