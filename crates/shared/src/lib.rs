use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "sqlx")]
#[allow(clippy::single_component_path_imports)]
use sqlx;

pub mod config;
pub mod lpis;
pub mod telemetry;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Id(pub Uuid);

impl Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Id>,
    pub updated_by: Option<Id>,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct Pagination {
    #[validate(range(min = 0))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 500))]
    pub per_page: Option<u64>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Some(0),
            per_page: Some(20),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[cfg(feature = "sqlx")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct TenantId(pub Uuid);

#[cfg(not(feature = "sqlx"))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct TenantId(pub Uuid);

impl TenantId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
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
    #[error("Database error: {0}")]
    Database(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Reference error: {0}")]
    ReferenceError(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for SharedError {
    fn from(err: sqlx::Error) -> Self {
        SharedError::Database(err.to_string())
    }
}

/// Generic retry helper with exponential backoff.
///
/// Repeatedly calls `operation` up to `max_retries` times, sleeping
/// `base_delay * 2^attempt` milliseconds between attempts.
/// Logs each retry at `warn` level and returns the final error on exhaustion.
///
/// # Arguments
/// * `operation_name` — human-readable name for logging (e.g. "connect to database")
/// * `max_retries` — maximum number of retry attempts
/// * `base_delay_secs` — base delay in seconds; actual delay grows exponentially
/// * `operation` — async closure returning `Result<T, E>`
///
/// # Example
/// ```ignore
/// let pool = with_retry(
///     "connect to database",
///     10, 1,
///     || async { pool_options.connect(&url).await },
/// ).await?;
/// ```
pub async fn with_retry<F, T, E>(
    operation_name: &str,
    max_retries: u32,
    base_delay_secs: u64,
    operation: F,
) -> std::result::Result<T, E>
where
    F: Fn()
        -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut retry_count = 0u32;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                let delay =
                    std::time::Duration::from_secs(base_delay_secs * (1u64 << (retry_count - 1)));
                tracing::warn!(
                    "Failed to {} (attempt {}/{}): {}. Retrying in {:?}...",
                    operation_name,
                    retry_count,
                    max_retries,
                    e,
                    delay
                );
                tokio::time::sleep(delay).await;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

/// Macro to create repository instances with reduced boilerplate.
/// Usage: `repo!(PgSiteRepo, SiteRepository, pool)`
#[macro_export]
macro_rules! repo {
    ($repo_type:ty, $trait_type:ty, $pool:expr) => {{
        use std::sync::Arc;
        let repo_instance = <$repo_type>::new($pool.clone());
        std::convert::Into::<Arc<dyn $trait_type>>::into(Arc::new(repo_instance))
    }};
}

/// Macro to generate a PostgreSQL repository struct with pool field and constructor.
/// Eliminates the boilerplate of `struct PgXxxRepo { pool: PgPool }` + `fn new()`.
///
/// Usage: `pg_repo!(PgSiteRepo);`
/// Generates:
/// ```ignore
/// #[derive(Clone)]
/// pub struct PgSiteRepo {
///     pool: PgPool,
/// }
/// impl PgSiteRepo {
///     pub fn new(pool: PgPool) -> Self {
///         Self { pool }
///     }
/// }
/// ```
#[cfg(feature = "sqlx")]
#[macro_export]
macro_rules! pg_repo {
    ($struct_name:ident) => {
        #[derive(Clone)]
        pub struct $struct_name {
            pub pool: sqlx::PgPool,
        }

        impl $struct_name {
            pub fn new(pool: sqlx::PgPool) -> Self {
                Self { pool }
            }
        }
    };
}

/// Macro to execute a database query with automatic pool cloning and error mapping.
/// Reduces the common pattern: `let pool = self.pool.clone(); Box::pin(async move { ... })`.
///
/// Usage: `db_exec!(self.pool, { sqlx::query_as!(...).fetch_optional(&pool).await })`
#[cfg(feature = "sqlx")]
#[macro_export]
macro_rules! db_exec {
    ($pool:expr, $body:expr) => {{
        let pool_ref: &sqlx::PgPool = &$pool;
        std::boxed::Box::pin(async move {
            let result: Result<_, sqlx::Error> = async {
                let pool = pool_ref.clone();
                $body
            }.await;
            result
        })
    }};
}
