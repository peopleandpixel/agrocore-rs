//! Request bodies must have a size limit, and the import endpoints must have a
//! larger one (tasks.md C3).
//!
//! No `JsonConfig`, `PayloadConfig` or payload limit existed anywhere in the
//! workspace. `web::Json` reads the whole body into memory and then deserialises it,
//! so a single request allocated twice its own size, unbounded, before any handler
//! code ran. All three import endpoints take a JSON body, and
//! `/sites/import/shapefile` carries the file as Base64 — a third more on top.
//!
//! Two limits, because one value cannot be right for both:
//!
//! - `MAX_JSON_PAYLOAD` (2 MiB) applies to everything.
//! - `MAX_IMPORT_PAYLOAD` (64 MiB) applies to the three import handlers, via the
//!   `LargeJson` extractor. `web::JsonConfig` is application data and therefore
//!   global, and re-registering the import routes in a scope with a second config
//!   does not work — the routes are already mounted by `handlers::configure` and a
//!   second registration is shadowed by the first.
//!
//! The bodies here are generated at the boundary rather than described, so a change
//! to either constant fails these tests instead of quietly invalidating them.

use actix_web::{App, http::StatusCode, http::header, test as awtest, web};
use agrocore_api::{AppState, MAX_IMPORT_PAYLOAD, MAX_JSON_PAYLOAD, handlers::configure};
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
    jti: String,
}

fn signed_token() -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            roles: vec!["Admin".to_string()],
            exp: usize::MAX / 2,
            jti: Uuid::new_v4().to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

fn state() -> AppState {
    let metrics_registry = prometheus::Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::new(MockDatabase::default()))),
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

/// An app with the same payload configuration the server builds. A test app
/// without it would not exercise the limit at all.
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
        .app_data(web::Data::new(state()))
        .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
        .app_data(web::PayloadConfig::default().limit(MAX_JSON_PAYLOAD))
        .configure(configure)
}

/// A JSON object of exactly `len` bytes.
fn body_of_len(len: usize) -> String {
    let prefix = r#"{"name":"#;
    let suffix = r#""}"#;
    let fill = len - prefix.len() - suffix.len();
    let mut s = String::with_capacity(len);
    s.push_str(prefix);
    s.extend(std::iter::repeat_n('a', fill));
    s.push_str(suffix);
    assert_eq!(s.len(), len);
    s
}

/// A shapefile import body of roughly `len` bytes: valid JSON, with a
/// `file_base64` string of that size.
fn shapefile_body_of_len(len: usize) -> String {
    let prefix = r#"{"file_base64":"#;
    let suffix = r#""}"#;
    let fill = len - prefix.len() - suffix.len();
    let mut s = String::with_capacity(len);
    s.push_str(prefix);
    s.extend(std::iter::repeat_n('A', fill));
    s.push_str(suffix);
    assert_eq!(s.len(), len);
    s
}

const IMPORT_URIS: [&str; 3] = [
    "/api/v1/sites/import",
    "/api/v1/sites/import/geojson",
    "/api/v1/sites/import/shapefile",
];

/// A body past the import ceiling is refused, even on the import endpoints.
///
/// This is the case that was open in both directions: an authenticated caller
/// could send gigabytes to `/sites/import/shapefile`, and the deserialiser would
/// hold all of it plus the parsed structure.
#[actix_web::test]
async fn a_body_past_the_import_limit_is_refused() {
    let service = awtest::init_service(app()).await;
    let token = signed_token();
    let oversized = shapefile_body_of_len(MAX_IMPORT_PAYLOAD + 4096);

    for uri in IMPORT_URIS {
        let req = awtest::TestRequest::post()
            .uri(uri)
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .set_payload(oversized.clone())
            .to_request();
        let resp = awtest::call_service(&service, req).await;

        assert_eq!(
            resp.status(),
            StatusCode::PAYLOAD_TOO_LARGE,
            "{uri} accepted {} bytes, past the {} byte import limit",
            oversized.len(),
            MAX_IMPORT_PAYLOAD
        );
    }
}

/// A body between the global and the import ceiling gets past the size check.
///
/// It still fails later — the payload is not a valid shapefile and the repository
/// is a mock — but it must not fail with 413. That distinction is the whole point:
/// it shows `LargeJson` raised the ceiling for these three endpoints rather than
/// leaving them on the global limit.
#[actix_web::test]
async fn the_import_endpoints_accept_more_than_the_global_limit() {
    let service = awtest::init_service(app()).await;
    let token = signed_token();

    // Comfortably above MAX_JSON_PAYLOAD and below MAX_IMPORT_PAYLOAD.
    let large = shapefile_body_of_len(MAX_JSON_PAYLOAD + (1024 * 1024));

    let req = awtest::TestRequest::post()
        .uri("/api/v1/sites/import/shapefile")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_payload(large)
        .to_request();
    let resp = awtest::call_service(&service, req).await;

    assert_ne!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "a {} byte body is over the {} byte global limit but under the import \\
         limit; the import endpoint must accept it for size purposes",
        MAX_JSON_PAYLOAD + (1024 * 1024),
        MAX_IMPORT_PAYLOAD
    );
}

/// The global limit applies to the rest of the API.
///
/// This is asserted on a normal endpoint, not on the imports, because that is where
/// the tight limit belongs: a settings update has no business being megabytes.
#[actix_web::test]
async fn the_global_limit_applies_to_ordinary_endpoints() {
    let service = awtest::init_service(app()).await;
    let token = signed_token();
    let oversized = body_of_len(MAX_JSON_PAYLOAD + 4096);

    // PUT, not POST: the settings group resource only registers GET and PUT, so a
    // POST would answer 405 before the payload was ever read, and the test would
    // pass for the wrong reason.
    let req = awtest::TestRequest::put()
        .uri("/api/v1/settings/backup")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_payload(oversized)
        .to_request();
    let resp = awtest::call_service(&service, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "an ordinary endpoint accepted a body past the {} byte limit",
        MAX_JSON_PAYLOAD
    );
}

/// A body under the limit is not rejected for size.
///
/// The request is unauthenticated on purpose. The extractor runs before the
/// handler's own guards, so this reaches the size check without needing a database
/// or a repository mock, and a 401 rather than a 413 is the correct answer: the body
/// was small enough to parse. Without this case, the tests above would also pass if
/// the limit rejected every body including tiny ones.
#[actix_web::test]
async fn a_small_body_is_not_rejected_for_size() {
    let service = awtest::init_service(app()).await;

    let req = awtest::TestRequest::post()
        .uri("/api/v1/sites/import/shapefile")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(r#"{"file_base64":"UEsDBBQ=","skip_duplicates":true}"#)
        .to_request();
    let resp = awtest::call_service(&service, req).await;

    assert_ne!(
        resp.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "a small body was rejected for size"
    );
}

/// The two constants are ordered. If `MAX_IMPORT_PAYLOAD` were ever set below the
/// global limit, the imports would be *more* restricted than the rest of the API,
/// which is the opposite of the intent.
#[test]
fn the_import_limit_is_larger_than_the_global_one() {
    assert!(
        MAX_IMPORT_PAYLOAD > MAX_JSON_PAYLOAD,
        "the import ceiling ({MAX_IMPORT_PAYLOAD}) must be above the global one \\
         ({MAX_JSON_PAYLOAD})"
    );
    assert_eq!(MAX_JSON_PAYLOAD, 2 * 1024 * 1024);
    assert_eq!(MAX_IMPORT_PAYLOAD, 64 * 1024 * 1024);
}
