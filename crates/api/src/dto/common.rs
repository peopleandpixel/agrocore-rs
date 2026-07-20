//! Common DTOs shared across modules

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PaginatedResponseDto<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponseDto {
    pub token: String,
    pub refresh_token: Option<String>,
    pub token_expires_in: i64,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub roles: Vec<agrocore_domain::entities::user::UserRole>,
}
