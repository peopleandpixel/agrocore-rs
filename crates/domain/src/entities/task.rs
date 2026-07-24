use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
// use crate::entities::user::UserRole;
use crate::repositories::VisibilityAwareEntity;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
pub struct TaskData {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub order_id: Uuid,
    pub worker_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1))]
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub resume_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    #[sqlx(json)]
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    #[sqlx(json)]
    pub gps_track: Option<Vec<GpsPoint>>,
    #[sqlx(json)]
    pub photo_urls: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// VISIBILITY SECURITY: TaskData Entity implements VisibilityAwareEntity
// =============================================================================
impl VisibilityAwareEntity for TaskData {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct MaterialUsage {
    pub article_id: Uuid,
    pub label: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GpsPoint {
    pub lng: f64,
    pub lat: f64,
    pub timestamp: DateTime<Utc>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateTaskDataDto {
    pub order_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1))]
    pub description: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub resume_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default, ToSchema)]
pub struct UpdateTaskDataDto {
    pub order_id: Option<Uuid>,
    pub site_id: Option<Uuid>,
    pub description: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub resume_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}
