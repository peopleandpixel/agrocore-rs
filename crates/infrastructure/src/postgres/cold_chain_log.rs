use agrocore_domain::entities::harvest::{ColdChainLog, CreateColdChainLogDto, UpdateColdChainLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{ColdChainLogRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgColdChainLogRepo { pool: PgPool }
impl PgColdChainLogRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl ColdChainLogRepo for PgColdChainLogRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<ColdChainLog>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<ColdChainLog>> {
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: 0 }) })
    }
    fn find_by_lot(&self, _tid: TenantId, _lot_id: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<ColdChainLog>> {
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: CreateColdChainLogDto) -> RepositoryFuture<ColdChainLog> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdateColdChainLogDto) -> RepositoryFuture<Option<ColdChainLog>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}
