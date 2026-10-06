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
    /// Find open sub-tasks near a given position (lat/lng) within radius_m meters.
    /// Returns sub-tasks with their distance from the position, sorted by distance.
    fn find_nearby(
        &self,
        tenant_id: TenantId,
        lat: f64,
        lng: f64,
        radius_m: f64,
        limit: i64,
    ) -> RepositoryFuture<Vec<NearbySubTask>>;
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
/// A sub-task with its distance from a reference point.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NearbySubTask {
    pub sub_task_id: Uuid,
    pub task_id: Uuid,
    pub label: String,
    pub unit_kind: String,
    pub status: String,
    pub planned_quantity: Option<f64>,
    pub completed_quantity: f64,
    pub site_id: Option<Uuid>,
    pub site_label: Option<String>,
    pub distance_m: Option<f64>,
}
