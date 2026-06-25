use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Worker {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: Uuid,
    pub contract_type: ContractType,
    pub language: Option<String>,
    pub skills: Vec<String>,
    pub certifications: Vec<Certification>,
    pub emergency_contact: Option<String>,
    pub nationality: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum ContractType {
    Permanent,
    Temporary,
    Seasonal,
    Freelance,
    Apprenticeship,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Certification {
    pub id: Uuid,
    pub name: String,
    pub issued_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub certificate_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WorkLog {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub worker_id: Uuid,
    pub date: DateTime<Utc>,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub rest_period_hours: f64,
    pub task_description: String,
    pub site_id: Option<Uuid>,
    pub is_night_shift: bool,
    pub breaks_taken: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWorkerDto {
    pub user_id: Uuid,
    pub contract_type: ContractType,
    pub language: Option<String>,
    pub skills: Option<Vec<String>>,
    pub emergency_contact: Option<String>,
    pub nationality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWorkLogDto {
    pub worker_id: Uuid,
    pub date: DateTime<Utc>,
    #[validate(range(min = 0.0, max = 24.0))]
    pub hours_worked: f64,
    #[validate(range(min = 0.0))]
    pub overtime_hours: f64,
    pub rest_period_hours: f64,
    #[validate(length(min = 1))]
    pub task_description: String,
    pub site_id: Option<Uuid>,
    pub is_night_shift: bool,
    pub breaks_taken: u32,
}
