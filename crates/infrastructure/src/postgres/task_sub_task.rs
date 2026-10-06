use crate::postgres::tenant_pool::TenantPool;
use agrocore_domain::entities::task_state::{OverallTaskStatus, SubTaskStatus};
use agrocore_domain::repositories::{
    RepositoryFuture, TaskSubTaskRepository,
    TaskSubTask, TaskProgressAggregate,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgTaskSubTaskRepo);

impl TaskSubTaskRepository for PgTaskSubTaskRepo {
    fn find_by_task(
        &self,
        task_id: Uuid,
        tid: TenantId,
    ) -> RepositoryFuture<Vec<TaskSubTask>> {
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
}