use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: Uuid,
    #[sqlx(json)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
pub struct ComplianceChecklist {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    #[sqlx(json)]
    pub checklist_type: ChecklistType,
    #[sqlx(json)]
    pub status: ComplianceStatus,
    #[sqlx(json)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub evidence_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateComplianceChecklistDto {
    pub site_id: Uuid,
    pub checklist_type: ChecklistType,
    pub items: Vec<ChecklistItem>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateComplianceChecklistDto {
    pub status: Option<ComplianceStatus>,
    pub items: Option<Vec<ChecklistItem>>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAuditLogDto {
    pub tenant_id: TenantId,
    pub user_id: Uuid,
    pub action: AuditAction,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
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
    pub updated_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default, ToSchema)]
pub struct UpdateFertilizerRecordDto {
    pub site_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub nutrient_n: Option<f64>,
    pub nutrient_p: Option<f64>,
    pub nutrient_k: Option<f64>,
    pub quantity_kg: Option<f64>,
    pub area_ha: Option<f64>,
    pub application_date: Option<DateTime<Utc>>,
}
