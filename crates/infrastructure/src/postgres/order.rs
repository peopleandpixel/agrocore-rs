use agrocore_domain::entities::order::{CreateOrderDto, Order, UpdateOrderDto, OrderType};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, OrderRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
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
            sqlx::query_as::<_, (Order,)>("SELECT row_to_json(orders) FROM orders WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(o,)| o)
                .ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<Order>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Order>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Order> = sqlx::query_as("SELECT * FROM orders WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            
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

    fn create(&self, tid: TenantId, dto: CreateOrderDto, _by: Uuid) -> RepositoryFuture<Order> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            sqlx::query_as::<_, Order>(
                r#"INSERT INTO orders (id, tenant_id, label, order_type, deadline_date, planned_date, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.label)
            .bind(format!("{:?}", dto.order_type))
            .bind(dto.deadline_date)
            .bind(dto.planned_date)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOrderDto, _by: Uuid) -> RepositoryFuture<Option<Order>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, Order>(
                r#"UPDATE orders SET
                    label = COALESCE($1, label),
                    order_type = COALESCE($2, order_type),
                    deadline_date = COALESCE($3, deadline_date),
                    planned_date = COALESCE($4, planned_date),
                    updated_at = $5
                   WHERE id = $6 AND tenant_id = $7
                   RETURNING *"#)
            .bind(&dto.label)
            .bind(dto.order_type.as_ref().map(|t| format!("{:?}", t)))
            .bind(dto.deadline_date)
            .bind(dto.planned_date)
            .bind(now)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("UPDATE orders SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
                .bind(Utc::now())
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }
}