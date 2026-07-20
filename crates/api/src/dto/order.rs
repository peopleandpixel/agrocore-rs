//! Order DTOs

use agrocore_domain::entities::order::{
    CreateOrderDto as DomainCreateOrderDto, Order, RecurrenceRule, TaskExecutionPolicy,
    UpdateOrderDto as DomainUpdateOrderDto,
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
