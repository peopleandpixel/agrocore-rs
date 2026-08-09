use actix_web::{App, http::StatusCode, test, test::TestRequest};
use agrocore_api::handlers::configure;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().configure(configure)).await;
    let req = TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let prometheus = actix_web_prometheus::PrometheusMetricsBuilder::new("agrocore")
        .endpoint("/metrics")
        .build()
        .unwrap();
    let app = test::init_service(App::new().wrap(prometheus).configure(configure)).await;
    let req = TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_swagger_ui() {
    let app = test::init_service(
        App::new()
            .service(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").url(
                "/api-docs/openapi.json",
                utoipa::openapi::OpenApi::default(),
            ))
            .configure(configure),
    )
    .await;
    let req = TestRequest::get().uri("/swagger-ui/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_iot_routes_require_authentication() {
    let app = test::init_service(App::new().configure(configure)).await;
    for uri in [
        "/api/v1/iot/devices",
        "/api/v1/iot/devices/demo",
        "/api/v1/iot/devices/demo/telemetry",
    ] {
        let req = TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[actix_web::test]
async fn test_task_routes_are_registered() {
    let app = test::init_service(App::new().configure(configure)).await;
    for (method, uri) in [
        ("GET", "/api/v1/tasks"),
        ("GET", "/api/v1/tasks/00000000-0000-0000-0000-000000000001"),
        (
            "DELETE",
            "/api/v1/tasks/00000000-0000-0000-0000-000000000001",
        ),
    ] {
        let req = match method {
            "DELETE" => TestRequest::delete().uri(uri).to_request(),
            _ => TestRequest::get().uri(uri).to_request(),
        };
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[actix_web::test]
async fn test_vineyard_routes_are_registered() {
    let app = test::init_service(App::new().configure(configure)).await;
    for uri in [
        "/api/v1/specialized/vineyards",
        "/api/v1/specialized/vineyards/site/00000000-0000-0000-0000-000000000001",
        "/api/v1/specialized/vineyards/00000000-0000-0000-0000-000000000001",
    ] {
        let req = TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[actix_web::test]
async fn test_remaining_module_routes_are_registered() {
    let app = test::init_service(App::new().configure(configure)).await;
    for uri in [
        "/api/v1/workforce/workers",
        "/api/v1/weather/data",
        "/api/v1/compliance/checklists",
        "/api/v1/finance/cost-centers",
        "/api/v1/finance/financial-records",
        "/api/v1/finance/pac-applications",
        "/api/v1/harvest/seasons",
        "/api/v1/animals",
        "/api/v1/specialized/olive-groves",
    ] {
        let req = TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_ne!(resp.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}
