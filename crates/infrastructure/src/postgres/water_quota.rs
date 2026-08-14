use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::water::{CreateWaterQuotaDto, WaterQuota};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, WaterQuotaRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgWaterQuotaRepo);

impl WaterQuotaRepo for PgWaterQuotaRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterQuota>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as("SELECT * FROM water_quotas WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(row)
        })
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<WaterQuota>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM water_quotas WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WaterQuota> = sqlx::query_as(
                "SELECT * FROM water_quotas WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: 0,
            })
        })
    }
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWaterQuotaDto,
        _by: Uuid,
    ) -> RepositoryFuture<WaterQuota> {
        let pool = self.pool.clone();
        let id = Uuid::new_v4();
        Box::pin(async move {
            let record = sqlx::query_as(
                "INSERT INTO water_quotas (id, tenant_id, source_id, site_id, year, allocated_m3, used_m3, remaining_m3, comunidad_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"
            )
            .bind(id)
            .bind(tid)
            .bind(dto.source_id)
            .bind(dto.site_id)
            .bind(dto.year)
            .bind(dto.allocated_m3)
            .bind(0.0) // used_m3 starts at 0
            .bind(dto.allocated_m3)
            .bind(dto.comunidad_id)
            .bind(chrono::Utc::now())
            .bind(chrono::Utc::now())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(record)
        })
    }
    fn find_by_source(
        &self,
        tid: TenantId,
        source_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::water::WaterQuota>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM water_quotas WHERE tenant_id = $1 AND source_id = $2",
            )
            .bind(tid)
            .bind(source_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            let data = sqlx::query_as("SELECT * FROM water_quotas WHERE tenant_id = $1 AND source_id = $2 ORDER BY year DESC LIMIT $3 OFFSET $4")
                .bind(tid).bind(source_id).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: if total == 0 {
                    0
                } else {
                    (total as f64 / per_page as f64).ceil() as u64
                },
            })
        })
    }
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: agrocore_domain::entities::water::UpdateWaterQuotaDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<agrocore_domain::entities::water::WaterQuota>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as("UPDATE water_quotas SET allocated_m3 = COALESCE($1, allocated_m3), used_m3 = COALESCE($2, used_m3), remaining_m3 = COALESCE($1, allocated_m3) - COALESCE($2, used_m3), comunidad_id = COALESCE($3, comunidad_id), updated_at = NOW() WHERE id = $4 AND tenant_id = $5 RETURNING *")
                .bind(dto.allocated_m3).bind(dto.used_m3).bind(dto.comunidad_id).bind(id).bind(tid)
                .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM water_quotas WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
