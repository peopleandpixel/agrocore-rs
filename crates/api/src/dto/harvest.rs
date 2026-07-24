//! Harvest DTOs

use agrocore_domain::entities::harvest::{
    ColdChainLog, CreateColdChainLogDto as DomainCreateColdChainLogDto,
    CreateHarvestDeliveryDto as DomainCreateHarvestDeliveryDto,
    CreateHarvestLotDto as DomainCreateHarvestLotDto,
    CreateHarvestSeasonDto as DomainCreateHarvestSeasonDto, HarvestDelivery, HarvestLot,
    HarvestSeason, LotStatus, UpdateColdChainLogDto as DomainUpdateColdChainLogDto,
    UpdateHarvestDeliveryDto as DomainUpdateHarvestDeliveryDto,
    UpdateHarvestLotDto as DomainUpdateHarvestLotDto,
    UpdateHarvestSeasonDto as DomainUpdateHarvestSeasonDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Harvest Season DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedHarvestSeasonResponse {
    pub data: Vec<HarvestSeasonDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HarvestSeasonDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub year: i32,
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestSeason> for HarvestSeasonDto {
    fn from(h: HarvestSeason) -> Self {
        Self {
            id: h.id,
            tenant_id: h.tenant_id.into(),
            year: h.year,
            label: h.label,
            start_date: h.start_date.to_rfc3339(),
            end_date: h.end_date.map(|d| d.to_rfc3339()),
            is_active: h.is_active,
            created_at: h.created_at.to_rfc3339(),
            updated_at: h.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestSeasonDto {
    #[validate(range(min = 2000, max = 2100))]
    pub year: i32,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
}

impl From<CreateHarvestSeasonDto> for DomainCreateHarvestSeasonDto {
    fn from(dto: CreateHarvestSeasonDto) -> Self {
        Self {
            year: dto.year,
            label: dto.label,
            start_date: chrono::DateTime::parse_from_rfc3339(&dto.start_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            end_date: dto.end_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestSeasonDto {
    pub label: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: Option<bool>,
}

impl From<UpdateHarvestSeasonDto> for DomainUpdateHarvestSeasonDto {
    fn from(dto: UpdateHarvestSeasonDto) -> Self {
        Self {
            label: dto.label,
            start_date: dto.start_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            end_date: dto.end_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            is_active: dto.is_active,
        }
    }
}

// =============================================================================
// Harvest Lot DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedHarvestLotResponse {
    pub data: Vec<HarvestLotDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HarvestLotDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub season_id: Uuid,
    pub lot_number: String,
    pub crop_type: String,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
    pub total_weight_kg: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestLot> for HarvestLotDto {
    fn from(l: HarvestLot) -> Self {
        Self {
            id: l.id,
            tenant_id: l.tenant_id.into(),
            season_id: l.season_id,
            lot_number: l.lot_number,
            crop_type: l.crop_type,
            variety: l.variety,
            quality_target: l.quality_target,
            total_weight_kg: l.total_weight_kg,
            status: l.status.to_string(),
            created_at: l.created_at.to_rfc3339(),
            updated_at: l.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestLotDto {
    pub season_id: Uuid,
    #[validate(length(min = 1))]
    pub lot_number: String,
    pub site_ids: Vec<Uuid>,
    pub crop_type: String,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
}

impl From<CreateHarvestLotDto> for DomainCreateHarvestLotDto {
    fn from(dto: CreateHarvestLotDto) -> Self {
        Self {
            season_id: dto.season_id,
            lot_number: dto.lot_number,
            site_ids: dto.site_ids,
            crop_type: dto.crop_type,
            variety: dto.variety,
            quality_target: dto.quality_target,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestLotDto {
    pub lot_number: Option<String>,
    pub site_ids: Option<Vec<Uuid>>,
    pub crop_type: Option<String>,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
    pub total_weight_kg: Option<f64>,
    pub status: Option<LotStatus>,
}

impl From<UpdateHarvestLotDto> for DomainUpdateHarvestLotDto {
    fn from(dto: UpdateHarvestLotDto) -> Self {
        Self {
            lot_number: dto.lot_number,
            site_ids: dto.site_ids,
            crop_type: dto.crop_type,
            variety: dto.variety,
            quality_target: dto.quality_target,
            total_weight_kg: dto.total_weight_kg,
            status: dto.status,
        }
    }
}

// =============================================================================
// Harvest Delivery DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedHarvestDeliveryResponse {
    pub data: Vec<HarvestDeliveryDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HarvestDeliveryDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub lot_id: Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestDelivery> for HarvestDeliveryDto {
    fn from(h: HarvestDelivery) -> Self {
        Self {
            id: h.id,
            tenant_id: h.tenant_id.into(),
            lot_id: h.lot_id,
            delivery_date: h.delivery_date.to_rfc3339(),
            gross_weight_kg: h.gross_weight_kg,
            net_weight_kg: h.net_weight_kg,
            tare_weight_kg: h.tare_weight_kg,
            carrier_name: h.carrier_name,
            vehicle_id: h.vehicle_id,
            quality_notes: h.quality_notes,
            temperature_at_delivery: h.temperature_at_delivery,
            created_at: h.created_at.to_rfc3339(),
            updated_at: h.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestDeliveryDto {
    pub lot_id: Uuid,
    pub delivery_date: String,
    #[validate(range(min = 0.0))]
    pub gross_weight_kg: f64,
    #[validate(range(min = 0.0))]
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

impl From<CreateHarvestDeliveryDto> for DomainCreateHarvestDeliveryDto {
    fn from(dto: CreateHarvestDeliveryDto) -> Self {
        Self {
            lot_id: dto.lot_id,
            delivery_date: chrono::DateTime::parse_from_rfc3339(&dto.delivery_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            gross_weight_kg: dto.gross_weight_kg,
            tare_weight_kg: dto.tare_weight_kg,
            carrier_name: dto.carrier_name,
            vehicle_id: dto.vehicle_id,
            quality_notes: dto.quality_notes,
            temperature_at_delivery: dto.temperature_at_delivery,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestDeliveryDto {
    pub delivery_date: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub tare_weight_kg: Option<f64>,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

impl From<UpdateHarvestDeliveryDto> for DomainUpdateHarvestDeliveryDto {
    fn from(dto: UpdateHarvestDeliveryDto) -> Self {
        Self {
            lot_id: None,
            delivery_date: dto.delivery_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            gross_weight_kg: dto.gross_weight_kg,
            net_weight_kg: None,
            tare_weight_kg: dto.tare_weight_kg,
            carrier_name: dto.carrier_name,
            vehicle_id: dto.vehicle_id,
            quality_notes: dto.quality_notes,
            temperature_at_delivery: dto.temperature_at_delivery,
        }
    }
}

// =============================================================================
// Cold Chain Log DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedColdChainLogResponse {
    pub data: Vec<ColdChainLogDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ColdChainLogDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub lot_id: Uuid,
    pub sensor_id: String,
    pub recorded_at: String,
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
    pub created_at: String,
}

impl From<ColdChainLog> for ColdChainLogDto {
    fn from(c: ColdChainLog) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id.into(),
            lot_id: c.lot_id,
            sensor_id: c.sensor_id,
            recorded_at: c.recorded_at.to_rfc3339(),
            temperature_c: c.temperature_c,
            humidity_pct: c.humidity_pct,
            location: c.location,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateColdChainLogDto {
    pub lot_id: Uuid,
    pub sensor_id: String,
    pub recorded_at: String,
    #[validate(range(min = -50.0, max = 100.0))]
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}

impl From<CreateColdChainLogDto> for DomainCreateColdChainLogDto {
    fn from(dto: CreateColdChainLogDto) -> Self {
        Self {
            lot_id: dto.lot_id,
            sensor_id: dto.sensor_id,
            recorded_at: chrono::DateTime::parse_from_rfc3339(&dto.recorded_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            temperature_c: dto.temperature_c,
            humidity_pct: dto.humidity_pct,
            location: dto.location,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateColdChainLogDto {
    pub lot_id: Option<Uuid>,
    pub sensor_id: Option<String>,
    pub recorded_at: Option<String>,
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}

impl From<UpdateColdChainLogDto> for DomainUpdateColdChainLogDto {
    fn from(dto: UpdateColdChainLogDto) -> Self {
        Self {
            lot_id: dto.lot_id,
            sensor_id: dto.sensor_id,
            recorded_at: dto.recorded_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            temperature_c: dto.temperature_c,
            humidity_pct: dto.humidity_pct,
            location: dto.location,
        }
    }
}
