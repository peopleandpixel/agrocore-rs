mod common;

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::configure;
use agrocore_domain::TenantId;
use agrocore_domain::entities::equipment::{Equipment, EquipmentType};
use agrocore_domain::repositories::{MockEquipmentRepository, PaginatedResponse};
use chrono::Utc;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[actix_web::test]
async fn test_list_equipment() {
    let mut equip_repo = MockEquipmentRepository::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let equipment = Equipment {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        label: "Tractor 1".into(),
        code: Some("TR-01".into()),
        equipment_type: EquipmentType::Tractor,
        in_usage: false,
        maintenance_intervals: None,
        next_maintenance_date: None,
        last_maintenance_hours: None,
        fuel_capacity_liters: None,
        fuel_type: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    equip_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![equipment.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mock_db = crate::common::db_with(|db| db.equipment_repo = Some(Arc::new(equip_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Viewer"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/equipments")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
