use crate::entities::site::GeoPoint;
use validator::Validate;

#[test]
fn test_geopoint_validation() {
    let valid = GeoPoint { lng: 10.0, lat: 20.0 };
    assert!(valid.validate().is_ok());

    let invalid_lng = GeoPoint { lng: 190.0, lat: 20.0 };
    assert!(invalid_lng.validate().is_err());

    let invalid_lat = GeoPoint { lng: 10.0, lat: 100.0 };
    assert!(invalid_lat.validate().is_err());
}

#[test]
fn test_site_label_validation() {
    use crate::entities::site::CreateSiteDto;
    use crate::entities::{SiteType, CropType};

    let dto = CreateSiteDto {
        label: "".to_string(),
        site_type: SiteType::Field,
        crop_type: CropType::Grape,
        variety: None,
        area: 10.0,
        gross_area: None,
        plots: None,
        row_config: None,
        bbch_stage: None,
        planted_date: None,
        soil_type: None,
        slope: None,
        slope_facing: None,
        altitude: None,
        organic: None,
        center: None,
        sigpac_data: None,
        regepac_id: None,
        boundary: None,
        properties: None,
        custom_fields: None,
        note1: None,
        note2: None,
    };
    assert!(dto.validate().is_err());
    
    let mut valid_dto = dto.clone();
    valid_dto.label = "Valid Site".to_string();
    assert!(valid_dto.validate().is_ok());
}

#[test]
fn test_fertilizer_record_validation() {
    use crate::entities::compliance::CreateFertilizerRecordDto;
    use chrono::Utc;
    use uuid::Uuid;

    let dto = CreateFertilizerRecordDto {
        site_id: Uuid::new_v4(),
        order_id: None,
        product_name: "".to_string(), // Invalid: empty
        nutrient_n: -1.0, // Invalid: negative
        nutrient_p: 10.0,
        nutrient_k: 5.0,
        quantity_kg: 100.0,
        area_ha: 1.0,
        application_date: Utc::now(),
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.field_errors().contains_key("product_name"));
    assert!(errs.field_errors().contains_key("nutrient_n"));

    let mut valid_dto = dto.clone();
    valid_dto.product_name = "NitroPlus".to_string();
    valid_dto.nutrient_n = 20.0;
    assert!(valid_dto.validate().is_ok());
}

#[test]
fn test_order_validation() {
    use crate::entities::order::CreateOrderDto;
    use crate::entities::OrderType;
    use uuid::Uuid;

    let dto = CreateOrderDto {
        label: "".to_string(), // Invalid
        order_type: OrderType::Fertilization,
        site_ids: vec![], // Invalid: empty
        assigned_worker_ids: None,
        planned_date: None,
        deadline_date: None,
        articles: None,
        quantities: None,
        custom_fields: None,
        parent_order_id: None,
        workflow_config: None,
        cost_center_id: None,
    };
    assert!(dto.validate().is_err());

    let mut valid_dto = dto.clone();
    valid_dto.label = "Test Order".to_string();
    valid_dto.site_ids = vec![Uuid::new_v4()];
    assert!(valid_dto.validate().is_ok());
}

#[test]
fn test_user_validation() {
    use crate::entities::user::CreateUserDto;

    let dto = CreateUserDto {
        firstname: "John".to_string(),
        lastname: "Doe".to_string(),
        email: "invalid-email".to_string(), // Invalid
        password: "short".to_string(), // Invalid
        roles: None,
        internal_cost_per_hour: None,
        external_cost_per_hour: None,
        language: None,
    };
    assert!(dto.validate().is_err());

    let mut valid_dto = dto.clone();
    valid_dto.email = "john@example.com".to_string();
    valid_dto.password = "password123".to_string();
    assert!(valid_dto.validate().is_ok());
}

#[test]
fn test_worklog_validation() {
    use crate::entities::workforce::CreateWorkLogDto;
    use chrono::Utc;
    use uuid::Uuid;

    let dto = CreateWorkLogDto {
        worker_id: Uuid::new_v4(),
        date: Utc::now(),
        hours_worked: 25.0, // Invalid: max 24
        overtime_hours: -1.0, // Invalid: negative
        rest_period_hours: 8.0,
        task_description: "".to_string(), // Invalid: empty
        site_id: None,
        is_night_shift: false,
        breaks_taken: 1,
    };
    let res = dto.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.field_errors().contains_key("hours_worked"));
    assert!(errs.field_errors().contains_key("overtime_hours"));
    assert!(errs.field_errors().contains_key("task_description"));

    let mut valid_dto = dto.clone();
    valid_dto.hours_worked = 8.0;
    valid_dto.overtime_hours = 0.0;
    valid_dto.task_description = "Pruning vines".to_string();
    assert!(valid_dto.validate().is_ok());
}
