use agrocore_domain::entities::compliance::{AuditLog, CreateAuditLogDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{AuditLogRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgAuditLogRepo {
    pool: PgPool,
}

impl PgAuditLogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AuditLogRepo for PgAuditLogRepo {
    fn find_by_id(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<Option<AuditLog>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }

    fn find_all(&self, _tid: TenantId, _p: Pagination) -> RepositoryFuture<PaginatedResponse<AuditLog>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
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
            .bind(tid.to_string())
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
