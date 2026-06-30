use crate::dto::{CreateSiteDto, CreateOrderDto, CreateUserDto, CreateTaskDataDto};
use agrocore_domain::entities::{SiteType, CropType, OrderType};
use validator::Validate;
use uuid::Uuid;

#[test]
fn test_api_site_dto_validation() {
    let mut dto = CreateSiteDto {
        label: "".to_string(),
        site_type: SiteType::Field,
        crop_type: CropType::Grape,
        variety: None,
        area: -1.0,
        gross_area: None,
        plots: None,
        properties: None,
    };
    assert!(dto.validate().is_err());

    dto.label = "Valid Site".to_string();
    dto.area = 10.0;
    assert!(dto.validate().is_ok());
}

#[test]
fn test_api_order_dto_validation() {
    let mut dto = CreateOrderDto {
        label: "".to_string(),
        order_type: OrderType::Harvest,
        site_ids: vec![],
        assigned_worker_ids: None,
    };
    assert!(dto.validate().is_err());

    dto.label = "Harvest 2024".to_string();
    dto.site_ids = vec![Uuid::new_v4()];
    assert!(dto.validate().is_ok());
}

#[test]
fn test_api_user_dto_validation() {
    let mut dto = CreateUserDto {
        firstname: "".to_string(),
        lastname: "".to_string(),
        email: "bad-email".to_string(),
        password: "short".to_string(),
        roles: None,
    };
    assert!(dto.validate().is_err());

    dto.firstname = "Alice".to_string();
    dto.lastname = "Smith".to_string();
    dto.email = "alice@agrocore.io".to_string();
    dto.password = "secure-pass-123".to_string();
    assert!(dto.validate().is_ok());
}

#[test]
fn test_api_task_dto_validation() {
    let mut dto = CreateTaskDataDto {
        order_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        description: "".to_string(),
    };
    assert!(dto.validate().is_err());

    dto.description = "Working on the field".to_string();
    assert!(dto.validate().is_ok());
}
