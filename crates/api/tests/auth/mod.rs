// Auth Handler Tests - Login Validation
use agrocore_api::handlers::auth::LoginRequest;
use validator::Validate;

#[test]
fn test_login_validation_rejects_short_password() {
    let req = LoginRequest {
        email: "test@example.com".into(),
        password: "short".into(), // < 8 chars
    };

    let result = Validate::validate(&req);
    assert!(result.is_err(), "Short password should fail validation");
}

#[test]
fn test_login_validation_rejects_invalid_email() {
    let req = LoginRequest {
        email: "not-an-email".into(),
        password: "validpassword123".into(),
    };

    let result = Validate::validate(&req);
    assert!(result.is_err(), "Invalid email should fail validation");
}

#[test]
fn test_login_validation_accepts_valid_credentials() {
    let req = LoginRequest {
        email: "valid@example.com".into(),
        password: "validpassword123".into(),
    };

    let result = Validate::validate(&req);
    assert!(result.is_ok(), "Valid credentials should pass validation");
}
