use agrocore_domain::entities::olive::{OliveGrove, CreateOliveGroveDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{OliveGroveRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveGroveRepo { pool: PgPool }
impl PgOliveGroveRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl OliveGroveRepo for PgOliveGroveRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<OliveGrove>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<OliveGrove>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM olive_groves WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<OliveGrove> = sqlx::query_as("SELECT * FROM olive_groves WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateOliveGroveDto) -> RepositoryFuture<OliveGrove> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveGrove>("INSERT INTO olive_groves (tenant_id, name, hectares, olive_variety, plant_year, location) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(&dto.name).bind(dto.hectares).bind(&dto.olive_variety).bind(dto.plant_year).bind(dto.location)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, dto: agrocore_domain::entities::olive::UpdateOliveGroveDto) -> RepositoryFuture<Option<OliveGrove>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveGrove>("UPDATE olive_groves SET name = $1, hectares = $2 WHERE tenant_id = $3 AND id = $4 RETURNING *")
            .bind(&dto.name).bind(dto.hectares).bind(tid.to_string()).bind(id)
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE olive_groves SET is_active = false WHERE tenant_id = $1 AND id = $2")
            .bind(tid.to_string()).bind(id).execute(&pool).await.map(|r| r.rows_affected() > 0).map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}