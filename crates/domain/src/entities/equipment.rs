use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use mongodb::bson::{doc, Document};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct MaintenanceInterval {
    pub label: String,
    pub interval_hours: Option<f64>,
    pub interval_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Equipment {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: EquipmentType,
    pub in_usage: bool,
    pub maintenance_intervals: Option<Vec<MaintenanceInterval>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub last_maintenance_hours: Option<f64>,
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

impl VisibilityAwareEntity for Equipment {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(_user_id: Uuid, _roles: &[crate::entities::user::UserRole]) -> Document {
        doc! {} // All visible for now
    }
}
