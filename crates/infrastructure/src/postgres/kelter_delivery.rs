use agrocore_domain::entities::finance::{KelterDelivery, CreateKelterDeliveryDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, KelterDeliveryRepo};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgKelterDeliveryRepo { pool: PgPool, }

impl PgKelterDeliveryRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl KelterDeliveryRepo for PgKelterDeliveryRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<KelterDelivery>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<KelterDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kelter_deliveries WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<KelterDelivery> = sqlx::query_as("SELECT * FROM kelter_deliveries WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages: 0 })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateKelterDeliveryDto) -> RepositoryFuture<KelterDelivery> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, KelterDelivery>("INSERT INTO kelter_deliveries (tenant_id, site_id, delivery_date, quantity_liters, sugar_brix, acidity_ph) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(dto.site_id.to_string()).bind(dto.delivery_date).bind(dto.quantity_liters).bind(dto.sugar_brix).bind(dto.acidity_ph)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}