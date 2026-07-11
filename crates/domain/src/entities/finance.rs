use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::user::UserRole;
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use mongodb::bson::{doc, Document};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PACApplication {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub year: i32,
    pub application_number: String,
    pub status: PACStatus,
    pub total_eligible_area: f64,
    pub submitted_at: Option<DateTime<Utc>>,
    pub eco_schemes: Vec<EcoSchemeParticipation>,
    pub documents_urls: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for PACApplication {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(_user_id: Uuid, _roles: &[UserRole]) -> Document {
        // Aktuell sehen alle Rollen im Tenant die PAC-Daten (inkl. Worker).
        // Rollenspezifische Einschränkung folgt bei Bedarf.
        doc! {}
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CostCenter {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub label: String,
    pub code: String,
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
    pub is_active: bool,
}

impl VisibilityAwareEntity for CostCenter {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(_user_id: Uuid, _roles: &[UserRole]) -> Document {
        // Kostenstellen sind für alle Rollen im Tenant sichtbar.
        doc! {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum CostCenterType {
    Site,
    Crop,
    Activity,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct FinancialRecord {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub cost_center_id: Uuid,
    pub date: DateTime<Utc>,
    pub amount: f64,
    pub currency: String,
    pub record_type: FinancialRecordType,
    pub category: String,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for FinancialRecord {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(_user_id: Uuid, _roles: &[UserRole]) -> Document {
        // Finanzsätze sind aktuell für alle Rollen im Tenant sichtbar.
        // Rollenspezifische Einschränkung (z. B. Worker über Auftragsbezug) folgt bei Bedarf.
        doc! {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum FinancialRecordType {
    Expense,
    Income,
    Subsidy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePACApplicationDto {
    pub year: i32,
    pub application_number: String,
    pub total_eligible_area: f64,
    pub eco_schemes: Vec<EcoSchemeParticipation>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateCostCenterDto {
    pub label: String,
    pub code: String,
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
}