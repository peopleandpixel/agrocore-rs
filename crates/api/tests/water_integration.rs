mod common;

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::configure;
use agrocore_domain::TenantId;
use agrocore_domain::entities::water::{
    IrrigationMethod, WaterQuota, WaterSource, WaterSourceType, WaterUsage,
};
use agrocore_domain::repositories::{
    MockWaterQuotaRepo, MockWaterSourceRepo, MockWaterUsageRepo, PaginatedResponse,
};
use chrono::Utc;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[actix_web::test]
async fn test_list_water_sources() {
    let mut source_repo = MockWaterSourceRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let source = WaterSource {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        source_type: WaterSourceType::Well,
        name: "Well 1".into(),
        capacity_m3: Some(1000.0),
        current_level_m3: Some(800.0),
        license_number: None,
        license_expiry: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    source_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![source.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mock_db = crate::common::db_with(|db| db.water_source_repo = Some(Arc::new(source_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Manager"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/water/sources")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_water_usage() {
    let mut usage_repo = MockWaterUsageRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let usage = WaterUsage {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        source_id: Uuid::new_v4(),
        usage_date: Utc::now(),
        volume_m3: 50.0,
        irrigation_method: IrrigationMethod::Drip,
        efficiency_pct: Some(90.0),
        created_at: Utc::now(),
    };

    usage_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![usage.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mock_db = crate::common::db_with(|db| db.water_usage_repo = Some(Arc::new(usage_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Manager"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/water/usage")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_water_quotas() {
    let mut quota_repo = MockWaterQuotaRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let quota = WaterQuota {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        source_id: Uuid::new_v4(),
        site_id: Uuid::new_v4(),
        year: 2026,
        allocated_m3: 5000.0,
        used_m3: 1200.0,
        remaining_m3: 3800.0,
        comunidad_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    quota_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![quota.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mock_db = crate::common::db_with(|db| db.water_quota_repo = Some(Arc::new(quota_repo)));

    let state = crate::common::state_with(mock_db);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token =
        crate::common::signed_token(&user_id.to_string(), &tenant_id.to_string(), &["Manager"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/water/quotas")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
