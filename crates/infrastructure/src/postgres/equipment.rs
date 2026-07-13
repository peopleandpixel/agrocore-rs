use agrocore_domain::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, EquipmentRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

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
            let row = sqlx::query_as::<_, (Equipment,)>("SELECT row_to_json(equipment) FROM equipment WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            row.map(|(e,)| e).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, _user_id: Uuid, _roles: &[UserRole]) -> RepositoryFuture<Option<Equipment>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<Equipment> = sqlx::query_as("SELECT * FROM equipment WHERE tenant_id = $1::uuid ORDER BY label LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    fn find_all_visible(&self, tid: TenantId, p: Pagination, _user_id: Uuid, _roles: &[UserRole]) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        self.find_all(tid, p)
    }

    fn create(&self, tid: TenantId, dto: CreateEquipmentDto, _by: Uuid) -> RepositoryFuture<Equipment> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            let equipment = sqlx::query_as::<_, Equipment>(
                "INSERT INTO equipment (id, tenant_id, label, category, serial_number, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 RETURNING *"
            )
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.label)
            .bind(&dto.category)
            .bind(&dto.serial_number)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateEquipmentDto, _by: Uuid) -> RepositoryFuture<Option<Equipment>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let equipment = sqlx::query_as::<_, Equipment>(
                r#"UPDATE equipment SET label = COALESCE($1, label), category = COALESCE($2, category), updated_at = $3 WHERE id = $4 AND tenant_id = $5 RETURNING *"#
            )
            .bind(&dto.label)
            .bind(&dto.category)
            .bind(now)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(equipment)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM equipment WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }
}