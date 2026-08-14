use agrocore_domain::entities::finance::{CreateFinancialRecordDto, FinancialRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    FinancialRecordRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgFinancialRecordRepo);

impl FinancialRecordRepo for PgFinancialRecordRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FinancialRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FinancialRecord>(
                "SELECT * FROM financial_records WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<Option<FinancialRecord>> {
        self.find_by_id(tid, id)
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FinancialRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM financial_records WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)").bind(tid).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data: Vec<FinancialRecord> = sqlx::query_as("SELECT * FROM financial_records WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY date DESC LIMIT $2 OFFSET $3").bind(tid).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let total_pages = if total == 0 {
                0
            } else {
                (total as f64 / per_page as f64).ceil() as u64
            };
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }
    fn find_by_cost_center(
        &self,
        tid: TenantId,
        cost_center_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<FinancialRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM financial_records WHERE tenant_id = $1 AND cost_center_id = $2")
                .bind(tid).bind(cost_center_id).fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let data = sqlx::query_as("SELECT * FROM financial_records WHERE tenant_id = $1 AND cost_center_id = $2 ORDER BY date DESC LIMIT $3 OFFSET $4")
                .bind(tid).bind(cost_center_id).bind(per_page as i32).bind(offset as i32).fetch_all(&pool).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: if total == 0 {
                    0
                } else {
                    (total as f64 / per_page as f64).ceil() as u64
                },
            })
        })
    }
    fn create(
        &self,
        tid: TenantId,
        dto: CreateFinancialRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<FinancialRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, FinancialRecord>(
                "INSERT INTO financial_records (tenant_id, cost_center_id, record_type, amount, currency, date, category, description, reference_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *")
            .bind(tid).bind(dto.cost_center_id).bind(serde_json::to_value(&dto.record_type).unwrap()).bind(dto.amount).bind(dto.currency).bind(dto.date).bind(dto.category).bind(&dto.description).bind(dto.reference_id)
            .fetch_one(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: agrocore_domain::entities::finance::UpdateFinancialRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<FinancialRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as("UPDATE financial_records SET cost_center_id = COALESCE($1, cost_center_id), date = COALESCE($2, date), amount = COALESCE($3, amount), currency = COALESCE($4, currency), record_type = COALESCE($5, record_type), category = COALESCE($6, category), description = COALESCE($7, description), reference_id = COALESCE($8, reference_id), updated_at = NOW() WHERE id = $9 AND tenant_id = $10 RETURNING *")
                .bind(dto.cost_center_id).bind(dto.date).bind(dto.amount).bind(dto.currency).bind(dto.record_type.map(|v| serde_json::to_value(v).unwrap())).bind(dto.category).bind(dto.description).bind(dto.reference_id).bind(id).bind(tid)
                .fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM financial_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
