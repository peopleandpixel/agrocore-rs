use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 100))]
    pub firstname: String,
    #[validate(length(min = 1, max = 100))]
    pub lastname: String,
    #[validate(email)]
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub assigned_site_ids: Option<Vec<Uuid>>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum Resource {
    Site,
    Equipment,
    Order,
    User,
    Tenant,
    Finance,
    Analytics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Manage, // Abstract action
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
    pub scope: PermissionScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum PermissionScope {
    All,
    Own,
    Tenant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum UserRole {
    Admin,
    Manager,
    Worker,
    Viewer,
    Custom(Uuid),
}

impl UserRole {
    pub fn has_permission(&self, _resource: Resource, _action: Action, _role_repo: Option<&Role>) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::Manager => true,
            UserRole::Worker => matches!(_action, Action::Read) || matches!(_action, Action::Update),
            UserRole::Viewer => matches!(_action, Action::Read),
            UserRole::Custom(_) => {
                // Logic to check role_repo for permissions
                false
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub tenant_id: TenantId,
    pub firstname: String,
    pub lastname: String,
    pub roles: Vec<UserRole>,
}
