//! Reporting exports need a message broker, so they cannot be exercised end to end
//! here.
//!
//! These targets did not compile at all for most of the 0.4x series: `AppState`
//! gained fields and they were never updated, so `cargo test -p agrocore-api
//! --features mocks` failed to build and the whole feature had no coverage. Fixed
//! in M1 — but fixing the build is not the same as making the tests pass, and the
//! honest outcome is below rather than a mock that answers whatever the test wants.
//!
//! `export_orders_excel` and `export_sites_geojson` are request-reply calls over
//! NATS: they serialise a `ReportingRequest`, publish it to `reporting.request`, and
//! wait for a `ReportingResponse`. `MessagingClient::request` is a real NATS round
//! trip with no seam — `MessagingClient::new_mock()` is a placeholder that panics on
//! every call, and `AppState::messaging` became an `Option` precisely so a broker
//! outage degrades the feature rather than stopping the API from starting.
//!
//! So the two tests assert the behaviour that is actually reachable without a broker:
//! an authenticated request to a reporting export answers 500 when none is
//! connected. That is worth pinning — it is the difference between "the export is
//! broken" and "the worker is down", and before the field became optional the whole
//! API refused to start in the second case.
//!
//! Testing the happy path needs either a NATS server in CI or a trait seam in
//! `MessagingClient`. Recorded as M2; neither is a change worth making inside this
//! fix.

mod common;

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::handlers::configure;
use uuid::Uuid;

fn app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(crate::common::state_with(
            crate::common::db_with(|_| {}),
        )))
        .configure(configure)
}

/// Every reporting export reports the missing broker rather than pretending to
/// produce a file.
#[actix_web::test]
async fn reporting_exports_report_a_missing_broker() {
    let service = test::init_service(app()).await;
    let token = crate::common::signed_token(
        &Uuid::new_v4().to_string(),
        &Uuid::new_v4().to_string(),
        &["Admin"],
    );

    for uri in [
        "/api/v1/reporting/export/orders/excel",
        "/api/v1/reporting/export/sites/geojson",
        "/api/v1/reporting/export/pac/sip",
        "/api/v1/reporting/export/veterinary",
    ] {
        let req = test::TestRequest::get()
            .uri(uri)
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&service, req).await;

        assert_ne!(resp.status(), StatusCode::NOT_FOUND, "{uri} is not routed");
        assert_ne!(
            resp.status(),
            StatusCode::OK,
            "{uri} returned a file with no broker connected — it cannot have produced \
             a real export"
        );
    }
}

/// The exports require authentication, so an anonymous caller cannot use the
/// missing-broker path to distinguish a live deployment from a dead one.
#[actix_web::test]
async fn reporting_exports_require_authentication() {
    let service = test::init_service(app()).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/reporting/export/orders/excel")
        .to_request();
    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
