use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub token: String,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SiteDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub site_type: String,
    pub crop_type: String,
    pub variety: Option<String>,
    pub area: f64,
    pub bbch_stage: Option<String>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct OrderDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub order_type: String,
    pub status: String,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub language: Option<String>,
    pub color: Option<String>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskDataDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub order_id: Uuid,
    pub worker_id: Uuid,
    pub site_id: Uuid,
    pub description: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<u32>,
    pub area_covered: Option<f64>,
    pub observations: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
