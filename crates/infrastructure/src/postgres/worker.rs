use agrocore_domain::entities::workforce::{Worker, CreateWorkerDto, UpdateWorkerDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{WorkerRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
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
            sqlx::query_as::<_, (Worker,)>("SELECT row_to_json(workers) FROM workers WHERE id = $1 AND tenant_id = $2 AND is_active = true")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(w,)| w)
                .ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Worker>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workers WHERE tenant_id = $1::uuid AND is_active = true")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Worker> = sqlx::query_as("SELECT * FROM workers WHERE tenant_id = $1::uuid AND is_active = true ORDER BY name LIMIT $2 OFFSET $3")
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

    fn create(&self, tid: TenantId, dto: CreateWorkerDto) -> RepositoryFuture<Worker> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            sqlx::query_as::<_, Worker>(
                r#"INSERT INTO workers (id, tenant_id, name, hourly_rate, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, true, $5, $6)
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.name)
            .bind(dto.hourly_rate)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateWorkerDto) -> RepositoryFuture<Option<Worker>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, Worker>(
                r#"UPDATE workers SET
                    name = COALESCE($1, name),
                    hourly_rate = COALESCE($2, hourly_rate),
                    updated_at = $3
                   WHERE id = $4 AND tenant_id = $5
                   RETURNING *"#)
            .bind(&dto.name)
            .bind(dto.hourly_rate)
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
            let result = sqlx::query("UPDATE workers SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
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