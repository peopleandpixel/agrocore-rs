use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WaterSource {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub source_type: WaterSourceType,
    pub name: String,
    pub capacity_m3: Option<f64>,
    pub current_level_m3: Option<f64>,
    pub license_number: Option<String>,
    pub license_expiry: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum WaterSourceType {
    Well,
    Reservoir,
    River,
    Canal,
    ComunidadDeRegantes,
    RainwaterHarvesting,
    Desalination,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WaterUsage {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub source_id: Uuid,
    pub usage_date: DateTime<Utc>,
    pub volume_m3: f64,
    pub irrigation_method: IrrigationMethod,
    pub efficiency_pct: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum IrrigationMethod {
    Drip,
    Sprinkler,
    Flood,
    Pivot,
    MicroSprinkler,
    Subsurface,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WaterQuota {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub source_id: Uuid,
    pub year: i32,
    pub allocated_m3: f64,
    pub used_m3: f64,
    pub comunidad_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWaterSourceDto {
    pub site_id: Uuid,
    pub source_type: WaterSourceType,
    #[validate(length(min = 1))]
    pub name: String,
    pub capacity_m3: Option<f64>,
    pub license_number: Option<String>,
    pub license_expiry: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWaterUsageDto {
    pub source_id: Uuid,
    pub site_id: Uuid,
    pub usage_date: DateTime<Utc>,
    #[validate(range(min = 0.0))]
    pub volume_m3: f64,
    pub irrigation_method: IrrigationMethod,
    pub efficiency_pct: Option<f64>,
}
