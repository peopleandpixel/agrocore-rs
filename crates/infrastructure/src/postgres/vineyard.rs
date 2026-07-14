use agrocore_domain::entities::vineyard::{CreateVineyardDto, Vineyard, UpdateVineyardDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, VineyardRepo};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgVineyardRepo {
    pool: PgPool,
}

impl PgVineyardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl VineyardRepo for PgVineyardRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<Vineyard>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Vineyard>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vineyards WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<Vineyard> = sqlx::query_as("SELECT * FROM vineyards WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateVineyardDto) -> RepositoryFuture<Vineyard> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Vineyard>("INSERT INTO vineyards (tenant_id, name, hectares, doc_area, quality_grade) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string())
            .bind(&dto.name)
            .bind(dto.hectares)
            .bind(dto.doc_area)
            .bind(dto.quality_grade)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateVineyardDto) -> RepositoryFuture<Option<Vineyard>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Vineyard>("UPDATE vineyards SET name = $1, hectares = $2, updated_at = NOW() WHERE tenant_id = $3 AND id = $4 RETURNING *")
            .bind(&dto.name)
            .bind(dto.hectares)
            .bind(tid.to_string())
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE vineyards SET is_active = false WHERE tenant_id = $1 AND id = $2")
            .bind(tid.to_string())
            .bind(id)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}