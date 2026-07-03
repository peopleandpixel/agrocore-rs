use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::{OrderStatus, OrderType};
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use crate::entities::user::UserRole;
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_order(status: OrderStatus) -> Order {
        Order {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            label: String::from("Test Order"),
            order_type: OrderType::Harvest,
            status,
            site_ids: vec![Uuid::new_v4()],
            assigned_worker_ids: vec![Uuid::new_v4()],
            planned_date: None,
            deadline_date: None,
            started_at: None,
            completed_at: None,
            articles: None,
            quantities: None,
            results: None,
            weather: None,
            custom_fields: None,
            parent_order_id: None,
            workflow_config: None,
            cost_center_id: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn order_transition_rules_cover_final_and_same_state() {
        let order = sample_order(OrderStatus::Draft);
        assert!(order.can_transition_to(OrderStatus::Planned));
        assert!(order.can_transition_to(OrderStatus::Cancelled));
        assert!(order.can_transition_to(OrderStatus::Draft));
        assert!(!order.can_transition_to(OrderStatus::Completed));

        let completed = sample_order(OrderStatus::Completed);
        assert!(!completed.can_transition_to(OrderStatus::Planned));
        assert!(!completed.can_transition_to(OrderStatus::Completed));
    }

    #[test]
    fn order_start_and_complete_update_timestamps_when_allowed() {
        let mut order = sample_order(OrderStatus::Planned);
        assert!(order.start());
        assert_eq!(order.status, OrderStatus::InProgress);
        assert!(order.started_at.is_some());

        assert!(order.complete());
        assert_eq!(order.status, OrderStatus::Completed);
        assert!(order.completed_at.is_some());
    }

    #[test]
    fn order_start_and_complete_fail_for_invalid_transitions() {
        let mut draft = sample_order(OrderStatus::Draft);
        assert!(!draft.complete());
        assert_eq!(draft.status, OrderStatus::Draft);

        let mut cancelled = sample_order(OrderStatus::Cancelled);
        assert!(!cancelled.start());
        assert_eq!(cancelled.status, OrderStatus::Cancelled);
    }
}
