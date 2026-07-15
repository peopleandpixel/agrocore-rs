use agrocore_domain::entities::harvest::{HarvestDelivery, CreateHarvestDeliveryDto, UpdateHarvestDeliveryDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestDeliveryRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

#[derive(Clone)]
pub struct PgHarvestDeliveryRepo {
    pool: PgPool,
}

impl PgHarvestDeliveryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestDeliveryRepo for PgHarvestDeliveryRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> Fut<Option<HarvestDelivery>> {
        Box::pin(async move { Ok(None) })
    }

    fn find_all(&self, _tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<HarvestDelivery>> {
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: 0 }) })
    }

    fn find_by_lot(&self, _tid: TenantId, _lot_id: Uuid, p: Pagination) -> Fut<PaginatedResponse<HarvestDelivery>> {
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: 0 }) })
    }

    fn create(&self, _tid: TenantId, _dto: CreateHarvestDeliveryDto, _by: Uuid) -> Fut<HarvestDelivery> {
        Box::pin(async move { Err(SharedError::Internal("TODO".into())) })
    }

    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdateHarvestDeliveryDto, _by: Uuid) -> Fut<Option<HarvestDelivery>> {
        Box::pin(async move { Ok(None) })
    }

    fn delete(&self, _tid: TenantId, _id: Uuid) -> Fut<bool> {
        Box::pin(async move { Ok(false) })
    }
}