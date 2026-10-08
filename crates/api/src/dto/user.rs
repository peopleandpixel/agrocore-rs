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
    #[validate(length(min = 12, max = 128))]
    #[validate(custom(function = "validate_password_strength"))]
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
    /// Min 12 instead of the 8 used at creation: an update can otherwise
    /// downgrade an existing strong password without any strength check.
    #[validate(length(min = 12, max = 128))]
    pub password: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub is_active: Option<bool>,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub assigned_site_ids: Option<Vec<Uuid>>,
}

/// Fields a user may change on their own account.
///
/// Deliberately excludes `roles`, `is_active` and both cost fields: those are
/// administrative and require an admin role on `PUT /users/{id}`.
#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateOwnProfileDto {
    #[validate(length(max = 128))]
    pub firstname: Option<String>,
    #[validate(length(max = 128))]
    pub lastname: Option<String>,
    #[validate(length(min = 12, max = 128))]
    pub password: Option<String>,
    #[validate(length(max = 16))]
    pub language: Option<String>,
    #[validate(length(max = 32))]
    pub color: Option<String>,
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

// Password strength validation: min 12 chars, at least one uppercase, one lowercase, one digit, one special char
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_special = false;

    for ch in password.chars() {
        if ch.is_ascii_uppercase() {
            has_upper = true;
        } else if ch.is_ascii_lowercase() {
            has_lower = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if "!@#$%^&*()_+-=[]{}|;:,.<>?/~`".contains(ch) {
            has_special = true;
        }
    }

    if !has_upper {
        return Err(
            validator::ValidationError::new("password_missing_uppercase").with_message(
                std::borrow::Cow::Borrowed("Password must contain at least one uppercase letter"),
            ),
        );
    }
    if !has_lower {
        return Err(
            validator::ValidationError::new("password_missing_lowercase").with_message(
                std::borrow::Cow::Borrowed("Password must contain at least one lowercase letter"),
            ),
        );
    }
    if !has_digit {
        return Err(
            validator::ValidationError::new("password_missing_digit").with_message(
                std::borrow::Cow::Borrowed("Password must contain at least one digit"),
            ),
        );
    }
    if !has_special {
        return Err(validator::ValidationError::new("password_missing_special")
            .with_message(std::borrow::Cow::Borrowed("Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?/~`)")));
    }

    Ok(())
}
