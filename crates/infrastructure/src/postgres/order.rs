use crate::postgres::audit_log::PgAuditLogRepo;
use agrocore_domain::entities::order::{CreateOrderDto, Order, UpdateOrderDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    AuditLogRepo, OrderRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOrderRepo {
    pool: PgPool,
}

impl PgOrderRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OrderRepository for PgOrderRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Order>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1 AND tenant_id = $2")
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
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<Order>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);
            if can_see_all {
                sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                // Workers only see orders assigned to them
                sqlx::query_as::<_, Order>(
                    "SELECT * FROM orders WHERE id = $1 AND tenant_id = $2 AND assigned_to = $3",
                )
                .bind(id)
                .bind(tid)
                .bind(user_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Order>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Order> = sqlx::query_as("SELECT * FROM orders WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
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

    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Order>> {
        self.find_all(tid, p)
    }

    fn find_my_tasks(&self, tid: TenantId, user_id: Uuid) -> RepositoryFuture<Vec<Order>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE tenant_id = $1 AND assigned_to = $2 AND is_active = true")
                .bind(tid)
                .bind(user_id)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateOrderDto, by: Uuid) -> RepositoryFuture<Order> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            let id = Uuid::new_v4();
            let order = sqlx::query_as::<_, Order>(
                r#"INSERT INTO orders (id, tenant_id, label, order_type, status, created_at, updated_at, is_active)
                   VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), true)
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(&dto.label)
            .bind(serde_json::to_value(&dto.order_type).unwrap())
            .bind(serde_json::to_value(agrocore_domain::entities::OrderStatus::Planned).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            // Audit Log
            let _ = audit_repo
                .create(
                    tid,
                    agrocore_domain::entities::compliance::CreateAuditLogDto {
                        tenant_id: tid,
                        user_id: by,
                        action: agrocore_domain::entities::compliance::AuditAction::Created,
                        entity_type: "Order".into(),
                        entity_id: order.id,
                        old_value: None,
                        new_value: Some(serde_json::to_value(&order).unwrap()),
                        ip_address: None,
                    },
                )
                .await;

            Ok(order)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOrderDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Order>> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            // Get old value for audit
            let old_order =
                sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let order = sqlx::query_as::<_, Order>(
                r#"UPDATE orders SET label = COALESCE($1, label), updated_at = NOW()
                   WHERE id = $2 AND tenant_id = $3 RETURNING *"#,
            )
            .bind(&dto.label)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let (Some(old), Some(new)) = (&old_order, &order) {
                let _ = audit_repo
                    .create(
                        tid,
                        agrocore_domain::entities::compliance::CreateAuditLogDto {
                            tenant_id: tid,
                            user_id: by,
                            action: agrocore_domain::entities::compliance::AuditAction::Updated,
                            entity_type: "Order".into(),
                            entity_id: new.id,
                            old_value: Some(serde_json::to_value(old).unwrap()),
                            new_value: Some(serde_json::to_value(new).unwrap()),
                            ip_address: None,
                        },
                    )
                    .await;
            }

            Ok(order)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        // Since delete doesn't have 'by' parameter in the trait yet, we might need to adjust the trait
        // or accept that delete audits don't have a user_id for now if called from here.
        // Actually AuditLogRepo::create needs a user_id.
        Box::pin(async move {
            sqlx::query("UPDATE orders SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_assigned_to_worker(&self, tid: Uuid, worker_id: Uuid) -> RepositoryFuture<Vec<Order>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Order>(
                "SELECT * FROM orders WHERE tenant_id = $1 AND assigned_to = $2",
            )
            .bind(tid)
            .bind(worker_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
