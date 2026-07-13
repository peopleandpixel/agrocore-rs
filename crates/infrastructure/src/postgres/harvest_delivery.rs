use agrocore_domain::entities::harvest::HarvestDelivery;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestDeliveryRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestDeliveryRepo {
    pool: PgPool,
}

impl PgHarvestDeliveryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestDeliveryRepository for PgHarvestDeliveryRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestDelivery>> {
        let _ = self.pool.clone();
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestDelivery>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: agrocore_domain::entities::harvest::CreateHarvestDeliveryDto) -> RepositoryFuture<HarvestDelivery> {
        let _ = self.pool.clone();
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::harvest::UpdateHarvestDeliveryDto) -> RepositoryFuture<Option<HarvestDelivery>> {
        let _ = self.pool.clone();
        Box::pin(async move { Ok(None) })
    }
}