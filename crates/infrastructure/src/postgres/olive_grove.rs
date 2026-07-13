use agrocore_domain::entities::olive::OliveGrove;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{OliveGroveRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveGroveRepo {
    pool: PgPool,
}

impl PgOliveGroveRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OliveGroveRepository for PgOliveGroveRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveGrove>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<OliveGrove>> {
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page, per_page, total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: agrocore_domain::entities::olive::CreateOliveGroveDto) -> RepositoryFuture<OliveGrove> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::olive::UpdateOliveGroveDto) -> RepositoryFuture<Option<OliveGrove>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}