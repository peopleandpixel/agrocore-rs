use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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
    pub duration_minutes: Option<u32>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MaterialUsage {
    pub article_id: Uuid,
    pub label: String,
    pub quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsPoint {
    pub lng: f64,
    pub lat: f64,
    pub timestamp: DateTime<Utc>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTaskDataDto {
    pub order_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1))]
    pub description: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<u32>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}
