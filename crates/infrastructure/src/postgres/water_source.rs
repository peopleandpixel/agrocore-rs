use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::water::WaterSource;
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, WaterSourceRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgWaterSourceRepo);

impl WaterSourceRepo for PgWaterSourceRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterSource>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row =
                sqlx::query_as("SELECT * FROM water_sources WHERE id = $1 AND tenant_id = $2")
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
    ) -> RepositoryFuture<PaginatedResponse<WaterSource>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM water_sources WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WaterSource> = sqlx::query_as(
                "SELECT * FROM water_sources WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 {
                0
            } else {
                (total as f64 / per_page as f64).ceil() as u64
            };
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }
    fn create(
        &self,
        tid: TenantId,
        dto: agrocore_domain::entities::water::CreateWaterSourceDto,
        _by: Uuid,
    ) -> RepositoryFuture<WaterSource> {
        let pool = self.pool.clone();
        let id = Uuid::new_v4();
        Box::pin(async move {
            let record = sqlx::query_as(
                "INSERT INTO water_sources (id, tenant_id, site_id, source_type, name, capacity_m3, license_number, license_expiry, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"
            )
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(serde_json::to_value(&dto.source_type).unwrap())
            .bind(dto.name)
            .bind(dto.capacity_m3)
            .bind(dto.license_number)
            .bind(dto.license_expiry)
            .bind(chrono::Utc::now())
            .bind(chrono::Utc::now())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(record)
        })
    }
    fn find_by_site(
        &self,
        tid: TenantId,
        site_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::water::WaterSource>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM water_sources WHERE tenant_id = $1 AND site_id = $2",
            )
            .bind(tid)
            .bind(site_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            let data = sqlx::query_as(
                "SELECT * FROM water_sources WHERE tenant_id = $1 AND site_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid).bind(site_id).bind(per_page as i32).bind(offset as i32)
            .fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
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
        dto: agrocore_domain::entities::water::UpdateWaterSourceDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<agrocore_domain::entities::water::WaterSource>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as("UPDATE water_sources SET source_type = COALESCE($1, source_type), name = COALESCE($2, name), capacity_m3 = COALESCE($3, capacity_m3), current_level_m3 = COALESCE($4, current_level_m3), license_number = COALESCE($5, license_number), license_expiry = COALESCE($6, license_expiry), is_active = COALESCE($7, is_active), updated_at = NOW() WHERE id = $8 AND tenant_id = $9 RETURNING *")
                .bind(dto.source_type.map(|v| serde_json::to_value(v).unwrap())).bind(dto.name).bind(dto.capacity_m3).bind(dto.current_level_m3).bind(dto.license_number).bind(dto.license_expiry).bind(dto.is_active).bind(id).bind(tid)
                .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE water_sources SET is_active = false, updated_at = NOW() WHERE id = $1 AND tenant_id = $2")
                .bind(id).bind(tid).execute(&pool).await.map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
