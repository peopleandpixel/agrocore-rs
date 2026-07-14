use agrocore_domain::entities::harvest::{HarvestDelivery, CreateHarvestDeliveryDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestDeliveryRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestDeliveryRepo { pool: PgPool }
impl PgHarvestDeliveryRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl HarvestDeliveryRepo for PgHarvestDeliveryRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestDelivery>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM harvest_deliveries WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<HarvestDelivery> = sqlx::query_as("SELECT * FROM harvest_deliveries WHERE tenant_id = $1::uuid ORDER BY delivery_date DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateHarvestDeliveryDto) -> RepositoryFuture<HarvestDelivery> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestDelivery>(
                "INSERT INTO harvest_deliveries (tenant_id, lot_id, delivery_date, quantity_kg, quality_grade, notes) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(dto.lot_id).bind(dto.delivery_date).bind(dto.quantity_kg).bind(dto.quality_grade).bind(&dto.notes)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, _dto: agrocore_domain::entities::harvest::UpdateHarvestDeliveryDto) -> RepositoryFuture<Option<HarvestDelivery>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestDelivery>("UPDATE harvest_deliveries SET notes = $1 WHERE tenant_id = $2 AND id = $3 RETURNING *")
            .bind("updated").bind(tid.to_string()).bind(id)
            .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE harvest_deliveries SET is_active = false WHERE tenant_id = $1 AND id = $2")
            .bind(tid.to_string()).bind(id).execute(&pool).await.map(|r| r.rows_affected() > 0).map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}