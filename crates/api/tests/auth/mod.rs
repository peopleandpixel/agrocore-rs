// Auth Handler Tests - Login Validation
use actix_web::{http::StatusCode, test, App, HttpResponse};
use agrocore_api::handlers::auth::LoginRequest;
use serde_json::json;

#[actix_web::test]
async fn test_login_validation_rejects_short_password() {
    let req = LoginRequest {
        email: "test@example.com".into(),
        password: "short".into(), // < 8 chars
    };
    
    let result = validator::Validate::validate(&req);
    assert!(result.is_err(), "Short password should fail validation");
}

#[actix_web::test]
async fn test_login_validation_rejects_invalid_email() {
    let req = LoginRequest {
        email: "not-an-email".into(),
        password: "validpassword123".into(),
    };
    
    let result = validator::Validate::validate(&req);
    assert!(result.is_err(), "Invalid email should fail validation");
}

#[actix_web::test]
async fn test_login_validation_accepts_valid_credentials() {
    let req = LoginRequest {
        email: "valid@example.com".into(),
        password: "validpassword123".into(),
    };
    
    let result = validator::Validate::validate(&req);
    assert!(result.is_ok(), "Valid credentials should pass validation");
}