use agrocore_domain::entities::water::{WaterSource, CreateWaterSourceDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, WaterSourceRepo};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWaterSourceRepo { pool: PgPool, }

impl PgWaterSourceRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WaterSourceRepo for PgWaterSourceRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WaterSource>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterSource>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM water_sources WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WaterSource> = sqlx::query_as("SELECT * FROM water_sources WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages: 0 })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateWaterSourceDto) -> RepositoryFuture<WaterSource> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WaterSource>("INSERT INTO water_sources (tenant_id, label, source_type, capacity_l_per_h, location) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(&dto.label).bind(dto.source_type).bind(dto.capacity_l_per_h).bind(dto.location)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}