use agrocore_domain::entities::finance::{CostCenter, CreateCostCenterDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{CostCenterRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgCostCenterRepo { pool: PgPool, }
impl PgCostCenterRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl CostCenterRepo for PgCostCenterRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<CostCenter>> {
        Box::pin(async move { Ok(None) })
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<CostCenter>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0); let per_page = p.per_page.unwrap_or(20); let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cost_centers WHERE tenant_id = $1::uuid").bind(tid.to_string()).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<CostCenter> = sqlx::query_as("SELECT * FROM cost_centers WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3").bind(tid.to_string()).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages: 0 })
        })
    }
    fn create(&self, tid: TenantId, dto: CreateCostCenterDto) -> RepositoryFuture<CostCenter> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, CostCenter>("INSERT INTO cost_centers (tenant_id, label, center_type, code) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(tid.to_string()).bind(&dto.label).bind(dto.center_type).bind(&dto.code)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}