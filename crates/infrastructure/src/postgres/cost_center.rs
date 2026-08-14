use agrocore_domain::entities::finance::{CostCenter, CreateCostCenterDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{CostCenterRepo, RepositoryFuture};
use agrocore_shared::SharedError;
use agrocore_shared::{PaginatedResponse, Pagination};
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgCostCenterRepo);

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
        tid: TenantId,
        id: Uuid,
        dto: agrocore_domain::entities::finance::UpdateCostCenterDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<CostCenter>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as("UPDATE cost_centers SET label = COALESCE($1, label), code = COALESCE($2, code), cost_center_type = COALESCE($3, cost_center_type), reference_id = COALESCE($4, reference_id), is_active = COALESCE($5, is_active), updated_at = NOW() WHERE id = $6 AND tenant_id = $7 RETURNING *")
                .bind(dto.label).bind(dto.code).bind(dto.cost_center_type.map(|v| serde_json::to_value(v).unwrap())).bind(dto.reference_id).bind(dto.is_active).bind(id).bind(tid)
                .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE cost_centers SET is_active = false, updated_at = NOW() WHERE id = $1 AND tenant_id = $2")
                .bind(id).bind(tid).execute(&pool).await.map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
