use agrocore_domain::entities::task::{TaskData, CreateTaskDataDto, UpdateTaskDataDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, TaskDataRepository};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgTaskDataRepo {
    pool: PgPool,
}

impl PgTaskDataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaskDataRepository for PgTaskDataRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<TaskData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TaskData>("SELECT * FROM task_data WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_data WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> = sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
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

    fn find_by_task(&self, tid: TenantId, task_id: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_data WHERE tenant_id = $1::uuid AND order_id = $2::uuid")
                .bind(tid.to_string())
                .bind(task_id.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> = sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1::uuid AND order_id = $2::uuid LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(task_id.to_string())
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

    fn find_by_worker(&self, tid: TenantId, worker_id: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_data WHERE tenant_id = $1::uuid AND worker_id = $2::uuid")
                .bind(tid.to_string())
                .bind(worker_id.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> = sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1::uuid AND worker_id = $2::uuid LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(worker_id.to_string())
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

    fn create(&self, tid: TenantId, dto: CreateTaskDataDto, by: Uuid) -> RepositoryFuture<TaskData> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, TaskData>(
                r#"INSERT INTO task_data (
                    id, tenant_id, order_id, worker_id, site_id, description, 
                    started_at, ended_at, paused_at, resume_at, duration_minutes,
                    machine_id, machine_hours, cost_center_id, area_covered,
                    materials_used, observations, gps_track, photo_urls,
                    created_at, updated_at
                   )
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(dto.order_id)
            .bind(by)
            .bind(dto.site_id)
            .bind(dto.description)
            .bind(dto.started_at)
            .bind(dto.ended_at)
            .bind(dto.paused_at)
            .bind(dto.resume_at)
            .bind(dto.duration_minutes)
            .bind(dto.machine_id)
            .bind(dto.machine_hours)
            .bind(dto.cost_center_id)
            .bind(dto.area_covered)
            .bind(serde_json::to_value(dto.materials_used).unwrap_or(serde_json::Value::Null))
            .bind(dto.observations)
            .bind(serde_json::to_value(dto.gps_track).unwrap_or(serde_json::Value::Null))
            .bind(serde_json::to_value(dto.photo_urls).unwrap_or(serde_json::Value::Null))
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdateTaskDataDto, _by: Uuid) -> RepositoryFuture<Option<TaskData>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }

    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}
