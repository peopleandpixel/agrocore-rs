use agrocore_domain::entities::finance::{FinancialRecord, CreateFinancialRecordDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{FinancialRecordRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgFinancialRecordRepo { pool: PgPool }
impl PgFinancialRecordRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl FinancialRecordRepo for PgFinancialRecordRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<FinancialRecord>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FinancialRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM financial_records WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<FinancialRecord> = sqlx::query_as("SELECT * FROM financial_records WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY date DESC LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateFinancialRecordDto) -> RepositoryFuture<FinancialRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FinancialRecord>(
                "INSERT INTO financial_records (tenant_id, cost_center_id, record_type, amount_cents, date, description, reference, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7, true) RETURNING *")
            .bind(tid.to_string()).bind(dto.cost_center_id).bind(dto.record_type).bind(dto.amount_cents).bind(dto.date).bind(&dto.description).bind(&dto.reference)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}