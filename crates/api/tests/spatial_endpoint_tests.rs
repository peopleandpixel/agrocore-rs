//! The map's spatial read endpoint: reachable, and it validates what it is given.
//!
//! The point of these tests is reachability and argument handling, not the response body.
//! Four times in this project a handler was written and either never registered, or
//! registered behind a `{id}` pattern that swallowed it. A test that builds the real
//! router and calls the real path is the only kind that catches that, so the last test
//! here mounts `handlers::configure` rather than a hand-picked scope.
//
//! Note: the `mocks` feature is required for the mock Database implementation.
#![cfg(feature = "mocks")]

use actix_web::{App, http::StatusCode, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_infrastructure::{Database, MockDatabase};
use agrocore_domain::repositories::MockSpatialObjectRepository;
use futures::future::ready;
use jsonwebtoken::{EncodingKey, Header, encode};
use prometheus::Registry;
use serde::Serialize;
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

fn signed_token(roles: Vec<&str>) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
            jti: Uuid::new_v4().to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

fn state() -> AppState {
    let metrics_registry = Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::default())),
        // `None` rather than `new_mock()`: that helper panics by design. The field is
        // `Option` so a broker outage does not stop the API from starting.
        messaging: None,
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
        db_metrics: agrocore_api::metrics::DbMetrics::new(&metrics_registry),
        business_metrics: agrocore_api::metrics::BusinessMetrics::new(&metrics_registry),
        metrics_registry: Arc::new(metrics_registry),
        backup_service: None,
        demo_endpoints_enabled: false,
    }
}

fn state_with_spatial_mock() -> AppState {
    let mut mock_db = MockDatabase::default();
    let mut spatial_repo = MockSpatialObjectRepository::new();
    spatial_repo
        .expect_find_by_filter()
        .returning(|_, _| Box::pin(ready(Ok(vec![]))));
    mock_db.spatial_object_repo = Some(Arc::new(spatial_repo));
    
    let metrics_registry = Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: None,
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
        db_metrics: agrocore_api::metrics::DbMetrics::new(&metrics_registry),
        business_metrics: agrocore_api::metrics::BusinessMetrics::new(&metrics_registry),
        metrics_registry: Arc::new(metrics_registry),
        backup_service: None,
        demo_endpoints_enabled: false,
    }
}

fn auth() -> String {
    format!("Bearer {}", signed_token(vec!["admin"]))
}

/// An authenticated GET reaches the handler and comes back with a FeatureCollection.
#[actix_web::test]
async fn spatial_objects_endpoint_answers() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/spatial/objects")
        .insert_header(("Authorization", auth()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "got {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "FeatureCollection");
    assert!(body["features"].is_array());
}

/// An anonymous caller is rejected — the map is not public.
#[actix_web::test]
async fn spatial_objects_requires_authentication() {
    let app = test::init_service(App::new().app_data(web::Data::new(state())).configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spatial/objects")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// A malformed bbox is a 400, not an empty collection.
///
/// This is the failure mode the validation exists to prevent: a viewport with latitude 140
/// matches nothing, so the map would draw an empty farm and report no error at all.
#[actix_web::test]
async fn bbox_is_rejected_when_malformed() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;
    for bad in [
        "1,2,3",               // three components
        "1,2,3,4,5",           // five
        "a,b,c,d",             // not numbers
        "-7.9,140,-7.8,37.0",  // latitude out of range
        "-200,37,-7.8,37.0",   // longitude out of range
        "-7.8,37.0,-7.9,37.1", // minimum above maximum
    ] {
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/spatial/objects?bbox={bad}"))
            .insert_header(("Authorization", auth()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "bbox '{bad}' should be rejected, got {}",
            resp.status()
        );
    }
}

/// A well-formed bbox in the Alentejo is accepted, as is every documented filter value.
#[actix_web::test]
async fn valid_filters_are_accepted() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;
    for uri in [
        "/api/v1/spatial/objects?bbox=-7.95,37.0,-7.85,37.05",
        "/api/v1/spatial/objects?object_type=olive_tree",
        "/api/v1/spatial/objects?planted_at=before:2015-01-01",
        "/api/v1/spatial/objects?planted_at=after:2015-01-01",
        "/api/v1/spatial/objects?planted_at=set",
        "/api/v1/spatial/objects?planted_at=null",
        "/api/v1/spatial/objects?include_inactive=true",
        "/api/v1/spatial/objects?limit=100",
    ] {
        let req = test::TestRequest::get()
            .uri(uri)
            .insert_header(("Authorization", auth()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "'{uri}' should be accepted, got {}", resp.status());
    }
}

/// An unknown `planted_at` prefix is rejected rather than ignored.
///
/// A silently ignored filter is worse than a missing one: the client would believe it
/// had filtered by planting date and would draw every object it received.
#[actix_web::test]
async fn unknown_planted_at_prefix_is_rejected() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;
    for bad in ["during:2015-01-01", "before:not-a-date", "2015-01-01"] {
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/spatial/objects?planted_at={bad}"))
            .insert_header(("Authorization", auth()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "'{bad}' should be rejected, got {}",
            resp.status()
        );
    }
}

/// `limit=0` returns data rather than an empty collection.
///
/// Zero would otherwise read as "no objects on this plot", which is a different statement
/// from "you asked for none".
#[actix_web::test]
async fn zero_limit_is_clamped_not_honoured() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spatial/objects?limit=0")
        .insert_header(("Authorization", auth()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "got {}", resp.status());
}

/// The route exists on the real router.
///
/// Every other test in this file mounts the same `configure`, so this one is the explicit
/// statement of intent: the spatial handler is wired into the application, not only into a
/// test-local scope.
#[actix_web::test]
async fn spatial_route_is_registered_on_the_real_router() {
    let app = test::init_service(App::new().app_data(web::Data::new(state_with_spatial_mock())).configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/spatial/objects")
        .insert_header(("Authorization", auth()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "the real router did not reach the spatial handler: {}",
        resp.status()
    );
}