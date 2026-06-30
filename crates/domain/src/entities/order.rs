use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::{OrderStatus, OrderType, user::UserRole};
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use mongodb::bson::{doc, Document};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct WorkflowConfig {
    pub auto_next_order_type: Option<OrderType>,
    pub delay_days: Option<u32>,
    pub trigger_status: Option<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub results: Option<String>,
    pub weather: Option<WeatherInfo>,
    pub custom_fields: Option<serde_json::Value>,
    pub parent_order_id: Option<Uuid>,
    pub workflow_config: Option<WorkflowConfig>,
    pub cost_center_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Order {
    pub fn can_transition_to(&self, next_status: OrderStatus) -> bool {
        match (self.status.clone(), next_status) {
            (OrderStatus::Draft, OrderStatus::Planned) => true,
            (OrderStatus::Draft, OrderStatus::Cancelled) => true,
            (OrderStatus::Planned, OrderStatus::InProgress) => true,
            (OrderStatus::Planned, OrderStatus::Cancelled) => true,
            (OrderStatus::InProgress, OrderStatus::Completed) => true,
            (OrderStatus::InProgress, OrderStatus::Cancelled) => true,
            (OrderStatus::Completed, _) => false, // Final state
            (OrderStatus::Cancelled, _) => false, // Final state
            (curr, next) if curr == next => true,
            _ => false,
        }
    }

    pub fn start(&mut self) -> bool {
        if self.can_transition_to(OrderStatus::InProgress) {
            self.status = OrderStatus::InProgress;
            self.started_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    pub fn complete(&mut self) -> bool {
        if self.can_transition_to(OrderStatus::Completed) {
            self.status = OrderStatus::Completed;
            self.completed_at = Some(Utc::now());
            true
        } else {
            false
        }
    }
}

impl VisibilityAwareEntity for Order {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(user_id: Uuid, roles: &[UserRole]) -> Document {
        if roles.contains(&UserRole::Admin) || roles.contains(&UserRole::Manager) {
            doc! {}
        } else {
            // Worker can only see orders assigned to them
            doc! { "assigned_worker_ids": user_id.to_string() }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrderArticle {
    pub article_id: Uuid,
    pub label: String,
    pub quantity_per_hectare: f64,
    pub unit: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: OrderType,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
    pub parent_order_id: Option<Uuid>,
    pub workflow_config: Option<WorkflowConfig>,
    pub cost_center_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateOrderDto {
    pub label: Option<String>,
    pub status: Option<OrderStatus>,
    pub site_ids: Option<Vec<Uuid>>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub results: Option<String>,
    pub weather: Option<WeatherInfo>,
    pub custom_fields: Option<serde_json::Value>,
    pub parent_order_id: Option<Uuid>,
    pub workflow_config: Option<WorkflowConfig>,
    pub cost_center_id: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTask {
    pub order_id: Uuid,
    pub label: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub site_count: u32,
    pub total_area: f64,
    pub deadline_date: Option<DateTime<Utc>>,
    pub planned_date: Option<DateTime<Utc>>,
}
