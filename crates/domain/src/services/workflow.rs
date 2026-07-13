use crate::entities::OrderStatus;
use crate::entities::order::{CreateOrderDto, Order};
use chrono::{Duration, Utc};

pub struct WorkflowService;

impl WorkflowService {
    pub fn process_status_transition(
        order: &Order,
        next_status: OrderStatus,
    ) -> Vec<CreateOrderDto> {
        let mut follow_up_orders = Vec::new();

        if order.status == next_status {
            return follow_up_orders;
        }

        if let Some(config) = &order.workflow_config
            && let Some(trigger) = &config.trigger_status
            && *trigger == next_status
            && let Some(next_type) = &config.auto_next_order_type
        {
            let mut next_dto = CreateOrderDto {
                label: format!("Folgeauftrag ({}): {}", next_type, order.label),
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
                recurrence: None,
                execution_policy: None,
                cost_center_id: order.cost_center_id,
            };

            if let Some(delay) = config.delay_days {
                let planned = Utc::now() + Duration::days(delay as i64);
                next_dto.planned_date = Some(planned);
            }

            follow_up_orders.push(next_dto);
        }

        follow_up_orders
    }
}

impl std::fmt::Display for crate::entities::OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            crate::entities::OrderType::PlantProtection => "Pflanzenschutz",
            crate::entities::OrderType::Fertilization => "Düngung",
            crate::entities::OrderType::Pruning => "Schnitt",
            crate::entities::OrderType::Harvest => "Ernte",
            crate::entities::OrderType::SoilWork => "Bodenbearbeitung",
            crate::entities::OrderType::Irrigation => "Bewässerung",
            crate::entities::OrderType::Monitoring => "Monitoring",
            crate::entities::OrderType::LivestockFeeding => "Fütterung",
            crate::entities::OrderType::LivestockWatering => "Wasser geben",
            crate::entities::OrderType::LivestockRelocation => "Verlegung",
            crate::entities::OrderType::LivestockHealthCheck => "Gesundheitskontrolle",
            crate::entities::OrderType::BarnCleaning => "Stallreinigung",
            crate::entities::OrderType::EggCollection => "Eier holen",
            crate::entities::OrderType::Shearing => "Scheren",
            crate::entities::OrderType::Milking => "Melken",
            crate::entities::OrderType::Other(s) => return f.write_str(s),
        };

        f.write_str(label)
    }
}