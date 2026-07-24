//! Order DTOs

use agrocore_domain::entities::order::{Order, RecurrenceRule, TaskExecutionPolicy};
use agrocore_domain::entities::task::{
    CreateTaskDataDto as DomainCreateTaskDataDto, GpsPoint, MaterialUsage, TaskData,
    UpdateTaskDataDto as DomainUpdateTaskDataDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedOrderResponse {
    pub data: Vec<OrderDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub order_type: agrocore_domain::entities::OrderType,
    pub status: agrocore_domain::entities::OrderStatus,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub last_completed_at: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Order> for OrderDto {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            tenant_id: o.tenant_id.into(),
            label: o.label,
            order_type: o.order_type,
            status: o.status,
            site_ids: o.site_ids,
            assigned_worker_ids: o.assigned_worker_ids,
            planned_date: o.planned_date.map(|d| d.to_rfc3339()),
            deadline_date: o.deadline_date.map(|d| d.to_rfc3339()),
            started_at: o.started_at.map(|d| d.to_rfc3339()),
            completed_at: o.completed_at.map(|d| d.to_rfc3339()),
            last_completed_at: o.last_completed_at.map(|d| d.to_rfc3339()),
            recurrence: o.recurrence,
            execution_policy: o.execution_policy,
            is_active: o.is_active,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: agrocore_domain::entities::OrderType,
    #[validate(length(min = 1))]
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
}

impl From<CreateOrderDto> for agrocore_domain::entities::order::CreateOrderDto {
    fn from(dto: CreateOrderDto) -> Self {
        Self {
            label: dto.label,
            order_type: dto.order_type,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
            planned_date: dto.planned_date,
            deadline_date: dto.deadline_date,
            articles: None,
            quantities: None,
            custom_fields: None,
            parent_order_id: None,
            workflow_config: None,
            recurrence: dto.recurrence,
            execution_policy: dto.execution_policy,
            cost_center_id: None,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub status: Option<agrocore_domain::entities::OrderStatus>,
    #[validate(length(min = 1))]
    pub site_ids: Option<Vec<Uuid>>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline_date: Option<chrono::DateTime<chrono::Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub is_active: Option<bool>,
}

impl From<UpdateOrderDto> for agrocore_domain::entities::order::UpdateOrderDto {
    fn from(dto: UpdateOrderDto) -> Self {
        Self {
            label: dto.label,
            status: dto.status,
            site_ids: dto.site_ids,
            assigned_worker_ids: dto.assigned_worker_ids,
            planned_date: dto.planned_date,
            deadline_date: dto.deadline_date,
            recurrence: dto.recurrence,
            execution_policy: dto.execution_policy,
            is_active: dto.is_active,
            ..Default::default()
        }
    }
}

// =============================================================================
// Task Data DTOs
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedTaskResponse {
    pub data: Vec<TaskDataDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskDataDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub worker_id: Uuid,
    pub order_id: Uuid,
    pub site_id: Uuid,
    pub description: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub paused_at: Option<String>,
    pub resume_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TaskData> for TaskDataDto {
    fn from(t: TaskData) -> Self {
        Self {
            id: t.id,
            tenant_id: t.tenant_id.into(),
            worker_id: t.worker_id,
            order_id: t.order_id,
            site_id: t.site_id,
            description: t.description,
            started_at: t.started_at.to_rfc3339(),
            ended_at: t.ended_at.map(|d| d.to_rfc3339()),
            paused_at: t.paused_at.map(|d| d.to_rfc3339()),
            resume_at: t.resume_at.map(|d| d.to_rfc3339()),
            duration_minutes: t.duration_minutes,
            machine_id: t.machine_id,
            machine_hours: t.machine_hours,
            cost_center_id: t.cost_center_id,
            area_covered: t.area_covered,
            materials_used: t.materials_used,
            observations: t.observations,
            gps_track: t.gps_track,
            photo_urls: t.photo_urls,
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateTaskDataDto {
    pub order_id: Uuid,
    pub site_id: Uuid,
    #[validate(length(min = 1))]
    pub description: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub paused_at: Option<String>,
    pub resume_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}

impl From<CreateTaskDataDto> for DomainCreateTaskDataDto {
    fn from(dto: CreateTaskDataDto) -> Self {
        Self {
            order_id: dto.order_id,
            site_id: dto.site_id,
            description: dto.description,
            started_at: dto.started_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            ended_at: dto.ended_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            paused_at: dto.paused_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            resume_at: dto.resume_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            duration_minutes: dto.duration_minutes,
            machine_id: dto.machine_id,
            machine_hours: dto.machine_hours,
            cost_center_id: dto.cost_center_id,
            area_covered: dto.area_covered,
            materials_used: dto
                .materials_used
                .map(|m| m.into_iter().collect()),
            observations: dto.observations,
            gps_track: dto
                .gps_track
                .map(|g| g.into_iter().collect()),
            photo_urls: dto.photo_urls,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateTaskDataDto {
    pub order_id: Option<Uuid>,
    pub site_id: Option<Uuid>,
    pub description: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub paused_at: Option<String>,
    pub resume_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub machine_id: Option<Uuid>,
    pub machine_hours: Option<f64>,
    pub cost_center_id: Option<Uuid>,
    pub area_covered: Option<f64>,
    pub materials_used: Option<Vec<MaterialUsage>>,
    pub observations: Option<String>,
    pub gps_track: Option<Vec<GpsPoint>>,
    pub photo_urls: Option<Vec<String>>,
}

impl From<UpdateTaskDataDto> for DomainUpdateTaskDataDto {
    fn from(dto: UpdateTaskDataDto) -> Self {
        Self {
            order_id: dto.order_id,
            site_id: dto.site_id,
            description: dto.description,
            started_at: dto.started_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            ended_at: dto.ended_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            paused_at: dto.paused_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            resume_at: dto.resume_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            duration_minutes: dto.duration_minutes,
            machine_id: dto.machine_id,
            machine_hours: dto.machine_hours,
            cost_center_id: dto.cost_center_id,
            area_covered: dto.area_covered,
            materials_used: dto
                .materials_used
                .map(|m| m.into_iter().collect()),
            observations: dto.observations,
            gps_track: dto
                .gps_track
                .map(|g| g.into_iter().collect()),
            photo_urls: dto.photo_urls,
        }
    }
}
