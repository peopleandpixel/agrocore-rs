use agrocore_domain::entities::finance::{FinancialRecord, CreateFinancialRecordDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{FinancialRecordRepo, RepositoryFuture, PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgFinancialRecordRepo { pool: PgPool, }
impl PgFinancialRecordRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl FinancialRecordRepo for PgFinancialRecordRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<FinancialRecord>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FinancialRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM financial_records WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<FinancialRecord> = sqlx::query_as("SELECT * FROM financial_records WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages: 0 })
        })
    }
    fn create(&self, tid: TenantId, _dto: CreateFinancialRecordDto) -> RepositoryFuture<FinancialRecord> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string()).to_error()) })
    }
}