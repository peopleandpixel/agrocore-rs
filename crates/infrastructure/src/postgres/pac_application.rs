use agrocore_domain::entities::finance::{PACApplication, CreatePACApplicationDto, UpdatePACApplicationDto};
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PACApplicationRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPACApplicationRepo { pool: PgPool }
impl PgPACApplicationRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl PACApplicationRepo for PgPACApplicationRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<PACApplication>> {
        self.find_by_id(tid, id)
    }
    fn find_all(&self, _tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        Box::pin(async move { Ok(PaginatedResponse { data: vec![], total: 0, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: 0 }) })
    }
    fn create(&self, _tid: TenantId, _dto: CreatePACApplicationDto, _by: Uuid) -> RepositoryFuture<PACApplication> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdatePACApplicationDto, _by: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
    fn find_by_year(&self, _tid: TenantId, _year: i32, _p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}
