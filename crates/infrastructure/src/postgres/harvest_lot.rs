use agrocore_domain::entities::harvest::{HarvestLot, CreateHarvestLotDto, UpdateHarvestLotDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestLotRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

#[derive(Clone)]
pub struct PgHarvestLotRepo {
    pool: PgPool,
}

impl PgHarvestLotRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestLotRepo for PgHarvestLotRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> Fut<Option<HarvestLot>> {
        Box::pin(async move {
            Ok(None) // TODO
        })
    }

    fn find_all(&self, _tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<HarvestLot>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move {
            Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 })
        })
    }

    fn find_by_season(&self, _tid: TenantId, _season_id: Uuid, p: Pagination) -> Fut<PaginatedResponse<HarvestLot>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move {
            Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 })
        })
    }

    fn create(&self, _tid: TenantId, _dto: CreateHarvestLotDto, _by: Uuid) -> Fut<HarvestLot> {
        Box::pin(async move { Err(SharedError::Internal("TODO".into())) })
    }

    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdateHarvestLotDto, _by: Uuid) -> Fut<Option<HarvestLot>> {
        Box::pin(async move { Ok(None) })
    }

    fn delete(&self, _tid: TenantId, _id: Uuid) -> Fut<bool> {
        Box::pin(async move { Ok(false) })
    }
}