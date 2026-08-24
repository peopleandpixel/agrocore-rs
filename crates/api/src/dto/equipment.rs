//! Equipment DTOs

use agrocore_domain::entities::equipment::EquipmentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MaintenanceIntervalDto {
    pub label: String,
    pub interval_hours: Option<f64>,
    pub interval_days: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedEquipmentResponse {
    pub data: Vec<EquipmentDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EquipmentDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
    pub in_usage: bool,
    pub maintenance_intervals: Option<Vec<MaintenanceIntervalDto>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub last_maintenance_hours: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::equipment::Equipment> for EquipmentDto {
    fn from(e: agrocore_domain::entities::equipment::Equipment) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id.into(),
            label: e.label,
            code: e.code,
            equipment_type: e.equipment_type,
            in_usage: e.in_usage,
            maintenance_intervals: e.maintenance_intervals.map(|intervals| {
                intervals
                    .into_iter()
                    .map(|i| MaintenanceIntervalDto {
                        label: i.label,
                        interval_hours: i.interval_hours,
                        interval_days: i.interval_days,
                    })
                    .collect()
            }),
            next_maintenance_date: e.next_maintenance_date,
            last_maintenance_hours: e.last_maintenance_hours,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateEquipmentDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
}

impl From<CreateEquipmentDto> for agrocore_domain::entities::equipment::CreateEquipmentDto {
    fn from(dto: CreateEquipmentDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            equipment_type: dto.equipment_type,
            maintenance_intervals: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateEquipmentDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub code: Option<String>,
    pub equipment_type: Option<EquipmentType>,
    pub in_usage: Option<bool>,
}

/// Maintenance record for equipment.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct MaintenanceRecordDto {
    #[validate(range(min = 0.0))]
    pub hours: f64,
    pub note: Option<String>,
}

/// Query parameters for filtering equipment list.
#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct EquipmentFilterDto {
    /// Full-text search on label and code
    pub search: Option<String>,
    /// Filter by equipment type (e.g., "tractor")
    pub equipment_type: Option<String>,
    /// Filter by in_usage status
    pub in_usage: Option<bool>,
    /// Only show equipment needing maintenance
    pub needs_maintenance: Option<bool>,
}

impl From<UpdateEquipmentDto> for agrocore_domain::entities::equipment::UpdateEquipmentDto {
    fn from(dto: UpdateEquipmentDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            equipment_type: dto.equipment_type,
            in_usage: dto.in_usage,
            maintenance_intervals: None,
            next_maintenance_date: None,
            last_maintenance_hours: None,
        }
    }
}
