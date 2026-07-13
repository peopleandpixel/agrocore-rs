use crate::postgres::site::PgSiteRepo;
use crate::postgres::user::PgUserRepo;
use crate::postgres::order::PgOrderRepo;
use crate::postgres::tenant::PgTenantRepo;
use agrocore_domain::repositories::{SiteRepository, OrderRepository, UserRepository, TenantRepository};
use sqlx::PgPool;
use std::sync::Arc;

pub enum Database {
    Postgres(PostgresDb),
    #[cfg(feature = "mongodb")]
    Mongo(MongoDb),
}

/// PostgreSQL Database Wrapper
pub struct PostgresDb {
    pub pool: PgPool,
}

impl PostgresDb {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub fn site_repo(&self) -> Arc<dyn SiteRepository> {
        Arc::new(PgSiteRepo::new(self.pool.clone()))
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        Arc::new(PgUserRepo::new(self.pool.clone()))
    }

    pub fn order_repo(&self) -> Arc<dyn OrderRepository> {
        Arc::new(PgOrderRepo::new(self.pool.clone()))
    }

    pub fn tenant_repo(&self) -> Arc<dyn TenantRepository> {
        Arc::new(PgTenantRepo::new(self.pool.clone()))
    }
}

// Re-exports für PostgreSQL Repositories
pub use crate::postgres::site::PgSiteRepo;
pub use crate::postgres::user::PgUserRepo;
pub use crate::postgres::order::PgOrderRepo;
pub use crate::postgres::tenant::PgTenantRepo;