use crate::repositories::{RepositoryFuture};
use crate::entities::tenant::TenantId;
#[cfg(feature = "mocks")]
use mockall::automock;
use uuid::Uuid;
use crate::entities::task_state::{SubTaskStatus, OverallTaskStatus};

#[cfg_attr(feature = "mocks", automock)]
pub trait TaskSubTaskRepository: Send + Sync {
    fn find_by_task(&self, task_id: Uuid, tenant_id: TenantId) -> RepositoryFuture<Vec<TaskSubTask>>;
    fn aggregate_progress(&self, task_id: Uuid, tenant_id: TenantId) -> RepositoryFuture<TaskProgressAggregate>;
    fn complete_sub_task(&self, sub_task_id: Uuid, completed_qty: f64, worker_id: Uuid, tenant_id: TenantId) -> RepositoryFuture<()>;
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
