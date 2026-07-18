use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::water::WaterSource;
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, WaterSourceRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWaterSourceRepo {
    pool: PgPool,
}
impl PgWaterSourceRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

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
        _tid: TenantId,
        _site_id: Uuid,
        _p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::water::WaterSource>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn update(
        &self,
        _tid: TenantId,
        _id: Uuid,
        _dto: agrocore_domain::entities::water::UpdateWaterSourceDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<agrocore_domain::entities::water::WaterSource>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}
