use agrocore_domain::entities::user::{Action, Resource, User, UserRole};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[test]
fn user_role_permissions_cover_core_branches() {
    assert!(UserRole::Admin.has_permission(Resource::Site, Action::Delete, None));
    assert!(UserRole::Manager.has_permission(Resource::Order, Action::Manage, None));
    assert!(UserRole::Worker.has_permission(Resource::Order, Action::Read, None));
    assert!(UserRole::Worker.has_permission(Resource::Order, Action::Update, None));
    assert!(!UserRole::Worker.has_permission(Resource::Order, Action::Delete, None));
    assert!(UserRole::Viewer.has_permission(Resource::User, Action::Read, None));
    assert!(!UserRole::Viewer.has_permission(Resource::User, Action::Update, None));
    assert!(!UserRole::Custom(Uuid::new_v4()).has_permission(
        Resource::Tenant,
        Action::Read,
        None
    ));
}

#[test]
fn user_validation_rejects_invalid_names_and_email() {
    let user = User {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        firstname: String::new(),
        lastname: String::new(),
        email: String::from("invalid"),
        password_hash: String::from("hash"),
        roles: vec![UserRole::Viewer],
        is_active: true,
        internal_cost_per_hour: None,
        external_cost_per_hour: None,
        color: None,
        language: None,
        assigned_site_ids: None,
        last_login: None,
        refresh_token: None,
        refresh_token_expires_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert!(user.validate().is_err());
}