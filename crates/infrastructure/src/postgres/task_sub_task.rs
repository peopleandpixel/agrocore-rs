use crate::postgres::tenant_pool::TenantPool;
use agrocore_domain::entities::task_state::{OverallTaskStatus, SubTaskStatus};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    NearbySubTask, RepositoryFuture, TaskProgressAggregate, TaskSubTask, TaskSubTaskRepository,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgTaskSubTaskRepo);

impl TaskSubTaskRepository for PgTaskSubTaskRepo {
    fn find_by_task(&self, task_id: Uuid, tid: TenantId) -> RepositoryFuture<Vec<TaskSubTask>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let tid_uuid = tid.0;
        Box::pin(async move {
            sqlx::query_as!(
                TaskSubTask,
                r#"
                SELECT
                    id, task_id, tenant_id,
                    status,
                    planned_quantity,
                    completed_quantity,
                    label,
                    unit_kind
                FROM task_sub_tasks
                WHERE task_id = $1 AND tenant_id = $2
                ORDER BY created_at
                "#,
                task_id,
                tid_uuid
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn aggregate_progress(
        &self,
        task_id: Uuid,
        tid: TenantId,
    ) -> RepositoryFuture<TaskProgressAggregate> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let tid_uuid = tid.0;
        Box::pin(async move {
            let row = sqlx::query!(
                r#"
                SELECT
                    overall_status as "overall_status: String",
                    sub_task_count,
                    sub_tasks_done,
                    progress_percent,
                    is_overdue
                FROM task_progress
                WHERE task_id = $1 AND tenant_id = $2
                "#,
                task_id,
                tid_uuid
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let overall_status = match row.overall_status.as_deref() {
                Some("completed") => OverallTaskStatus::Completed,
                Some("in_progress") => OverallTaskStatus::InProgress,
                Some("stopped") => OverallTaskStatus::Stopped,
                _ => OverallTaskStatus::New,
            };

            Ok(TaskProgressAggregate {
                task_id,
                overall_status,
                progress_percent: row.progress_percent,
                is_overdue: row.is_overdue.unwrap_or(false),
                sub_task_count: row.sub_task_count.unwrap_or(0),
                sub_tasks_done: row.sub_tasks_done.unwrap_or(0),
            })
        })
    }

    fn complete_sub_task(
        &self,
        sub_task_id: Uuid,
        completed_qty: f64,
        worker_id: Uuid,
        tid: TenantId,
    ) -> RepositoryFuture<()> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let tid_uuid = tid.0;
        Box::pin(async move {
            sqlx::query!(
                r#"
                UPDATE task_sub_tasks
                SET status = 'done',
                    completed_quantity = $1,
                    completed_by = $2,
                    completed_at = NOW(),
                    updated_at = NOW()
                WHERE id = $3 AND tenant_id = $4
                "#,
                completed_qty,
                worker_id,
                sub_task_id,
                tid_uuid
            )
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(())
        })
    }

    fn find_nearby(
        &self,
        tid: TenantId,
        lat: f64,
        lng: f64,
        radius_m: f64,
        limit: i64,
    ) -> RepositoryFuture<Vec<NearbySubTask>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let tid_uuid = tid.0;
        Box::pin(async move {
            let rows = sqlx::query!(
                r#"
                SELECT
                    st.id AS sub_task_id,
                    st.task_id,
                    st.label,
                    st.unit_kind,
                    st.status,
                    st.planned_quantity,
                    st.completed_quantity,
                    st.site_id,
                    s.label AS "site_label: Option<String>",
                    (COALESCE(ST_Distance(s.boundary::geography, ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography)::double precision, 0.0))::double precision AS distance_m
                FROM task_sub_tasks st
                LEFT JOIN sites s ON s.id = st.site_id AND s.tenant_id = st.tenant_id
                WHERE st.tenant_id = $1
                  AND st.status NOT IN ('done', 'stopped')
                  AND (s.boundary IS NOT NULL OR st.site_id IS NULL)
                  AND (st.site_id IS NULL OR ST_DWithin(s.boundary::geography, ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography, $4))
                ORDER BY distance_m
                LIMIT $5
                "#,
                tid_uuid,
                lng,
                lat,
                radius_m,
                limit
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let results = rows
                .into_iter()
                .map(|row| NearbySubTask {
                    sub_task_id: row.sub_task_id,
                    task_id: row.task_id,
                    label: row.label,
                    unit_kind: row.unit_kind,
                    status: row.status,
                    planned_quantity: row.planned_quantity,
                    completed_quantity: row.completed_quantity,
                    site_id: row.site_id,
                    site_label: row.site_label,
                    distance_m: row.distance_m,
                })
                .collect();

            Ok(results)
        })
    }

    fn complete_all_sub_tasks(
        &self,
        task_id: Uuid,
        worker_id: Uuid,
        tid: TenantId,
    ) -> RepositoryFuture<usize> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let tid_uuid = tid.0;
        Box::pin(async move {
            let rows_affected = sqlx::query!(
                r#"
                UPDATE task_sub_tasks
                SET status = 'done',
                    completed_quantity = planned_quantity,
                    completed_by = $1,
                    completed_at = NOW(),
                    updated_at = NOW()
                WHERE task_id = $2 AND tenant_id = $3 AND status NOT IN ('done', 'stopped')
                "#,
                worker_id,
                task_id,
                tid_uuid
            )
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?
            .rows_affected() as usize;
            Ok(rows_affected)
        })
    }
}
