use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
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
    #[sqlx(json)]
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    #[sqlx(json)]
    pub assigned_site_ids: Option<Vec<Uuid>>,
    pub last_login: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
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
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub is_system: bool, // System roles cannot be deleted/modified
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub fn has_permission(
        &self,
        _resource: Resource,
        _action: Action,
        _role_repo: Option<&Role>,
    ) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::Manager => true,
            UserRole::Worker => {
                matches!(_action, Action::Read) || matches!(_action, Action::Update)
            }
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default, ToSchema)]
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

// =============================================================================
// ADVANCED RBAC: Custom Roles & API Keys
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRoleDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default, ToSchema)]
pub struct UpdateRoleDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateApiKeyDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub roles: Vec<UserRole>, // Roles assigned to this API key
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<String>>, // CIDR notation
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default, ToSchema)]
pub struct UpdateApiKeyDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<String>>,
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub roles: Vec<UserRole>,
    pub key_prefix: String, // First 8 chars of key for display
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<String>>,
    pub allowed_domains: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResult {
    pub api_key: ApiKeyResponse,
    pub plain_key: String, // Only returned once on creation
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub key_hash: String,   // Argon2 hash
    pub key_prefix: String, // First 8 chars
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<String>>,
    pub allowed_domains: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// PERMISSION CHECKING UTILITIES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PermissionCheckRequest {
    pub resource: Resource,
    pub action: Action,
    pub resource_id: Option<Uuid>, // For ownership checks
    pub scope: PermissionScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PermissionCheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_permissions: Vec<Permission>,
}

// =============================================================================
// VISIBILITY SECURITY: User Entity implements VisibilityAwareEntity
// =============================================================================
use crate::repositories::VisibilityAwareEntity;

impl VisibilityAwareEntity for User {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
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
