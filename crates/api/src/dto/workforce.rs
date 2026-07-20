//! Worker Task Status DTOs

use agrocore_domain::entities::worker_task_status::{
    CreateWorkerTaskStatusDto as DomainCreateWorkerTaskStatusDto, WorkerTaskStatus,
    WorkerTaskStatusType,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWorkerTaskStatusResponse {
    pub data: Vec<WorkerTaskStatusDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
    pub status: WorkerTaskStatusTypeDto,
    pub started_at: Option<String>,
    pub paused_at: Option<String>,
    pub resumed_at: Option<String>,
    pub stopped_at: Option<String>,
    pub done_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<WorkerTaskStatus> for WorkerTaskStatusDto {
    fn from(w: WorkerTaskStatus) -> Self {
        Self {
            task_id: w.task_id,
            worker_id: w.worker_id,
            tenant_id: w.tenant_id.into(),
            status: w.status.into(),
            started_at: w.started_at.map(|d| d.to_rfc3339()),
            paused_at: w.paused_at.map(|d| d.to_rfc3339()),
            resumed_at: w.resumed_at.map(|d| d.to_rfc3339()),
            stopped_at: w.stopped_at.map(|d| d.to_rfc3339()),
            done_at: w.done_at.map(|d| d.to_rfc3339()),
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerTaskStatusTypeDto {
    New,
    Started,
    Paused,
    Stopped,
    Done,
}

impl From<WorkerTaskStatusTypeDto> for WorkerTaskStatusType {
    fn from(s: WorkerTaskStatusTypeDto) -> Self {
        match s {
            WorkerTaskStatusTypeDto::New => Self::New,
            WorkerTaskStatusTypeDto::Started => Self::Started,
            WorkerTaskStatusTypeDto::Paused => Self::Paused,
            WorkerTaskStatusTypeDto::Stopped => Self::Stopped,
            WorkerTaskStatusTypeDto::Done => Self::Done,
        }
    }
}

impl From<WorkerTaskStatusType> for WorkerTaskStatusTypeDto {
    fn from(s: WorkerTaskStatusType) -> Self {
        match s {
            WorkerTaskStatusType::New => Self::New,
            WorkerTaskStatusType::Started => Self::Started,
            WorkerTaskStatusType::Paused => Self::Paused,
            WorkerTaskStatusType::Stopped => Self::Stopped,
            WorkerTaskStatusType::Done => Self::Done,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorkerTaskStatusDto {
    pub status: WorkerTaskStatusTypeDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusAggregateDto {
    pub task_id: Uuid,
    pub aggregated_status: WorkerTaskStatusTypeDto,
    pub worker_statuses: Vec<WorkerTaskStatusDto>,
}
