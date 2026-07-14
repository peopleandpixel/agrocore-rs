use agrocore_domain::entities::water::WaterUsage;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WaterUsageRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWaterUsageRepo { pool: PgPool }
impl PgWaterUsageRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl WaterUsageRepo for PgWaterUsageRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<WaterUsage>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterUsage>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM water_usages WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<WaterUsage> = sqlx::query_as("SELECT * FROM water_usages WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY date DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, _dto: agrocore_domain::entities::water::CreateWaterUsageDto) -> RepositoryFuture<WaterUsage> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, WaterUsage>("INSERT INTO water_usages (tenant_id, source_id, site_id, date, quantity_m3, irrigation_method) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(None::<Uuid>).bind(None::<Uuid>).bind(chrono::Utc::now().date_naive()).bind(0.0).bind(&agrocore_domain::entities::water::IrrigationMethod::Drip.to_string())
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: agrocore_domain::entities::water::UpdateWaterUsageDto) -> RepositoryFuture<Option<WaterUsage>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}