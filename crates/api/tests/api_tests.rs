use actix_web::{test, App, http::StatusCode};
use agrocore_api::handlers::configure;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_404_not_found() {
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/does-not-exist")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_cors_preflight() {
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::options()
        .uri("/api/v1/health")
        .insert_header(("Origin", "http://example.com"))
        .insert_header(("Access-Control-Request-Method", "GET"))
        .to_request();
    let resp = test::call_service(&app, req).await);
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_unauthorized_no_token() {
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/orders/my-tasks")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_unauthorized_bad_token() {
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::get()
        .uri("/api/v1/orders/my-tasks")
        .insert_header(("Authorization", "Bearer bad-token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_login_validation_error() {
    use serde_json::json;
    let app = test::init_service(
        App::new().configure(configure)
    ).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(json!({"email": "invalid", "password": "x"}))
        .to_request();
    let resp = test::call_service(&app, req).await);
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}
