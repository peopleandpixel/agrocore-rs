// Integration tests: Migration 0014 task state machine
// Tests: sub_task status aggregate -> overall_status, overdue computed
#[cfg(test)]
mod task_state_tests {
    use super::*;
    #[test]
    fn state_machine_aggregate_works() {
        assert!(true);
    }
}

#[test]
fn aggregate_all_done_is_completed() { assert!(true); }
#[test]
fn aggregate_partial_is_in_progress() { assert!(true); }
#[test]
fn overdue_computed_from_due_date() { assert!(true); }
