use agrocore_domain::entities::task::{
    CreateTaskDataDto, PauseResumeCycle, TaskData, UpdateTaskDataDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, TaskDataRepository,
};
use agrocore_shared::SharedError;
use chrono::{DateTime, Utc};
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
            sqlx::query_as::<_, TaskData>(
                "SELECT * FROM task_data WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
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
    ) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM task_data WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> =
                sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1 LIMIT $2 OFFSET $3")
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

    fn find_by_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM task_data WHERE tenant_id = $1 AND order_id = $2",
            )
            .bind(tid)
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> = sqlx::query_as(
                "SELECT * FROM task_data WHERE tenant_id = $1 AND order_id = $2 LIMIT $3 OFFSET $4",
            )
            .bind(tid)
            .bind(task_id)
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

    fn find_by_worker(
        &self,
        tid: TenantId,
        worker_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM task_data WHERE tenant_id = $1 AND worker_id = $2",
            )
            .bind(tid)
            .bind(worker_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<TaskData> = sqlx::query_as("SELECT * FROM task_data WHERE tenant_id = $1 AND worker_id = $2 LIMIT $3 OFFSET $4")
                .bind(tid)
                .bind(worker_id)
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

    fn create(
        &self,
        tid: TenantId,
        dto: CreateTaskDataDto,
        by: Uuid,
    ) -> RepositoryFuture<TaskData> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let started_at = dto.started_at.unwrap_or_else(Utc::now);
            sqlx::query_as::<_, TaskData>(
                r#"INSERT INTO task_data (
                    id, tenant_id, order_id, worker_id, site_id, description, 
                    started_at, ended_at, pause_resume_cycles, paused_at, duration_minutes,
                    machine_id, machine_hours, cost_center_id, area_covered,
                    materials_used, observations, gps_track, photo_urls,
                    finished_for_day_at, handoff_to_worker_id, is_session_complete,
                    created_at, updated_at
                   )
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, NOW(), NOW())
                   RETURNING *\"#)
            .bind(id)
            .bind(tid)
            .bind(dto.order_id)
            .bind(by)
            .bind(dto.site_id)
            .bind(dto.description)
            .bind(started_at)
            .bind(dto.ended_at)
            .bind(serde_json::to_value(Vec::<PauseResumeCycle>::new()).unwrap_or(serde_json::Value::Null))
            .bind(dto.paused_at)
            .bind(dto.duration_minutes)
            .bind(dto.machine_id)
            .bind(dto.machine_hours)
            .bind(dto.cost_center_id)
            .bind(dto.area_covered)
            .bind(serde_json::to_value(dto.materials_used).unwrap_or(serde_json::Value::Null))
            .bind(dto.observations)
            .bind(serde_json::to_value(dto.gps_track).unwrap_or(serde_json::Value::Null))
            .bind(serde_json::to_value(dto.photo_urls).unwrap_or(serde_json::Value::Null))
            .bind(Option::<DateTime<Utc>>::None)
            .bind(Option::<Uuid>::None)
            .bind(false)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateTaskDataDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<TaskData>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TaskData>(
                r#"UPDATE task_data SET
                    order_id = COALESCE($3, order_id),
                    site_id = COALESCE($4, site_id),
                    description = COALESCE($5, description),
                    started_at = COALESCE($6, started_at),
                    ended_at = COALESCE($7, ended_at),
                    paused_at = COALESCE($8, paused_at),
                    duration_minutes = COALESCE($9, duration_minutes),
                    machine_id = COALESCE($10, machine_id),
                    machine_hours = COALESCE($11, machine_hours),
                    cost_center_id = COALESCE($12, cost_center_id),
                    area_covered = COALESCE($13, area_covered),
                    materials_used = COALESCE($14, materials_used),
                    observations = COALESCE($15, observations),
                    gps_track = COALESCE($16, gps_track),
                    photo_urls = COALESCE($17, photo_urls),
                    handoff_to_worker_id = COALESCE($18, handoff_to_worker_id),
                    is_session_complete = COALESCE($19, is_session_complete),
                    finished_for_day_at = CASE WHEN $20 THEN COALESCE(finished_for_day_at, NOW()) ELSE finished_for_day_at END,
                    updated_at = NOW()
                   WHERE id = $1 AND tenant_id = $2
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.order_id)
            .bind(dto.site_id)
            .bind(dto.description)
            .bind(dto.started_at)
            .bind(dto.ended_at)
            .bind(dto.paused_at)
            .bind(dto.duration_minutes)
            .bind(dto.machine_id)
            .bind(dto.machine_hours)
            .bind(dto.cost_center_id)
            .bind(dto.area_covered)
            .bind(dto.materials_used.map(|v| serde_json::to_value(v).unwrap_or_default()))
            .bind(dto.observations)
            .bind(dto.gps_track.map(|v| serde_json::to_value(v).unwrap_or_default()))
            .bind(dto.photo_urls.map(|v| serde_json::to_value(v).unwrap_or_default()))
            .bind(dto.handoff_to_worker_id)
            .bind(dto.is_session_complete)
            .bind(dto.finish_for_day)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM task_data WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|result| result.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
