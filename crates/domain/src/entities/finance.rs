use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::user::UserRole;
use crate::repositories::VisibilityAwareEntity;


#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct PACApplication {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub year: i32,
    pub application_number: String,
    #[sqlx(json)]
    pub status: PACStatus,
    pub total_eligible_area: f64,
    pub submitted_at: Option<DateTime<Utc>>,
    #[sqlx(json)]
    pub eco_schemes: Vec<EcoSchemeParticipation>,
    #[sqlx(json)]
    pub documents_urls: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for PACApplication {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum PACStatus {
    Draft,
    Submitted,
    InReview,
    Approved,
    Rejected,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EcoSchemeParticipation {
    pub scheme_code: String,
    pub label: String,
    pub area_ha: f64,
    pub estimated_subsidy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct CostCenter {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub label: String,
    pub code: String,
    #[sqlx(json)]
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
    pub is_active: bool,
}

impl VisibilityAwareEntity for CostCenter {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum CostCenterType {
    Site,
    Crop,
    Activity,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct FinancialRecord {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub cost_center_id: Uuid,
    pub date: DateTime<Utc>,
    pub amount: f64,
    pub currency: String,
    #[sqlx(json)]
    pub record_type: FinancialRecordType,
    pub category: String,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for FinancialRecord {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum FinancialRecordType {
    Expense,
    Income,
    Subsidy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateCostCenterDto {
    pub label: String,
    pub code: String,
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCostCenterDto {
    pub label: Option<String>,
    pub code: Option<String>,
    pub cost_center_type: Option<CostCenterType>,
    pub reference_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateFinancialRecordDto {
    pub cost_center_id: Uuid,
    pub date: DateTime<Utc>,
    pub amount: f64,
    pub currency: String,
    pub record_type: FinancialRecordType,
    pub category: String,
    pub description: String,
    pub reference_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateFinancialRecordDto {
    pub cost_center_id: Option<Uuid>,
    pub date: Option<DateTime<Utc>>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub record_type: Option<FinancialRecordType>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub reference_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePACApplicationDto {
    pub year: i32,
    pub application_number: String,
    pub total_eligible_area: f64,
    pub eco_schemes: Vec<EcoSchemeParticipation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePACApplicationDto {
    pub application_number: Option<String>,
    pub status: Option<PACStatus>,
    pub total_eligible_area: Option<f64>,
    pub eco_schemes: Option<Vec<EcoSchemeParticipation>>,
    pub documents_urls: Option<Vec<String>>,
}
