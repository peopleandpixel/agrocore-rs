use agrocore_domain::entities::harvest::HarvestLot;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestLotRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestLotRepo {
    pool: PgPool,
}

impl PgHarvestLotRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestLotRepository for PgHarvestLotRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestLot>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestLot>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: agrocore_domain::entities::harvest::CreateHarvestLotDto) -> RepositoryFuture<HarvestLot> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::harvest::UpdateHarvestLotDto) -> RepositoryFuture<Option<HarvestLot>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}