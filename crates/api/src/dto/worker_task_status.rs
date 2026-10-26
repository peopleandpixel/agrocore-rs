//! Worker task status DTOs

use agrocore_domain::entities::worker_task_status::{
    CreateWorkerTaskStatusDto as DomainCreateWorkerTaskStatusDto, WorkerTaskStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub resumed_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub done_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkerTaskStatus> for WorkerTaskStatusDto {
    fn from(e: WorkerTaskStatus) -> Self {
        Self {
            task_id: e.task_id,
            worker_id: e.worker_id,
            tenant_id: e.tenant_id.into(),
            status: e.status_id.to_string(),
            started_at: e.started_at,
            paused_at: e.paused_at,
            resumed_at: e.resumed_at,
            stopped_at: e.stopped_at,
            done_at: e.done_at,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWorkerTaskStatusResponse {
    pub data: Vec<WorkerTaskStatusDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWorkerTaskStatusDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub tenant_id: Uuid,
}

impl From<CreateWorkerTaskStatusDto> for DomainCreateWorkerTaskStatusDto {
    fn from(dto: CreateWorkerTaskStatusDto) -> Self {
        Self {
            task_id: dto.task_id,
            worker_id: dto.worker_id,
            tenant_id: agrocore_domain::entities::tenant::TenantId(dto.tenant_id),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateWorkerTaskStatusDto {
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkerTaskStatusAggregateDto {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub status: String,
    pub duration_seconds: Option<i64>,
}
