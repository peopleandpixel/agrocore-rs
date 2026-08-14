use crate::dto::{
    CreateOrderDto, CreateSiteDto, CreateTaskDataDto, CreateUserDto, IoTCommandRequestDto,
};
use agrocore_domain::entities::site::GeoPoint;
use agrocore_domain::entities::{CropType, OrderType, SiteType};
use uuid::Uuid;
use validator::Validate;

#[test]
fn test_api_site_dto_validation() {
    let mut dto = CreateSiteDto {
        label: "".to_string(),
        site_type: SiteType::Field,
        crop_type: CropType::Grape,
        variety: None,
        area: -1.0,
        gross_area: None,
        center: Some(GeoPoint {
            lng: 14.0,
            lat: 47.0,
        }),
        boundary: Some(vec![
            GeoPoint {
                lng: 14.0,
                lat: 47.0,
            },
            GeoPoint {
                lng: 14.1,
                lat: 47.0,
            },
            GeoPoint {
                lng: 14.1,
                lat: 47.1,
            },
        ]),
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
        planned_date: None,
        deadline_date: None,
        recurrence: None,
        execution_policy: None,
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
        internal_cost_per_hour: None,
        external_cost_per_hour: None,
        language: None,
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
        started_at: None,
        ended_at: None,
        paused_at: None,
        pause_reason: None,
        duration_minutes: None,
        machine_id: None,
        machine_hours: None,
        cost_center_id: None,
        area_covered: None,
        materials_used: None,
        observations: None,
        gps_track: None,
        photo_urls: None,
    };
    assert!(dto.validate().is_err());

    dto.description = "Working on the field".to_string();
    assert!(dto.validate().is_ok());
}

#[test]
fn test_iot_command_request_dto_validation() {
    // Empty command_type fails validation
    let dto = IoTCommandRequestDto {
        command_type: "".to_string(),
        payload: serde_json::Value::Null,
        timeout_seconds: None,
    };
    assert!(dto.validate().is_err());

    // Valid command_type passes
    let dto = IoTCommandRequestDto {
        command_type: "reboot".to_string(),
        payload: serde_json::json!({"force": true}),
        timeout_seconds: Some(30),
    };
    assert!(dto.validate().is_ok());

    // timeout_seconds out of range fails
    let dto = IoTCommandRequestDto {
        command_type: "reboot".to_string(),
        payload: serde_json::Value::Null,
        timeout_seconds: Some(999),
    };
    assert!(dto.validate().is_err());

    // timeout_seconds within range passes
    let dto = IoTCommandRequestDto {
        command_type: "firmware_update".to_string(),
        payload: serde_json::Value::Null,
        timeout_seconds: Some(120),
    };
    assert!(dto.validate().is_ok());
}
