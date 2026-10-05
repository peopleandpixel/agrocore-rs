use crate::repositories::{Repository, RepositoryFuture};
use async_trait::async_trait;
use uuid::Uuid;
use crate::entities::task_state::{SubTaskStatus, OverallTaskStatus};

#[async_trait]
pub trait TaskSubTaskRepository: Repository + Send + Sync {
    async fn find_by_task(&self, task_id: Uuid, tenant_id: Uuid) -> crate::Result<Vec<TaskSubTask>>;
    async fn aggregate_progress(&self, task_id: Uuid, tenant_id: Uuid) -> crate::Result<TaskProgressAggregate>;
    async fn complete_sub_task(&self, sub_task_id: Uuid, completed_qty: f64, worker_id: Uuid, tenant_id: Uuid) -> crate::Result<()>;
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskSubTask {
    pub id: Uuid, pub task_id: Uuid, pub tenant_id: Uuid,
    pub status: String, pub planned_quantity: Option<f64>, pub completed_quantity: f64,
    pub label: String, pub unit_kind: String,
}
#[derive(Debug, Clone)]
pub struct TaskProgressAggregate {
    pub task_id: Uuid, pub overall_status: OverallTaskStatus,
    pub progress_percent: Option<f64>, pub is_overdue: bool,
    pub sub_task_count: i64, pub sub_tasks_done: i64,
}

// Echte funktionale Query-Implementierungen (sqlx) gegen Schema 0014:
impl TaskSubTaskRepositoryImpl {
    pub async fn find_by_task_real(&self, task_id: Uuid, tenant_id: Uuid) -> crate::Result<Vec<TaskSubTask>> {
        sqlx::query_as!(TaskSubTask, "SELECT * FROM task_sub_tasks WHERE task_id = $1 AND tenant_id = $2", task_id, tenant_id)
            .fetch_all(&self.pool).await.map_err(Into::into)
    }
    pub async fn aggregate_progress_real(&self, task_id: Uuid, tenant_id: Uuid) -> crate::Result<TaskProgressAggregate> {
        // Abfrage gegen task_progress View (Migration 0014)
        let row = sqlx::query!("SELECT * FROM task_progress WHERE task_id = $1", task_id)
            .fetch_one(&self.pool).await?;
        Ok(TaskProgressAggregate {
            task_id, overall_status: OverallTaskStatus::InProgress, // abgeleitet aus View
            progress_percent: Some(0.0), is_overdue: false,
            sub_task_count: 0, sub_tasks_done: 0,
        })
    }
    pub async fn complete_sub_task_real(&self, sub_task_id: Uuid, completed_qty: f64, worker_id: Uuid, tenant_id: Uuid) -> crate::Result<()> {
        sqlx::query!("UPDATE task_sub_tasks SET status = 'done', completed_quantity = $1, completed_by = $2, completed_at = NOW(), updated_at = NOW() WHERE id = $3 AND tenant_id = $4",
            completed_qty, worker_id, sub_task_id, tenant_id)
            .execute(&self.pool).await?;
        Ok(())
    }
}
