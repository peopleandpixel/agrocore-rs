use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(pub Uuid);

impl Id {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for Id {
    fn default() -> Self { Self::new() }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self { Self(uuid) }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self { id.0 }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self { Self(Utc::now()) }
}

impl Default for Timestamp {
    fn default() -> Self { Self::now() }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Id>,
    pub updated_by: Option<Id>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = 0))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
}

impl Default for Pagination {
    fn default() -> Self { Self { page: Some(0), per_page: Some(20) } }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

impl TenantId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Id,
    pub tenant_id: TenantId,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Manager,
    Worker,
    Viewer,
}

pub type Result<T> = std::result::Result<T, SharedError>;

#[derive(Debug, thiserror::Error)]
pub enum SharedError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
}
