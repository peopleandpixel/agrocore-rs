use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::reporting::worker::ReportingResponse;
use agrocore_api::{AppState, handlers::configure};
use agrocore_infrastructure::{Database, MockDatabase};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    exp: usize,
}

fn signed_token(sub: &str, tenant_id: &str, roles: Vec<&str>) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

#[actix_web::test]
async fn test_export_orders_excel() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mock_db = MockDatabase::default();
    let messaging = agrocore_messaging::MessagingClient::new_mock();

    // Set mock response for reporting.request
    let response = ReportingResponse::Excel(vec![0x50, 0x4B, 0x03, 0x04]); // ZIP header
    messaging.set_mock_response("reporting.request", serde_json::to_vec(&response).unwrap());

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(messaging),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Admin"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/reporting/export/orders/excel")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
}

#[actix_web::test]
async fn test_export_sites_geojson() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mock_db = MockDatabase::default();
    let messaging = agrocore_messaging::MessagingClient::new_mock();

    let feature_collection = geojson::FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None,
    };
    let response = ReportingResponse::GeoJson(feature_collection);
    messaging.set_mock_response("reporting.request", serde_json::to_vec(&response).unwrap());

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: Arc::new(messaging),
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = signed_token(&user_id.to_string(), &tenant_id.to_string(), vec!["Admin"]);
    let req = test::TestRequest::get()
        .uri("/api/v1/reporting/export/sites/geojson")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
