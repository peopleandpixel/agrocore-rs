//! Harvest DTOs

use agrocore_domain::entities::harvest::{
    ColdChainLog, CreateColdChainLogDto as DomainCreateColdChainLogDto,
    CreateHarvestDeliveryDto as DomainCreateHarvestDeliveryDto,
    CreateHarvestLotDto as DomainCreateHarvestLotDto,
    CreateHarvestSeasonDto as DomainCreateHarvestSeasonDto, HarvestDelivery, HarvestLot,
    HarvestSeason, UpdateColdChainLogDto as DomainUpdateColdChainLogDto,
    UpdateHarvestDeliveryDto as DomainUpdateHarvestDeliveryDto,
    UpdateHarvestLotDto as DomainUpdateHarvestLotDto,
    UpdateHarvestSeasonDto as DomainUpdateHarvestSeasonDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub site_id: Uuid,
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
    pub total_planned_kg: f64,
    pub total_harvested_kg: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestSeason> for HarvestSeasonDto {
    fn from(h: HarvestSeason) -> Self {
        Self {
            id: h.id,
            tenant_id: h.tenant_id.into(),
            site_id: h.site_id,
            label: h.label,
            start_date: h.start_date.to_rfc3339(),
            end_date: h.end_date.map(|d| d.to_rfc3339()),
            status: h.status.to_string(),
            total_planned_kg: h.total_planned_kg,
            total_harvested_kg: h.total_harvested_kg,
            created_at: h.created_at.to_rfc3339(),
            updated_at: h.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestSeasonDto {
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
    #[validate(range(min = 0.0))]
    pub total_planned_kg: f64,
}

impl From<CreateHarvestSeasonDto> for DomainCreateHarvestSeasonDto {
    fn from(dto: CreateHarvestSeasonDto) -> Self {
        Self {
            site_id: dto.site_id,
            label: dto.label,
            start_date: chrono::DateTime::parse_from_rfc3339(&dto.start_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            end_date: dto.end_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            total_planned_kg: dto.total_planned_kg,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestSeasonDto {
    pub label: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
    pub total_planned_kg: Option<f64>,
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
            status: dto.status,
            total_planned_kg: dto.total_planned_kg,
        }
    }
}

// Harvest Lot DTOs
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
    pub harvest_season_id: Uuid,
    pub site_id: Uuid,
    pub label: String,
    pub variety: Option<String>,
    pub quantity_kg: f64,
    pub quality_grade: Option<String>,
    pub harvest_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestLot> for HarvestLotDto {
    fn from(l: HarvestLot) -> Self {
        Self {
            id: l.id,
            tenant_id: l.tenant_id.into(),
            harvest_season_id: l.harvest_season_id,
            site_id: l.site_id,
            label: l.label,
            variety: l.variety,
            quantity_kg: l.quantity_kg,
            quality_grade: l.quality_grade,
            harvest_date: l.harvest_date.to_rfc3339(),
            created_at: l.created_at.to_rfc3339(),
            updated_at: l.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestLotDto {
    pub harvest_season_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    pub quality_grade: Option<String>,
    pub harvest_date: String,
}

impl From<CreateHarvestLotDto> for DomainCreateHarvestLotDto {
    fn from(dto: CreateHarvestLotDto) -> Self {
        Self {
            harvest_season_id: dto.harvest_season_id,
            site_id: dto.site_id,
            label: dto.label,
            variety: dto.variety,
            quantity_kg: dto.quantity_kg,
            quality_grade: dto.quality_grade,
            harvest_date: chrono::DateTime::parse_from_rfc3339(&dto.harvest_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestLotDto {
    pub label: Option<String>,
    pub variety: Option<String>,
    pub quantity_kg: Option<f64>,
    pub quality_grade: Option<String>,
    pub harvest_date: Option<String>,
}

impl From<UpdateHarvestLotDto> for DomainUpdateHarvestLotDto {
    fn from(dto: UpdateHarvestLotDto) -> Self {
        Self {
            label: dto.label,
            variety: dto.variety,
            quantity_kg: dto.quantity_kg,
            quality_grade: dto.quality_grade,
            harvest_date: dto.harvest_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

// Harvest Delivery DTOs
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
    pub harvest_lot_id: Uuid,
    pub destination: String,
    pub quantity_kg: f64,
    pub delivery_date: String,
    pub vehicle: Option<String>,
    pub driver: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<HarvestDelivery> for HarvestDeliveryDto {
    fn from(h: HarvestDelivery) -> Self {
        Self {
            id: h.id,
            tenant_id: h.tenant_id.into(),
            harvest_lot_id: h.harvest_lot_id,
            destination: h.destination,
            quantity_kg: h.quantity_kg,
            delivery_date: h.delivery_date.to_rfc3339(),
            vehicle: h.vehicle,
            driver: h.driver,
            temperature_c: h.temperature_c,
            notes: h.notes,
            created_at: h.created_at.to_rfc3339(),
            updated_at: h.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateHarvestDeliveryDto {
    pub harvest_lot_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub destination: String,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    pub delivery_date: String,
    pub vehicle: Option<String>,
    pub driver: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

impl From<CreateHarvestDeliveryDto> for DomainCreateHarvestDeliveryDto {
    fn from(dto: CreateHarvestDeliveryDto) -> Self {
        Self {
            harvest_lot_id: dto.harvest_lot_id,
            destination: dto.destination,
            quantity_kg: dto.quantity_kg,
            delivery_date: chrono::DateTime::parse_from_rfc3339(&dto.delivery_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            vehicle: dto.vehicle,
            driver: dto.driver,
            temperature_c: dto.temperature_c,
            notes: dto.notes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateHarvestDeliveryDto {
    pub destination: Option<String>,
    pub quantity_kg: Option<f64>,
    pub delivery_date: Option<String>,
    pub vehicle: Option<String>,
    pub driver: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

impl From<UpdateHarvestDeliveryDto> for DomainUpdateHarvestDeliveryDto {
    fn from(dto: UpdateHarvestDeliveryDto) -> Self {
        Self {
            destination: dto.destination,
            quantity_kg: dto.quantity_kg,
            delivery_date: dto.delivery_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            vehicle: dto.vehicle,
            driver: dto.driver,
            temperature_c: dto.temperature_c,
            notes: dto.notes,
        }
    }
}

// Cold Chain Log DTOs
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
    pub timestamp: String,
    #[validate(range(min = -50.0, max = 100.0))]
    pub temperature_c: f64,
    pub humidity_percent: Option<f64>,
    pub location: Option<String>,
}

impl From<CreateColdChainLogDto> for DomainCreateColdChainLogDto {
    fn from(dto: CreateColdChainLogDto) -> Self {
        Self {
            lot_id: dto.lot_id,
            sensor_id: dto.location.clone().unwrap_or_default(),
            recorded_at: chrono::DateTime::parse_from_rfc3339(&dto.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            temperature_c: dto.temperature_c,
            humidity_pct: dto.humidity_percent,
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
