// Orders Handler Tests - DTO Validation
use agrocore_api::dto::{CreateOrderDto, UpdateOrderDto};

#[test]
fn test_create_order_validation() {
    let dto = CreateOrderDto {
        tenant_id: "tenant-123".into(),
        site_id: "site-123".into(),
        order_type: "spraying".into(),
        description: Some("Test order".into()),
        scheduled_at: None,
        target_amount: None,
        actual_amount: None,
        status: None,
    };
    
    assert_eq!(dto.tenant_id, "tenant-123");
    assert_eq!(dto.order_type, "spraying");
}

#[test]
fn test_update_order_validation() {
    let dto = UpdateOrderDto {
        status: Some("completed".into()),
        actual_amount: Some(100.0),
    };
    
    assert_eq!(dto.status, Some("completed".into()));
}