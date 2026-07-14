use agrocore_domain::entities::finance::{PACApplication, CreatePACApplicationDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PACApplicationRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPACApplicationRepo { pool: PgPool }
impl PgPACApplicationRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl PACApplicationRepo for PgPACApplicationRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pac_applications WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<PACApplication> = sqlx::query_as("SELECT * FROM pac_applications WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreatePACApplicationDto) -> RepositoryFuture<PACApplication> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PACApplication>(
                "INSERT INTO pac_applications (tenant_id, eco_scheme, hectares, application_date, status) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(&dto.eco_scheme).bind(dto.hectares).bind(dto.application_date).bind(dto.status)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}