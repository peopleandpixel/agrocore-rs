use agrocore_domain::entities::harvest::HarvestSeason;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestSeasonRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestSeasonRepo {
    pool: PgPool,
}

impl PgHarvestSeasonRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestSeasonRepo for PgHarvestSeasonRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestSeason>> {
        let pool = self.pool.clone();
        let _ = (tid, id, pool);
        Box::pin(async move { Err(SharedError::NotFound.to_error()) })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestSeason>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        
        Box::pin(async move {
            Ok(PaginatedResponse {
                data: vec![],
                total: 0,
                page,
                per_page,
                total_pages: 0,
            })
        })
    }
}