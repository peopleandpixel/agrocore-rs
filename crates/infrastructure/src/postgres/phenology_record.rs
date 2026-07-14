use agrocore_domain::entities::weather::{PhenologyRecord, CreatePhenologyRecordDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PhenologyRecordRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPhenologyRecordRepo { pool: PgPool }
impl PgPhenologyRecordRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl PhenologyRecordRepo for PgPhenologyRecordRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<PhenologyRecord>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PhenologyRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM phenology_records WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<PhenologyRecord> = sqlx::query_as("SELECT * FROM phenology_records WHERE tenant_id = $1::uuid ORDER BY date DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreatePhenologyRecordDto) -> RepositoryFuture<PhenologyRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PhenologyRecord>(
                "INSERT INTO phenology_records (tenant_id, site_id, bbch_stage, date, observation) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid.to_string()).bind(dto.site_id.to_string()).bind(dto.bbch_stage).bind(dto.date).bind(&dto.observation)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}