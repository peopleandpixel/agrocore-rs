use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: Uuid,
    pub action: AuditAction,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    Viewed,
    Exported,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ComplianceChecklist {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub checklist_type: ChecklistType,
    pub status: ComplianceStatus,
    pub items: Vec<ChecklistItem>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ChecklistType {
    GAP,
    Organic,
    GlobalGAP,
    HACCP,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ComplianceStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub evidence_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAuditLogDto {
    pub user_id: Uuid,
    pub action: AuditAction,
    #[validate(length(min = 1, max = 100))]
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateComplianceChecklistDto {
    pub site_id: Uuid,
    pub checklist_type: ChecklistType,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FertilizerRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFertilizerRecordDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
}
