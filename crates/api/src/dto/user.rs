//! User DTOs

use agrocore_domain::entities::user::{
    CreateUserDto as DomainCreateUserDto, UpdateUserDto as DomainUpdateUserDto, User, UserRole,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUserResponse {
    pub data: Vec<UserDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            tenant_id: u.tenant_id.into(),
            firstname: u.firstname,
            lastname: u.lastname,
            email: u.email,
            roles: u.roles,
            is_active: u.is_active,
            internal_cost_per_hour: u.internal_cost_per_hour,
            external_cost_per_hour: u.external_cost_per_hour,
            color: u.color,
            language: u.language,
            last_login: u.last_login.map(|d| d.to_rfc3339()),
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 1, max = 100))]
    pub firstname: String,
    #[validate(length(min = 1, max = 100))]
    pub lastname: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub roles: Option<Vec<UserRole>>,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub language: Option<String>,
}

impl From<CreateUserDto> for DomainCreateUserDto {
    fn from(dto: CreateUserDto) -> Self {
        Self {
            firstname: dto.firstname,
            lastname: dto.lastname,
            email: dto.email,
            password: dto.password, // Will be hashed in repository
            roles: Some(dto.roles.unwrap_or_default()),
            internal_cost_per_hour: dto.internal_cost_per_hour,
            external_cost_per_hour: dto.external_cost_per_hour,
            language: dto.language,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateUserDto {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub password: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub is_active: Option<bool>,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub assigned_site_ids: Option<Vec<Uuid>>,
}

impl From<UpdateUserDto> for DomainUpdateUserDto {
    fn from(dto: UpdateUserDto) -> Self {
        Self {
            firstname: dto.firstname,
            lastname: dto.lastname,
            email: dto.email,
            password: dto.password,
            roles: dto.roles,
            is_active: dto.is_active,
            internal_cost_per_hour: dto.internal_cost_per_hour,
            external_cost_per_hour: dto.external_cost_per_hour,
            color: dto.color,
            language: dto.language,
            assigned_site_ids: dto.assigned_site_ids,
        }
    }
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
    pub roles: Vec<UserRole>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
