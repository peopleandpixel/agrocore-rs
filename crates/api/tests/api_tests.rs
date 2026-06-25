use actix_web::{test, test::TestRequest, http::StatusCode, App};
use agrocore_api::handlers::configure;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().configure(configure)).await;
    let req = TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let prometheus = actix_web_prometheus::PrometheusMetricsBuilder::new("agrocore")
        .endpoint("/metrics")
        .build()
        .unwrap();
    let app = test::init_service(
        App::new()
            .wrap(prometheus)
            .configure(configure)
    ).await;
    let req = TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_swagger_ui() {
    let app = test::init_service(
        App::new()
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", utoipa::openapi::OpenApi::default()),
            )
            .configure(configure)
    ).await;
    let req = TestRequest::get().uri("/swagger-ui/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK); // UI itself returns 200
}