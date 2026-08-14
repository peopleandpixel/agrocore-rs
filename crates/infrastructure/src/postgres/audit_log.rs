use agrocore_domain::entities::compliance::{AuditLog, CreateAuditLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    AuditLogRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgAuditLogRepo);

impl AuditLogRepo for PgAuditLogRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<AuditLog>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, AuditLog>(
                "SELECT * FROM audit_logs WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tid)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<AuditLog>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<AuditLog> = sqlx::query_as("SELECT * FROM audit_logs WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
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

    fn create(&self, tid: TenantId, dto: CreateAuditLogDto) -> RepositoryFuture<AuditLog> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, AuditLog>(
                r#"INSERT INTO audit_logs (id, tenant_id, user_id, action, entity_type, entity_id, old_value, new_value, ip_address, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.user_id)
            .bind(serde_json::to_value(&dto.action).unwrap())
            .bind(&dto.entity_type)
            .bind(dto.entity_id)
            .bind(&dto.old_value)
            .bind(&dto.new_value)
            .bind(&dto.ip_address)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
