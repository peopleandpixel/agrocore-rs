use agrocore_domain::entities::task::{CreateTaskDataDto, TaskData};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, TaskDataRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use serde_json;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

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
            let row = sqlx::query_as::<_, (TaskData,)>("SELECT row_to_json(task_data) FROM task_data WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            row.map(|(t,)| t).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<TaskData>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_data WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<TaskData> = sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1::uuid ORDER BY started_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    fn create(&self, tid: TenantId, wid: Uuid, dto: CreateTaskDataDto) -> RepositoryFuture<TaskData> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            let task = sqlx::query_as::<_, TaskData>(
                "INSERT INTO task_data (id, tenant_id, order_id, worker_id, site_id, description, started_at, ended_at, paused_at, resume_at, duration_minutes, machine_id, machine_hours, cost_center_id, area_covered, materials_used, observations, gps_track, photo_urls, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) \
                 RETURNING *"
            )
            .bind(id)
            .bind(tid.to_string())
            .bind(dto.order_id.to_string())
            .bind(wid.to_string())
            .bind(dto.site_id.to_string())
            .bind(&dto.description)
            .bind(dto.started_at.unwrap_or(now))
            .bind(dto.ended_at)
            .bind(dto.paused_at)
            .bind(dto.resume_at)
            .bind(dto.duration_minutes.map(|v| v as i32))
            .bind(dto.machine_id.map(|u| u.to_string()))
            .bind(dto.machine_hours)
            .bind(dto.cost_center_id.map(|u| u.to_string()))
            .bind(dto.area_covered)
            .bind(serde_json::to_string(&dto.materials_used).ok())
            .bind(&dto.observations)
            .bind(serde_json::to_string(&dto.gps_track).ok())
            .bind(serde_json::to_string(&dto.photo_urls).ok())
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(task)
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, _dto: CreateTaskDataDto, _by: Uuid) -> RepositoryFuture<Option<TaskData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let task = sqlx::query_as::<_, TaskData>(
                r#"UPDATE task_data SET updated_at = $1 WHERE id = $2 AND tenant_id = $3 RETURNING *"#
            )
            .bind(Utc::now())
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(task)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM task_data WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }
}