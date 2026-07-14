use agrocore_domain::entities::water::{WaterQuota, CreateWaterQuotaDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WaterQuotaRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWaterQuotaRepo { pool: PgPool, }
impl PgWaterQuotaRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WaterQuotaRepo for PgWaterQuotaRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterQuota>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as("SELECT * FROM water_quotas WHERE id = $1::uuid AND tenant_id = $2::uuid")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(row)
        })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterQuota>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM water_quotas WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WaterQuota> = sqlx::query_as(
                "SELECT * FROM water_quotas WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3"
            )
            .bind(tid.to_string())
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages: 0 })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateWaterQuotaDto) -> RepositoryFuture<WaterQuota> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let record = sqlx::query_as(
                "INSERT INTO water_quotas (id, tenant_id, source_id, period_start, period_end, volume_m3, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(dto.id)
            .bind(tid.to_string())
            .bind(dto.source_id)
            .bind(dto.period_start)
            .bind(dto.period_end)
            .bind(dto.volume_m3)
            .bind(chrono::Utc::now())
            .bind(chrono::Utc::now())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(record)
        })
    }
}