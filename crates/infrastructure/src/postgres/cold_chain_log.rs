use agrocore_domain::entities::coldchain::ColdChainLog;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{ColdChainLogRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgColdChainLogRepo {
    pool: PgPool,
}

impl PgColdChainLogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ColdChainLogRepository for PgColdChainLogRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<ColdChainLog>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<ColdChainLog>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: agrocore_domain::entities::coldchain::CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
}