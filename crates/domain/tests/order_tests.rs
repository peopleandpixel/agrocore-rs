use agrocore_domain::entities::order::{
    Order, OrderStatus, OrderType, RecurrenceCadence, RecurrenceRule, TaskAutomationAction,
    TaskExecutionMode, TaskExecutionPolicy,
};
use chrono::{Datelike, TimeZone, Utc};
use uuid::Uuid;

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

#[test]
fn recurring_order_reschedules_on_completion() {
    let completed_at = Utc
        .with_ymd_and_hms(2026, 7, 4, 6, 30, 0)
        .single()
        .expect("valid timestamp");
    let mut order = sample_order(OrderStatus::Planned);
    order.recurrence = Some(RecurrenceRule {
        cadence: RecurrenceCadence::Daily,
        every: 1,
        day_of_month: None,
        month: None,
    });

    let next_due = order.complete_recurring(completed_at);
    assert!(next_due.is_some());
    assert_eq!(order.status, OrderStatus::Planned);
    assert_eq!(order.last_completed_at, Some(completed_at));
    assert_eq!(order.completed_at, None);
    assert_eq!(order.planned_date, next_due);
}

#[test]
fn recurrence_rule_handles_monthly_and_yearly_dates() {
    let reference = Utc
        .with_ymd_and_hms(2026, 1, 31, 8, 0, 0)
        .single()
        .expect("valid timestamp");

    let monthly = RecurrenceRule {
        cadence: RecurrenceCadence::Monthly,
        every: 1,
        day_of_month: None,
        month: None,
    };
    let next_month = monthly.next_after(reference).expect("next monthly date");
    assert_eq!(next_month.year(), 2026);
    assert_eq!(next_month.month(), 2);
    assert_eq!(next_month.day(), 28);

    let yearly = RecurrenceRule {
        cadence: RecurrenceCadence::Yearly,
        every: 1,
        day_of_month: Some(15),
        month: Some(5),
    };
    let next_year = yearly.next_after(reference).expect("next yearly date");
    assert_eq!(next_year.year(), 2027);
    assert_eq!(next_year.month(), 5);
    assert_eq!(next_year.day(), 15);
}

#[test]
fn auto_presence_starts_and_completes_with_thresholds() {
    let mut order = sample_order(OrderStatus::Planned);
    order.execution_policy = Some(TaskExecutionPolicy {
        mode: TaskExecutionMode::AutoPresence,
        auto_start_after_minutes: Some(10),
        auto_end_after_minutes: Some(5),
        default_duration_minutes: None,
    });

    let t0 = Utc
        .with_ymd_and_hms(2026, 7, 4, 8, 0, 0)
        .single()
        .expect("valid timestamp");
    assert_eq!(order.observe_presence(true, t0), None);
    assert_eq!(order.status, OrderStatus::Planned);

    let t1 = t0 + chrono::Duration::minutes(10);
    assert_eq!(
        order.observe_presence(true, t1),
        Some(TaskAutomationAction::Started)
    );
    assert_eq!(order.status, OrderStatus::InProgress);

    let t2 = t1 + chrono::Duration::minutes(3);
    assert_eq!(order.observe_presence(false, t2), None);
    assert_eq!(order.status, OrderStatus::InProgress);

    let t3 = t2 + chrono::Duration::minutes(5);
    assert_eq!(
        order.observe_presence(false, t3),
        Some(TaskAutomationAction::Completed)
    );
    assert_eq!(order.status, OrderStatus::Completed);
    assert!(order.completed_at.is_some());
}

#[test]
fn manual_default_duration_allows_quick_completion() {
    let mut order = sample_order(OrderStatus::Planned);
    order.execution_policy = Some(TaskExecutionPolicy {
        mode: TaskExecutionMode::ManualWithDefaultDuration,
        auto_start_after_minutes: None,
        auto_end_after_minutes: None,
        default_duration_minutes: Some(5),
    });

    let completed_at = Utc
        .with_ymd_and_hms(2026, 7, 4, 9, 0, 0)
        .single()
        .expect("valid timestamp");
    assert_eq!(order.complete_at(completed_at), Some(5));
    assert_eq!(order.status, OrderStatus::Completed);
    assert_eq!(order.completed_at, Some(completed_at));
}