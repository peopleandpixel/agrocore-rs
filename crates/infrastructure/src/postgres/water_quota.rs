use agrocore_domain::entities::water::{WaterQuota, CreateWaterQuotaDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WaterQuotaRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::SharedError;
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
    fn create(&self, tid: TenantId, dto: CreateWaterQuotaDto, _by: Uuid) -> RepositoryFuture<WaterQuota> {
        let pool = self.pool.clone();
        let id = Uuid::new_v4();
        Box::pin(async move {
            let record = sqlx::query_as(
                "INSERT INTO water_quotas (id, tenant_id, source_id, year, allocated_m3, used_m3, comunidad_id, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(id)
            .bind(tid.to_string())
            .bind(dto.source_id)
            .bind(dto.year)
            .bind(dto.allocated_m3)
            .bind(0.0) // used_m3 starts at 0
            .bind(dto.comunidad_id)
            .bind(chrono::Utc::now())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(record)
        })
    }
    fn find_by_source(&self, _tid: TenantId, _source_id: Uuid, _p: Pagination) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::water::WaterQuota>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::water::UpdateWaterQuotaDto, _by: Uuid) -> RepositoryFuture<Option<agrocore_domain::entities::water::WaterQuota>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}