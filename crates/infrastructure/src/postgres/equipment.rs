use agrocore_domain::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    EquipmentRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use chrono::Utc;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgEquipmentRepo {
    pool: PgPool,
}

impl PgEquipmentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl EquipmentRepository for PgEquipmentRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Equipment>(
                "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2",
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
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);

            if can_see_all {
                sqlx::query_as::<_, Equipment>(
                    "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2",
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, Equipment>(
                    "SELECT * FROM equipment WHERE id = $1 AND tenant_id = $2 AND assigned_to = $3",
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

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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
                data: items,
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
        roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        let roles_vec = roles.to_vec();

        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);

            if can_see_all {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

                let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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
                    data: items,
                    total: total as u64,
                    page,
                    per_page,
                    total_pages,
                })
            } else {
                let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

                let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY label LIMIT $2 OFFSET $3")
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
                    data: items,
                    total: total as u64,
                    page,
                    per_page,
                    total_pages,
                })
            }
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateEquipmentDto,
        _by: Uuid,
    ) -> RepositoryFuture<Equipment> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();
            let equipment_type =
                serde_json::to_string(&dto.equipment_type).unwrap_or_else(|_| "{}".to_string());
            let maintenance_intervals = dto
                .maintenance_intervals
                .map(|m| serde_json::to_string(&m).unwrap_or_else(|_| "[]".to_string()));

            let equipment = sqlx::query_as::<_, Equipment>(
                r#"INSERT INTO equipment (id, tenant_id, label, code, equipment_type, maintenance_intervals, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                   RETURNING *"#
            )
            .bind(id)
            .bind(tid)
            .bind(&dto.label)
            .bind(&dto.code)
            .bind(equipment_type)
            .bind(maintenance_intervals)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateEquipmentDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let equipment_type = dto
                .equipment_type
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string()));
            let maintenance_intervals = dto
                .maintenance_intervals
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()));
            let next_maintenance = dto.next_maintenance_date;

            let equipment = sqlx::query_as::<_, Equipment>(
                r#"UPDATE equipment SET 
                    label = COALESCE($1, label), 
                    code = COALESCE($2, code), 
                    equipment_type = COALESCE($3, equipment_type),
                    in_usage = COALESCE($4, in_usage),
                    maintenance_intervals = COALESCE($5, maintenance_intervals),
                    next_maintenance_date = COALESCE($6, next_maintenance_date),
                    last_maintenance_hours = COALESCE($7, last_maintenance_hours),
                    updated_at = $8
                   WHERE id = $9 AND tenant_id = $10
                   RETURNING *"#,
            )
            .bind(&dto.label)
            .bind(&dto.code)
            .bind(equipment_type)
            .bind(dto.in_usage)
            .bind(maintenance_intervals)
            .bind(next_maintenance)
            .bind(dto.last_maintenance_hours)
            .bind(now)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("UPDATE equipment SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
                .bind(Utc::now())
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
