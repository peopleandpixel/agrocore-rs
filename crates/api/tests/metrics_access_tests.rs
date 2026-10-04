//! The metrics endpoints must not be public (tasks.md C2).
//!
//! `/metrics/db` and `/metrics/business` were registered directly on the `App` in
//! `main`, outside `handlers::configure`, with no extractor in the signature. Any
//! caller who could reach the port got the whole registry: per-table query counts
//! and durations, pool saturation, and record counts.
//!
//! Two things about that are worth stating plainly, because the fix should not
//! oversell itself:
//!
//! - The registry carries no tenant label. The metric vectors are keyed by query
//!   type and table name only, so this was never a cross-tenant data leak. It was
//!   an unauthenticated operational map of the deployment.
//! - The endpoints were also invisible to the route inventory test, because that
//!   test builds an `App` from `handlers::configure` and the metrics routes were
//!   never part of it. An endpoint no test can reach is an endpoint no test can
//!   catch, so the guard below and the routes themselves are both asserted here
//!   against a scope the test mounts itself.

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, metrics_routes};
use agrocore_infrastructure::Database;
use jsonwebtoken::{EncodingKey, Header, encode};
use prometheus::Registry;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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

/// A stable peer address for every request in this file.
///
/// Each test builds a fresh `App`, so the governor's per-IP state is fresh with it
/// and one address per file is enough.
fn peer_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)), 40000)
}

fn state() -> AppState {
    let metrics_registry = Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::default())),
        // `None` rather than `new_mock()`: that helper panics by design — it is a
        // placeholder for a mock that was never written, and every caller that
        // constructs `AppState` has to work around it. The metrics handlers do not
        // touch messaging, and the field is `Option` precisely so a broker outage
        // does not stop the API from starting.
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

/// The two endpoints exist, and an anonymous caller gets nothing.
///
/// Note what this test does and does not prove. The 401 comes from
/// `AuthExtractor`, which the handler gets as an argument — it would reject an
/// anonymous caller whether or not `require_admin` is called. This test guards
/// against the routes being unregistered or the extractor being dropped from the
/// signature; `metrics_endpoints_reject_non_admin_roles` is the one that fails when
/// the admin check is removed. Both were verified by deleting the check.
#[actix_web::test]
async fn metrics_endpoints_reject_anonymous_callers() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state()))
            .configure(metrics_routes),
    )
    .await;

    for uri in ["/metrics/db", "/metrics/business"] {
        // `actix-governor` keys its state on the peer address. A bare `TestRequest`
        // has none, and the call fails with a 500 before the handler is reached, so
        // the request has to carry one.
        let req = test::TestRequest::get()
            .peer_addr(peer_addr())
            .uri(uri)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_ne!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "{uri} must still be routed; a 404 here means the guard test is vacuous"
        );
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "{uri} served an unauthenticated caller"
        );
    }
}

/// A signed-in user without the Admin role is refused too. The guard is not just
/// "is there a token" — a tenant's own Manager must not read another tenant's
/// deployment shape.
#[actix_web::test]
async fn metrics_endpoints_reject_non_admin_roles() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state()))
            .configure(metrics_routes),
    )
    .await;

    for role in ["Manager", "Viewer", "User"] {
        let token = signed_token(
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            vec![role],
        );
        for uri in ["/metrics/db", "/metrics/business"] {
            let req = test::TestRequest::get()
                .peer_addr(peer_addr())
                .uri(uri)
                .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
                .to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(
                resp.status(),
                StatusCode::FORBIDDEN,
                "{uri} served a {role} who is not an Admin"
            );
        }
    }
}

/// An Admin gets the registry, in Prometheus text format.
#[actix_web::test]
async fn metrics_endpoints_serve_an_admin() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state()))
            .configure(metrics_routes),
    )
    .await;

    let token = signed_token(
        &Uuid::new_v4().to_string(),
        &Uuid::new_v4().to_string(),
        vec!["Admin"],
    );

    for uri in ["/metrics/db", "/metrics/business"] {
        let req = test::TestRequest::get()
            .peer_addr(peer_addr())
            .uri(uri)
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK, "{uri} refused a valid Admin");

        let body = test::read_body(resp).await;
        let text = String::from_utf8_lossy(&body);
        assert!(
            text.contains("agrocore_db_"),
            "{uri} did not return the DB metrics it is supposed to serve; body was: {}",
            &text[..text.len().min(400)]
        );
    }
}
