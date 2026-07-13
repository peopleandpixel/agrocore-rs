use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
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
use mongodb::bson::{Document, doc};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct WorkflowConfig {
    pub auto_next_order_type: Option<OrderType>,
    pub delay_days: Option<u32>,
    pub trigger_status: Option<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskExecutionMode {
    Manual,
    AutoPresence,
    ManualWithDefaultDuration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSchema)]
pub struct TaskAutomationState {
    pub inside_since: Option<DateTime<Utc>>,
    pub outside_since: Option<DateTime<Utc>>,
    pub last_presence_inside: Option<bool>,
    pub last_observed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Validate)]
pub struct TaskExecutionPolicy {
    pub mode: TaskExecutionMode,
    #[validate(range(min = 0, max = 10000))]
    pub auto_start_after_minutes: Option<u32>,
    #[validate(range(min = 0, max = 10000))]
    pub auto_end_after_minutes: Option<u32>,
    #[validate(range(min = 1, max = 10080))]
    pub default_duration_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceCadence {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Validate)]
pub struct RecurrenceRule {
    pub cadence: RecurrenceCadence,
    #[validate(range(min = 1, max = 1000))]
    pub every: u32,
    #[validate(range(min = 1, max = 31))]
    pub day_of_month: Option<u32>,
    #[validate(range(min = 1, max = 12))]
    pub month: Option<u32>,
}

impl RecurrenceRule {
    pub fn next_after(&self, reference: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self.cadence {
            RecurrenceCadence::Hourly => {
                Some(reference + chrono::Duration::hours(self.every as i64))
            }
            RecurrenceCadence::Daily => Some(reference + chrono::Duration::days(self.every as i64)),
            RecurrenceCadence::Weekly => {
                Some(reference + chrono::Duration::weeks(self.every as i64))
            }
            RecurrenceCadence::Monthly => {
                let total_months =
                    reference.year() * 12 + reference.month0() as i32 + self.every as i32;
                let target_year = total_months.div_euclid(12);
                let target_month = total_months.rem_euclid(12) as u32 + 1;
                let day = self
                    .day_of_month
                    .unwrap_or(reference.day())
                    .min(last_day_of_month(target_year, target_month));
                Utc.with_ymd_and_hms(
                    target_year,
                    target_month,
                    day,
                    reference.hour(),
                    reference.minute(),
                    reference.second(),
                )
                .single()
            }
            RecurrenceCadence::Yearly => {
                let target_year = reference.year() + self.every as i32;
                let target_month = self.month.unwrap_or(reference.month());
                let day = self
                    .day_of_month
                    .unwrap_or(reference.day())
                    .min(last_day_of_month(target_year, target_month));
                Utc.with_ymd_and_hms(
                    target_year,
                    target_month,
                    day,
                    reference.hour(),
                    reference.minute(),
                    reference.second(),
                )
                .single()
            }
        }
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_day_next_month = Utc
        .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .single()
        .expect("valid date");
    (first_day_next_month - chrono::Duration::days(1)).day()
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
    pub last_completed_at: Option<DateTime<Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub automation_state: Option<TaskAutomationState>,
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
        self.start_at(Utc::now())
    }

    pub fn start_at(&mut self, started_at: DateTime<Utc>) -> bool {
        if self.can_transition_to(OrderStatus::InProgress) {
            self.status = OrderStatus::InProgress;
            self.started_at = Some(started_at);
            if let Some(state) = self.automation_state.as_mut() {
                state.last_presence_inside = Some(true);
                state.last_observed_at = Some(started_at);
                state.outside_since = None;
            }
            true
        } else {
            false
        }
    }

    pub fn complete(&mut self) -> bool {
        self.complete_at(Utc::now()).is_some()
    }

    pub fn expected_work_duration_minutes(&self) -> Option<u32> {
        if let Some(started_at) = self.started_at {
            let elapsed = Utc::now() - started_at;
            return Some(elapsed.num_minutes().max(0) as u32);
        }
        self.execution_policy
            .as_ref()
            .and_then(|policy| policy.default_duration_minutes)
    }

