//! Common DTOs shared across modules

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[schema(as = PaginatedResponse<T>)]
pub struct PaginatedResponseDto<T>
where
    T: Serialize + ToSchema,
{
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
