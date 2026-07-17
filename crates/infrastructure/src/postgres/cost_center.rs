use agrocore_domain::entities::finance::{CostCenter, CreateCostCenterDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{CostCenterRepo, RepositoryFuture};
use agrocore_shared::SharedError;
use agrocore_shared::{PaginatedResponse, Pagination};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgCostCenterRepo {
    pool: PgPool,
}
impl PgCostCenterRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CostCenterRepo for PgCostCenterRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<CostCenter>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, CostCenter>(
                "SELECT * FROM cost_centers WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<CostCenter>> {
        self.find_by_id(tid, id)
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<CostCenter>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM cost_centers WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<CostCenter> = sqlx::query_as(
                "SELECT * FROM cost_centers WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }
    fn create(
        &self,
        tid: TenantId,
        dto: CreateCostCenterDto,
        _by: Uuid,
    ) -> RepositoryFuture<CostCenter> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, CostCenter>("INSERT INTO cost_centers (tenant_id, label, cost_center_type, code, reference_id) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(tid).bind(&dto.label).bind(serde_json::to_value(&dto.cost_center_type).unwrap()).bind(&dto.code).bind(dto.reference_id)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(
        &self,
        _tid: TenantId,
        _id: Uuid,
        _dto: agrocore_domain::entities::finance::UpdateCostCenterDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<CostCenter>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string())) })
    }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".to_string())) })
    }
}
