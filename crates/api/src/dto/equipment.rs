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
    pub fuel_capacity_liters: Option<f64>,
    pub fuel_type: Option<String>,
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
            fuel_capacity_liters: e.fuel_capacity_liters,
            fuel_type: e.fuel_type,
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
    #[validate(range(min = 0.0))]
    pub parts_cost: Option<f64>,
    #[validate(range(min = 0.0))]
    pub labor_hours: Option<f64>,
    #[validate(range(min = 0.0))]
    pub downtime_hours: Option<f64>,
}

/// Maintenance log entry — one row per maintenance action.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MaintenanceLogDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: Uuid,
    pub hours: f64,
    pub note: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub parts_cost: f64,
    pub labor_hours: f64,
    pub downtime_hours: f64,
}

/// Aggregated maintenance cost summary.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MaintenanceCostSummaryDto {
    pub equipment_id: Uuid,
    pub total_parts_cost: f64,
    pub total_labor_hours: f64,
    pub total_downtime_hours: f64,
    pub total_maintenance_count: i64,
    pub total_cost: f64,
}

/// Fuel consumption entry — tracks liters, cost, operation context.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FuelConsumptionDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: Uuid,
    pub liters: f64,
    pub cost_per_liter: Option<f64>,
    pub total_cost: Option<f64>,
    pub operation_type: Option<String>,
    pub field_id: Option<Uuid>,
    pub hours_operated: Option<f64>,
    pub consumed_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to record a fuel consumption entry.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateFuelConsumptionRequest {
    #[validate(range(min = 0.0))]
    pub liters: f64,
    #[validate(range(min = 0.0))]
    pub cost_per_liter: Option<f64>,
    pub operation_type: Option<String>,
    pub field_id: Option<Uuid>,
    #[validate(range(min = 0.0))]
    pub hours_operated: Option<f64>,
    pub notes: Option<String>,
}

/// Equipment usage log entry — tracks who used equipment, when, and for how long.
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UsageLogDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: Uuid,
    pub worker_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub operation_type: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub hours_operated: f64,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aggregated usage summary for an equipment.
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UsageSummaryDto {
    pub equipment_id: Uuid,
    pub total_hours: f64,
    pub total_sessions: i64,
    pub avg_hours_per_session: f64,
    pub first_used: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
}

/// Request to record a usage log entry.
#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct CreateUsageLogRequest {
    pub worker_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    #[validate(length(max = 100))]
    pub operation_type: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    #[validate(range(min = 0.0))]
    pub hours_operated: Option<f64>,
    pub note: Option<String>,
}

/// Depreciation method for equipment asset amortization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
pub enum DepreciationMethod {
    #[default]
    StraightLine,
    DoubleDeclining,
}

/// Equipment depreciation summary — financial asset tracking.
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct EquipmentDepreciationDto {
    pub equipment_id: Uuid,
    pub tenant_id: Uuid,
    pub original_cost: f64,
    pub salvage_value: f64,
    pub purchase_date: Option<DateTime<Utc>>,
    pub depreciation_method: DepreciationMethod,
    pub useful_life_years: u32,
    pub accumulated_depreciation: f64,
    pub net_book_value: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Yearly depreciation schedule entry.
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct DepreciationScheduleEntry {
    pub year: i32,
    pub depreciation_amount: f64,
    pub accumulated_depreciation: f64,
    pub net_book_value: f64,
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
