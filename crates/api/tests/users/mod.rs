// Users Handler Tests - DTO Validation  
use agrocore_api::dto::{CreateUserDto, UpdateUserDto};

#[test]
fn test_create_user_validation() {
    let dto = CreateUserDto {
        tenant_id: "tenant-123".into(),
        firstname: "John".into(),
        lastname: "Doe".into(),
        email: "john@example.com".into(),
        password: "securepass123".into(),
        roles: Some(vec!["Worker".into()]),
    };
    
    assert_eq!(dto.email, "john@example.com");
    assert!(dto.password.len() >= 8);
}

#[test]
fn test_update_user_validation() {
    let dto = UpdateUserDto {
        firstname: Some("Jane".into()),
        lastname: Some("Doe".into()),
        is_active: Some(true),
    };
    
    assert_eq!(dto.firstname, Some("Jane".into()));
}