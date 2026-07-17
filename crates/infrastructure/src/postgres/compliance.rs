use agrocore_domain::entities::compliance::{
    ComplianceChecklist, CreateComplianceChecklistDto, UpdateComplianceChecklistDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    ComplianceChecklistRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgComplianceChecklistRepo {
    pool: PgPool,
}

impl PgComplianceChecklistRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ComplianceChecklistRepo for PgComplianceChecklistRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ComplianceChecklist>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ComplianceChecklist>(
                "SELECT * FROM compliance_checklists WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<ComplianceChecklist>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM compliance_checklists WHERE tenant_id = $1",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<ComplianceChecklist> = sqlx::query_as(
                "SELECT * FROM compliance_checklists WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

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

    fn create(
        &self,
        tid: TenantId,
        dto: CreateComplianceChecklistDto,
        _by: Uuid,
    ) -> RepositoryFuture<ComplianceChecklist> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, ComplianceChecklist>(
                r#"INSERT INTO compliance_checklists (id, tenant_id, site_id, checklist_type, status, items, due_date, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(serde_json::to_value(&dto.checklist_type).unwrap())
            .bind(serde_json::to_value(&agrocore_domain::entities::compliance::ComplianceStatus::Pending).unwrap())
            .bind(serde_json::to_value(&dto.items).unwrap())
            .bind(dto.due_date)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateComplianceChecklistDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<ComplianceChecklist>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, ComplianceChecklist>(
                r#"UPDATE compliance_checklists SET status = COALESCE($1, status), updated_at = NOW()
                   WHERE id = $2 AND tenant_id = $3 RETURNING *"#)
            .bind(dto.status.map(|s| serde_json::to_value(s).unwrap()))
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM compliance_checklists WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
