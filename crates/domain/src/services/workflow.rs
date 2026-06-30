use chrono::{Duration, Utc};
use crate::entities::order::{Order, CreateOrderDto};
use crate::entities::OrderStatus;

pub struct WorkflowService;

impl WorkflowService {
    pub fn process_status_transition(order: &Order, next_status: OrderStatus) -> Vec<CreateOrderDto> {
        let mut follow_up_orders = Vec::new();

        if order.status == next_status {
            return follow_up_orders;
        }

        if let Some(config) = &order.workflow_config {
            if let Some(trigger) = &config.trigger_status {
                if *trigger == next_status {
                    if let Some(next_type) = &config.auto_next_order_type {
                        let mut next_dto = CreateOrderDto {
                            label: format!("Folgeauftrag ({}): {}", next_type.to_string(), order.label),
                            order_type: next_type.clone(),
                            site_ids: order.site_ids.clone(),
                            assigned_worker_ids: Some(order.assigned_worker_ids.clone()),
                            planned_date: None,
                            deadline_date: None,
                            articles: None,
                            quantities: None,
                            custom_fields: None,
                            parent_order_id: Some(order.id),
                            workflow_config: None, // Prevent infinite loops or chain them
                            cost_center_id: order.cost_center_id,
                        };

                        if let Some(delay) = config.delay_days {
                            let planned = Utc::now() + Duration::days(delay as i64);
                            next_dto.planned_date = Some(planned);
                        }

                        follow_up_orders.push(next_dto);
                    }
                }
            }
        }

        follow_up_orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::OrderType;
    use crate::entities::order::{Order, WorkflowConfig, CreateOrderDto};
    use crate::entities::OrderStatus;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_process_status_transition_no_config() {
        let order = Order {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            label: "Test".into(),
            order_type: OrderType::Harvest,
            status: OrderStatus::InProgress,
            site_ids: vec![],
            assigned_worker_ids: vec![],
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
        };

        let follow_ups = WorkflowService::process_status_transition(&order, OrderStatus::Completed);
        assert!(follow_ups.is_empty());
    }

    #[test]
    fn test_process_status_transition_with_trigger() {
        let config = WorkflowConfig {
            auto_next_order_type: Some(OrderType::Fertilization),
            delay_days: Some(2),
            trigger_status: Some(OrderStatus::Completed),
        };

        let order = Order {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            label: "Harvest 2024".into(),
            order_type: OrderType::Harvest,
            status: OrderStatus::InProgress,
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
            workflow_config: Some(config),
            cost_center_id: Some(Uuid::new_v4()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            updated_by: None,
        };

        let follow_ups = WorkflowService::process_status_transition(&order, OrderStatus::Completed);
        assert_eq!(follow_ups.len(), 1);
        let next = &follow_ups[0];
        // We use the to_string implementation we just added or it was already there
        // Actually to_string for Fertilization returns "Düngung"
        assert!(next.label.contains("Düngung"));
        assert!(next.label.contains("Harvest 2024"));
        assert_eq!(next.order_type, OrderType::Fertilization);
        assert_eq!(next.site_ids, order.site_ids);
        assert_eq!(next.parent_order_id, Some(order.id));
        assert!(next.planned_date.is_some());
    }
}

impl ToString for crate::entities::OrderType {
    fn to_string(&self) -> String {
        match self {
            crate::entities::OrderType::PlantProtection => "Pflanzenschutz".to_string(),
            crate::entities::OrderType::Fertilization => "Düngung".to_string(),
            crate::entities::OrderType::Pruning => "Schnitt".to_string(),
            crate::entities::OrderType::Harvest => "Ernte".to_string(),
            crate::entities::OrderType::SoilWork => "Bodenbearbeitung".to_string(),
            crate::entities::OrderType::Irrigation => "Bewässerung".to_string(),
            crate::entities::OrderType::Monitoring => "Monitoring".to_string(),
            crate::entities::OrderType::Other(s) => s.clone(),
        }
    }
}
