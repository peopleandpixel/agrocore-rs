use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::repositories::VisibilityAwareEntity;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct MaintenanceInterval {
    pub label: String,
    pub interval_hours: Option<f64>,
    pub interval_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct Equipment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub label: String,
    pub code: Option<String>,
    #[sqlx(json)]
    pub equipment_type: EquipmentType,
    pub in_usage: bool,
    #[sqlx(json)]
    pub maintenance_intervals: Option<Vec<MaintenanceInterval>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub last_maintenance_hours: Option<f64>,
    pub fuel_capacity_liters: Option<f64>,
    pub fuel_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum EquipmentType {
    Tractor,
    Sprayer,
    Harvester,
    Mulcher,
    Plow,
    Trailer,
    Tool,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEquipmentDto {
    #[validate(length(min = 1))]
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
    pub maintenance_intervals: Option<Vec<MaintenanceInterval>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateEquipmentDto {
    pub label: Option<String>,
    pub code: Option<String>,
    pub equipment_type: Option<EquipmentType>,
    pub in_usage: Option<bool>,
    pub maintenance_intervals: Option<Vec<MaintenanceInterval>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub last_maintenance_hours: Option<f64>,
}

impl Equipment {
    pub const PROP_TANK_CAPACITY: &'static str = "tank_capacity";
    pub const PROP_WORKING_WIDTH: &'static str = "working_width";
    pub const PROP_FUEL_TYPE: &'static str = "fuel_type";
}

impl VisibilityAwareEntity for Equipment {}

/// Maintenance log entry — one row per maintenance action.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MaintenanceLogDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: TenantId,
    pub hours: Option<f64>,
    pub note: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub parts_cost: Option<f64>,
    pub labor_hours: Option<f64>,
    pub downtime_hours: Option<f64>,
}

/// Aggregated maintenance cost summary for an equipment.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MaintenanceCostSummaryDto {
    pub equipment_id: Uuid,
    pub total_parts_cost: f64,
    pub total_labor_hours: f64,
    pub total_downtime_hours: f64,
    pub total_maintenance_count: i64,
    pub total_cost: f64,
}

/// Fuel consumption entry — tracks liters, cost, operation context.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct FuelConsumptionDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: TenantId,
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

/// Equipment usage log entry — tracks who used equipment, when, and for how long.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLogDto {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub tenant_id: TenantId,
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
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageSummaryDto {
    pub equipment_id: Uuid,
    pub total_hours: f64,
    pub total_sessions: i64,
    pub avg_hours_per_session: f64,
    pub first_used: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
}
