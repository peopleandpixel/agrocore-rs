use agrocore_domain::TenantId;
use agrocore_domain::entities::order::{Order, OrderStatus, OrderType, WorkflowConfig};
use agrocore_domain::services::workflow::WorkflowService;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_process_status_transition_no_config() {
    let order = Order {
        id: Uuid::new_v4(),
        tenant_id: TenantId(Uuid::new_v4()),
        label: "Test".into(),
        order_type: OrderType::Harvest,
        status: OrderStatus::InProgress,
        site_ids: vec![],
        assigned_worker_ids: vec![],
        planned_date: None,
        deadline_date: None,
        started_at: None,
        completed_at: None,
        last_completed_at: None,
        recurrence: None,
        execution_policy: None,
        automation_state: None,
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
        tenant_id: TenantId(Uuid::new_v4()),
        label: "Harvest 2024".into(),
        order_type: OrderType::Harvest,
        status: OrderStatus::InProgress,
        site_ids: vec![Uuid::new_v4()],
        assigned_worker_ids: vec![Uuid::new_v4()],
        planned_date: None,
        deadline_date: None,
        started_at: None,
        completed_at: None,
        last_completed_at: None,
        recurrence: None,
        execution_policy: None,
        automation_state: None,
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

#[test]
fn test_process_status_transition_same_status_returns_none() {
    let order = Order {
        id: Uuid::new_v4(),
        tenant_id: TenantId(Uuid::new_v4()),
        label: "No-op".into(),
        order_type: OrderType::Harvest,
        status: OrderStatus::Completed,
        site_ids: vec![Uuid::new_v4()],
        assigned_worker_ids: vec![Uuid::new_v4()],
        planned_date: None,
        deadline_date: None,
        started_at: None,
        completed_at: None,
        last_completed_at: None,
        recurrence: None,
        execution_policy: None,
        automation_state: None,
        articles: None,
        quantities: None,
        results: None,
        weather: None,
        custom_fields: None,
        parent_order_id: None,
        workflow_config: Some(WorkflowConfig {
            auto_next_order_type: Some(OrderType::Fertilization),
            delay_days: Some(1),
            trigger_status: Some(OrderStatus::Completed),
        }),
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
fn test_process_status_transition_without_matching_trigger_returns_none() {
    let order = Order {
        id: Uuid::new_v4(),
        tenant_id: TenantId(Uuid::new_v4()),
        label: "No follow up".into(),
        order_type: OrderType::Harvest,
        status: OrderStatus::InProgress,
        site_ids: vec![Uuid::new_v4()],
        assigned_worker_ids: vec![Uuid::new_v4()],
        planned_date: None,
        deadline_date: None,
        started_at: None,
        completed_at: None,
        last_completed_at: None,
        recurrence: None,
        execution_policy: None,
        automation_state: None,
        articles: None,
        quantities: None,
        results: None,
        weather: None,
        custom_fields: None,
        parent_order_id: None,
        workflow_config: Some(WorkflowConfig {
            auto_next_order_type: None,
            delay_days: None,
            trigger_status: Some(OrderStatus::Completed),
        }),
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
fn order_type_display_covers_livestock_tasks() {
    assert_eq!(OrderType::LivestockFeeding.to_string(), "Fütterung");
    assert_eq!(OrderType::LivestockWatering.to_string(), "Wasser geben");
    assert_eq!(OrderType::LivestockRelocation.to_string(), "Verlegung");
    assert_eq!(
        OrderType::LivestockHealthCheck.to_string(),
        "Gesundheitskontrolle"
    );
    assert_eq!(OrderType::BarnCleaning.to_string(), "Stallreinigung");
    assert_eq!(OrderType::EggCollection.to_string(), "Eier holen");
    assert_eq!(OrderType::Shearing.to_string(), "Scheren");
    assert_eq!(OrderType::Milking.to_string(), "Melken");
}
