use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use mongodb::bson::{doc, Document};

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
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// VISIBILITY SECURITY: TaskData Entity implements VisibilityAwareEntity
// =============================================================================
impl VisibilityAwareEntity for TaskData {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(user_id: Uuid, roles: &[UserRole]) -> Document {
        // Worker sieht nur Aufgaben, die ihm direkt zugewiesen sind (worker_id match)
        if roles.contains(&UserRole::Admin) || roles.contains(&UserRole::Manager) {
            doc! {} // Alle Tasks im Tenant sichtbar
        } else if roles.contains(&UserRole::Worker) {
            // Worker filtert nach worker_id
            doc! { "worker_id": user_id.to_string() }
        } else {
            // Viewer hat keinen Zugriff auf Tasks - zurückgeben leeres Set
            doc! { "worker_id": "never-match-visibility" }
        }
    }
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
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}