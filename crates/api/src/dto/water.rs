//! Water DTOs

use agrocore_domain::entities::water::{
    CreateWaterQuotaDto as DomainCreateWaterQuotaDto,
    CreateWaterSourceDto as DomainCreateWaterSourceDto,
    CreateWaterUsageDto as DomainCreateWaterUsageDto, IrrigationMethod,
    UpdateWaterQuotaDto as DomainUpdateWaterQuotaDto,
    UpdateWaterSourceDto as DomainUpdateWaterSourceDto,
    UpdateWaterUsageDto as DomainUpdateWaterUsageDto, WaterQuota, WaterSource, WaterSourceType,
    WaterUsage,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWaterSourceResponse {
    pub data: Vec<WaterSourceDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaterSourceDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub name: String,
    pub source_type: String,
    pub capacity_m3: Option<f64>,
    pub current_usage_m3: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::water::WaterSource> for WaterSourceDto {
    fn from(w: WaterSource) -> Self {
        Self {
            id: w.id,
            tenant_id: w.tenant_id.into(),
            site_id: w.site_id,
            name: w.name,
            source_type: w.source_type.to_string(),
            capacity_m3: w.capacity_m3,
            current_usage_m3: w.current_level_m3.unwrap_or(0.0),
            is_active: w.is_active,
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWaterSourceDto {
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub source_type: String,
    pub capacity_m3: Option<f64>,
}

impl From<CreateWaterSourceDto> for DomainCreateWaterSourceDto {
    fn from(dto: CreateWaterSourceDto) -> Self {
        Self {
            site_id: dto.site_id,
            name: dto.name,
            source_type: dto.source_type.parse().unwrap_or(WaterSourceType::Well),
            capacity_m3: dto.capacity_m3,
            license_number: None,
            license_expiry: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateWaterSourceDto {
    pub name: Option<String>,
    pub source_type: Option<String>,
    pub capacity_m3: Option<f64>,
    pub is_active: Option<bool>,
}

impl From<UpdateWaterSourceDto> for DomainUpdateWaterSourceDto {
    fn from(dto: UpdateWaterSourceDto) -> Self {
        Self {
            name: dto.name,
            source_type: dto
                .source_type
                .map(|s| s.parse().unwrap_or(WaterSourceType::Well)),
            capacity_m3: dto.capacity_m3,
            current_level_m3: None,
            license_number: None,
            license_expiry: None,
            is_active: dto.is_active,
        }
    }
}

// Water Usage DTOs
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWaterUsageResponse {
    pub data: Vec<WaterUsageDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaterUsageDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub source_id: Uuid,
    pub usage_date: String,
    pub volume_m3: f64,
    pub irrigation_method: String,
    pub efficiency_pct: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::water::WaterUsage> for WaterUsageDto {
    fn from(w: WaterUsage) -> Self {
        Self {
            id: w.id,
            tenant_id: w.tenant_id.into(),
            site_id: w.site_id,
            source_id: w.source_id,
            usage_date: w.usage_date.to_rfc3339(),
            volume_m3: w.volume_m3,
            irrigation_method: w.irrigation_method.to_string(),
            efficiency_pct: w.efficiency_pct,
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWaterUsageDto {
    pub site_id: Uuid,
    pub source_id: Uuid,
    pub usage_date: String,
    #[validate(range(min = 0.0))]
    pub volume_m3: f64,
    pub irrigation_method: String,
    pub efficiency_pct: Option<f64>,
}

impl From<CreateWaterUsageDto> for DomainCreateWaterUsageDto {
    fn from(dto: CreateWaterUsageDto) -> Self {
        Self {
            site_id: dto.site_id,
            source_id: dto.source_id,
            usage_date: chrono::DateTime::parse_from_rfc3339(&dto.usage_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            volume_m3: dto.volume_m3,
            irrigation_method: dto
                .irrigation_method
                .parse()
                .unwrap_or(IrrigationMethod::Drip),
            efficiency_pct: dto.efficiency_pct,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateWaterUsageDto {
    pub site_id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub usage_date: Option<String>,
    pub volume_m3: Option<f64>,
    pub irrigation_method: Option<String>,
    pub efficiency_pct: Option<f64>,
}

impl From<UpdateWaterUsageDto> for DomainUpdateWaterUsageDto {
    fn from(dto: UpdateWaterUsageDto) -> Self {
        Self {
            site_id: dto.site_id,
            source_id: dto.source_id,
            usage_date: dto.usage_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            volume_m3: dto.volume_m3,
            irrigation_method: dto
                .irrigation_method
                .map(|p| p.parse().unwrap_or(IrrigationMethod::Drip)),
            efficiency_pct: dto.efficiency_pct,
        }
    }
}

// Water Quota DTOs
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWaterQuotaResponse {
    pub data: Vec<WaterQuotaDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaterQuotaDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub source_id: Uuid,
    pub site_id: Uuid,
    pub year: i32,
    pub allocated_m3: f64,
    pub used_m3: f64,
    pub remaining_m3: f64,
    pub comunidad_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::water::WaterQuota> for WaterQuotaDto {
    fn from(w: WaterQuota) -> Self {
        Self {
            id: w.id,
            tenant_id: w.tenant_id.into(),
            source_id: w.source_id,
            site_id: w.site_id,
            year: w.year,
            allocated_m3: w.allocated_m3,
            used_m3: w.used_m3,
            remaining_m3: w.remaining_m3,
            comunidad_id: w.comunidad_id,
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWaterQuotaDto {
    pub source_id: Uuid,
    pub site_id: Uuid,
    pub year: i32,
    #[validate(range(min = 0.0))]
    pub allocated_m3: f64,
    pub comunidad_id: Option<Uuid>,
}

impl From<CreateWaterQuotaDto> for DomainCreateWaterQuotaDto {
    fn from(dto: CreateWaterQuotaDto) -> Self {
        Self {
            source_id: dto.source_id,
            site_id: dto.site_id,
            year: dto.year,
            allocated_m3: dto.allocated_m3,
            comunidad_id: dto.comunidad_id,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateWaterQuotaDto {
    pub allocated_m3: Option<f64>,
    pub used_m3: Option<f64>,
    pub comunidad_id: Option<Uuid>,
}

impl From<UpdateWaterQuotaDto> for DomainUpdateWaterQuotaDto {
    fn from(dto: UpdateWaterQuotaDto) -> Self {
        Self {
            allocated_m3: dto.allocated_m3,
            used_m3: dto.used_m3,
            comunidad_id: dto.comunidad_id,
        }
    }
}
