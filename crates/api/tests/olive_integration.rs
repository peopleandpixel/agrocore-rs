use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::olive::{OilGrade, OliveGrove, OliveOilRecord};
use agrocore_domain::repositories::{
    MockOliveGroveRepo, MockOliveOilRecordRepo, PaginatedResponse,
};
use agrocore_infrastructure::{Database, MockDatabase};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    exp: usize,
    jti: String,
}

fn signed_token(sub: &str, tenant_id: &str, roles: Vec<&str>) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
            jti: uuid::Uuid::new_v4().to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

#[actix_web::test]
async fn test_list_olive_groves() {
    let mut grove_repo = MockOliveGroveRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let grove = OliveGrove {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        site_id: Uuid::new_v4(),
        label: "Grove 1".into(),
        variety: "Picual".into(),
        tree_count: Some(100),
        planting_year: Some(2010),
        area_ha: 2.5,
        spacing_m: Some(7.0),
        irrigation_type: None,
        is_organic: true,
        certification_body: None,
        certification_number: None,
        organic_certified: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    grove_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![grove.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.olive_grove_repo = Some(Arc::new(grove_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
    );
    let req = test::TestRequest::get()
        .uri("/api/v1/specialized/olive-groves")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_list_olive_oil_records() {
    let mut record_repo = MockOliveOilRecordRepo::new();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let record = OliveOilRecord {
        id: Uuid::new_v4(),
        tenant_id: TenantId(tenant_id),
        grove_id: Uuid::new_v4(),
        harvest_year: 2023,
        oil_grade: OilGrade::ExtraVirgin,
        acidity_pct: Some(0.2),
        peroxide_value: Some(5.0),
        sensory_score: Some(8.5),
        liters_produced: Some(500.0),
        mill_name: Some("Test Mill".into()),
        lot_number: Some("LOT-001".into()),
        harvest_date: Utc::now(),
        quantity_kg: Some(3000.0),
        oil_yield_kg: Some(450.0),
        oil_yield_percent: Some(15.0),
        k232: None,
        k270: None,
        quality_grade: None,
        notes: None,
        created_at: Utc::now(),
    };

    record_repo.expect_find_all().returning(move |_, p| {
        Box::pin(ready(Ok(PaginatedResponse {
            data: vec![record.clone()],
            total: 1,
            page: p.page.unwrap_or(0),
            per_page: p.per_page.unwrap_or(20),
            total_pages: 1,
        })))
    });

    let mut mock_db = MockDatabase::default();
    mock_db.olive_oil_record_repo = Some(Arc::new(record_repo));

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(agrocore_messaging::MessagingClient::new_mock()),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
    );
    let req = test::TestRequest::get()
        .uri("/api/v1/specialized/olive-oil-records")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
