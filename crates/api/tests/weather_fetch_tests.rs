//! The weather endpoint must fetch from a provider, not return constants
//! (tasks.md J9).
//!
//! `GET /api/v1/calculate/weather/fetch` parsed the requested provider into
//! `_service_type` and discarded it, then answered with a fixed 20.5 C, 65 %
//! humidity, 0.0 mm precipitation, 12 km/h wind, 800 W/m², 1013.25 hPa, 18 °C soil
//! and 45 % soil moisture. The three providers in `crates/weather-service` had no
//! caller anywhere in the workspace.
//!
//! These tests assert what can be asserted without a network: that the constants
//! are gone, that an unknown provider is rejected rather than silently defaulted,
//! that the provider that was asked for is the one named in the response, and that
//! the validation on the coordinates still runs. The network call itself is not
//! exercised — a test that depended on api.open-meteo.com would be a test that
//! fails when the internet does.

use actix_web::{App, http::StatusCode, http::header, test as awtest, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_infrastructure::Database;
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
    let metrics_registry = prometheus::Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::default())),
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

async fn post_fetch(
    body: serde_json::Value,
) -> actix_web::dev::ServiceResponse<actix_web::body::BoxBody> {
    let app = awtest::init_service(
        App::new()
            .app_data(web::Data::new(state()))
            .configure(configure),
    )
    .await;
    let token = signed_token(vec!["Manager"]);
    let req = awtest::TestRequest::post()
        .uri("/api/v1/calculate/weather/fetch")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .set_payload(body.to_string())
        .to_request();
    awtest::call_service(&app, req).await
}

/// An unknown provider is a 400, not a silent fallback to the default one.
///
/// The old handler matched `provider.to_lowercase()` with a `_ =>` arm that fell
/// through to Open-Meteo, so `{"provider":"weatherbug"}` was accepted and the
/// caller had no way to know they were not talking to the provider they asked for.
#[actix_web::test]
async fn an_unknown_provider_is_rejected() {
    let resp = post_fetch(serde_json::json!({
        "latitude": 38.7,
        "longitude": -9.1,
        "provider": "weatherbug",
    }))
    .await;

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "an unknown provider must be refused, not silently replaced"
    );

    let body: serde_json::Value = awtest::read_body_json(resp).await;
    let message = body["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("weatherbug"),
        "the error should name the provider that was not recognised: {message}"
    );
}

/// The three real providers are accepted. The request still reaches the network, so
/// this asserts the *rejection* path is not what answers — a 400 here would mean
/// the name was rejected; anything else means it was accepted and dispatched.
#[actix_web::test]
async fn the_three_real_providers_are_accepted() {
    for provider in ["openmeteo", "openweather", "wunderground"] {
        let resp = post_fetch(serde_json::json!({
            "latitude": 38.7,
            "longitude": -9.1,
            "provider": provider,
        }))
        .await;

        let status = resp.status();
        let body: serde_json::Value = awtest::read_body_json(resp).await;
        let message = body["message"].as_str().unwrap_or_default();

        assert!(
            !(status == StatusCode::BAD_REQUEST && message.contains("Unknown weather provider")),
            "`{provider}` must be a known provider; it was rejected as unknown"
        );
    }
}

/// Coordinates outside the valid range are refused by validation, not sent to a
/// provider. The DTO carries `range(min = -90, max = 90)` on latitude and
/// `range(min = -180, max = 180)` on longitude.
#[actix_web::test]
async fn impossible_coordinates_are_rejected_before_any_fetch() {
    for (lat, lon) in [(91.0, 0.0), (-91.0, 0.0), (0.0, 181.0), (0.0, -181.0)] {
        let resp = post_fetch(serde_json::json!({ "latitude": lat, "longitude": lon })).await;
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "latitude {lat}, longitude {lon} must not reach a provider"
        );
    }
}

/// The endpoint still requires authentication.
#[actix_web::test]
async fn the_weather_endpoint_requires_authentication() {
    let app = awtest::init_service(
        App::new()
            .app_data(web::Data::new(state()))
            .configure(configure),
    )
    .await;
    let req = awtest::TestRequest::post()
        .uri("/api/v1/calculate/weather/fetch")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload(r#"{"latitude":38.7,"longitude":-9.1}"#)
        .to_request();
    let resp = awtest::call_service(&app, req).await;

    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// The hard-coded response is gone from the source.
///
/// The previous implementation could not be caught by a request test without a
/// network stub, because the constants only appear on the success path. This
/// asserts their absence in the handler itself, which is where the defect was.
#[test]
fn the_hard_coded_reading_is_gone_from_the_handler() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/handlers/calculation.rs"),
    )
    .expect("read calculation.rs");

    // Doc comments are stripped first: the values appear in this test's own prose
    // about the defect, and the handler's doc comment quotes them to say what was
    // replaced. Only code counts here.
    let code: String = src
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    let body = code
        .split("pub async fn fetch_weather")
        .nth(1)
        .expect("fetch_weather must exist")
        // up to the next top-level `pub async fn`
        .split("\npub async fn ")
        .next()
        .unwrap_or_default();

    for constant in ["20.5", "65.0", "1013.25", "800.0", "12.0", "45.0"] {
        assert!(
            !body.contains(constant),
            "fetch_weather still contains the hard-coded value {constant}"
        );
    }
    assert!(
        body.contains("fetch_current"),
        "fetch_weather must call a provider"
    );
}
