use agrocore_domain::entities::harvest::{HarvestLot, CreateHarvestLotDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestLotRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgHarvestLotRepo { pool: PgPool }
impl PgHarvestLotRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl HarvestLotRepo for PgHarvestLotRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<HarvestLot>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestLot>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM harvest_lots WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<HarvestLot> = sqlx::query_as("SELECT * FROM harvest_lots WHERE tenant_id = $1::uuid ORDER BY created_at DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateHarvestLotDto) -> RepositoryFuture<HarvestLot> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestLot>("INSERT INTO harvest_lots (tenant_id, site_id, season_id, label, harvested_at, yield_kg) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
            .bind(tid.to_string()).bind(dto.site_id.to_string()).bind(dto.season_id.to_string()).bind(&dto.label).bind(dto.harvested_at).bind(dto.yield_kg)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, _dto: agrocore_domain::entities::harvest::UpdateHarvestLotDto) -> RepositoryFuture<Option<HarvestLot>> {
        Box::pin(async move { Ok(None) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}