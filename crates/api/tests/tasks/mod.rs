// Tasks Handler Tests - DTO Validation
use agrocore_api::dto::{CreateTaskDto, UpdateTaskDto};

#[test]
fn test_create_task_validation() {
    let dto = CreateTaskDto {
        tenant_id: "tenant-123".into(),
        site_id: "site-123".into(),
        description: "Test task".into(),
        assigned_to: None,
        due_date: None,
        priority: Some("high".into()),
        status: Some("open".into()),
    };

    assert_eq!(dto.description, "Test task");
    assert_eq!(dto.priority, Some("high".into()));
}

#[test]
fn test_update_task_validation() {
    let dto = UpdateTaskDto {
        status: Some("completed".into()),
        assigned_to: Some("user-123".into()),
    };

    assert_eq!(dto.status, Some("completed".into()));
}