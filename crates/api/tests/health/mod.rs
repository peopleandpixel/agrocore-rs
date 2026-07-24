use actix_web::{App, http::StatusCode, test, test::TestRequest};
use agrocore_api::handlers::configure;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().configure(configure)).await;
    let req = TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