    pub fn complete_at(&mut self, completed_at: DateTime<Utc>) -> Option<u32> {
        if self.complete_recurring(completed_at).is_some() {
            return self
                .execution_policy
                .as_ref()
                .and_then(|policy| policy.default_duration_minutes)
                .or_else(|| {
                    self.started_at.map(|started_at| {
                        completed_at
                            .signed_duration_since(started_at)
                            .num_minutes()
                            .max(0) as u32
                    })
                })
                .or(Some(0));
        }

        let duration_minutes = self
            .started_at
            .map(|started_at| {
                completed_at
                    .signed_duration_since(started_at)
                    .num_minutes()
                    .max(0) as u32
            })
            .or_else(|| {
                self.execution_policy
                    .as_ref()
                    .and_then(|policy| policy.default_duration_minutes)
            });

        if duration_minutes.is_none()
            && !matches!(self.status, OrderStatus::InProgress)
            && !matches!(
                self.execution_policy.as_ref().map(|policy| &policy.mode),
                Some(TaskExecutionMode::ManualWithDefaultDuration)
            )
        {
            return None;
        }

        self.status = OrderStatus::Completed;
        self.completed_at = Some(completed_at);
        self.started_at = self.started_at.or(Some(completed_at));
        self.updated_at = completed_at;
        if let Some(state) = self.automation_state.as_mut() {
            state.last_presence_inside = Some(false);
            state.last_observed_at = Some(completed_at);
            state.inside_since = None;
            state.outside_since = None;
        }
        duration_minutes.or(Some(0))
    }

    pub fn complete_recurring(&mut self, completed_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let recurrence = self.recurrence.as_ref()?;
        if !matches!(
            self.status,
            OrderStatus::Draft | OrderStatus::Planned | OrderStatus::InProgress
        ) {
            return None;
        }

        let next_due = recurrence.next_after(completed_at)?;
        self.status = OrderStatus::Planned;
        self.started_at = None;
        self.completed_at = None;
        self.last_completed_at = Some(completed_at);
        self.planned_date = Some(next_due);
        self.is_active = true;
        self.updated_at = completed_at;
        Some(next_due)
    }

    pub fn observe_presence(
        &mut self,
        inside: bool,
        observed_at: DateTime<Utc>,
    ) -> Option<TaskAutomationAction> {
        let policy = self.execution_policy.as_ref()?;
        if policy.mode != TaskExecutionMode::AutoPresence {
            return None;
        }

        let state = self
            .automation_state
            .get_or_insert_with(TaskAutomationState::default);
        state.last_observed_at = Some(observed_at);

        if inside {
            if state.last_presence_inside != Some(true) {
                state.inside_since = Some(observed_at);
                state.outside_since = None;
            }
            state.last_presence_inside = Some(true);

            let threshold = policy.auto_start_after_minutes.unwrap_or(0);
            let should_start = self.status != OrderStatus::InProgress
                && state
                    .inside_since
                    .map(|since| {
                        observed_at.signed_duration_since(since).num_minutes() >= threshold as i64
                    })
                    .unwrap_or(threshold == 0);
            if should_start && self.start_at(observed_at) {
                return Some(TaskAutomationAction::Started);
            }
            return None;
        }

        if state.last_presence_inside != Some(false) {
            state.outside_since = Some(observed_at);
            state.inside_since = None;
        }
        state.last_presence_inside = Some(false);

        let threshold = policy.auto_end_after_minutes.unwrap_or(0);
        let should_complete = self.status == OrderStatus::InProgress
            && state
                .outside_since
                .map(|since| {
                    observed_at.signed_duration_since(since).num_minutes() >= threshold as i64
                })
                .unwrap_or(threshold == 0);
        if should_complete && self.complete_at(observed_at).is_some() {
            return Some(TaskAutomationAction::Completed);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskAutomationAction {
    Started,
    Completed,
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
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
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
    pub last_completed_at: Option<DateTime<Utc>>,
    pub recurrence: Option<RecurrenceRule>,
    pub execution_policy: Option<TaskExecutionPolicy>,
    pub automation_state: Option<TaskAutomationState>,
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
}
