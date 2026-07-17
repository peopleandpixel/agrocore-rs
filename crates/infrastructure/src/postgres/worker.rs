use crate::postgres::audit_log::PgAuditLogRepo;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::workforce::{CreateWorkerDto, UpdateWorkerDto, Worker};
use agrocore_domain::repositories::{
    AuditLogRepo, PaginatedResponse, Pagination, RepositoryFuture, WorkerRepo,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgWorkerRepo {
    pool: PgPool,
}

impl PgWorkerRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WorkerRepo for PgWorkerRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Worker>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Worker>(
                "SELECT * FROM workers WHERE id = $1 AND tenant_id = $2 AND is_active = true",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_user_id(&self, tid: TenantId, user_id: Uuid) -> RepositoryFuture<Option<Worker>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Worker>(
                "SELECT * FROM workers WHERE user_id = $1 AND tenant_id = $2 AND is_active = true",
            )
            .bind(user_id)
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
    ) -> RepositoryFuture<PaginatedResponse<Worker>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM workers WHERE tenant_id = $1 AND is_active = true",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Worker> = sqlx::query_as("SELECT * FROM workers WHERE tenant_id = $1 AND is_active = true LIMIT $2 OFFSET $3")
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

    fn create(&self, tid: TenantId, dto: CreateWorkerDto, by: Uuid) -> RepositoryFuture<Worker> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            let id = Uuid::new_v4();
            let entity = sqlx::query_as::<_, Worker>(
                r#"INSERT INTO workers (id, tenant_id, user_id, contract_type, language, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.user_id)
            .bind(serde_json::to_value(&dto.contract_type).unwrap())
            .bind(&dto.language)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let _ = audit_repo
                .create(
                    tid,
                    agrocore_domain::entities::compliance::CreateAuditLogDto {
                        tenant_id: tid,
                        user_id: by,
                        action: agrocore_domain::entities::compliance::AuditAction::Created,
                        entity_type: "Worker".into(),
                        entity_id: entity.id,
                        old_value: None,
                        new_value: Some(serde_json::to_value(&entity).unwrap()),
                        ip_address: None,
                    },
                )
                .await;

            Ok(entity)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateWorkerDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Worker>> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            let old_val = sqlx::query_as::<_, Worker>(
                "SELECT * FROM workers WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let entity = sqlx::query_as::<_, Worker>(
                r#"UPDATE workers SET language = COALESCE($1, language), updated_at = NOW()
                   WHERE id = $2 AND tenant_id = $3 RETURNING *"#,
            )
            .bind(&dto.language)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let (Some(old), Some(new)) = (&old_val, &entity) {
                let _ = audit_repo
                    .create(
                        tid,
                        agrocore_domain::entities::compliance::CreateAuditLogDto {
                            tenant_id: tid,
                            user_id: by,
                            action: agrocore_domain::entities::compliance::AuditAction::Updated,
                            entity_type: "Worker".into(),
                            entity_id: new.id,
                            old_value: Some(serde_json::to_value(old).unwrap()),
                            new_value: Some(serde_json::to_value(new).unwrap()),
                            ip_address: None,
                        },
                    )
                    .await;
            }

            Ok(entity)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE workers SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
