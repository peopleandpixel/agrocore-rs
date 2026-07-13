use agrocore_domain::entities::tenant::{CreateTenantDto, Module, Tenant, TenantConfig, TenantValidationRules};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[test]
fn tenant_and_create_tenant_validation_cover_required_fields() {
    let invalid = Tenant {
        id: Uuid::new_v4(),
        name: String::new(),
        slug: String::new(),
        config: TenantConfig::default(),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    assert!(invalid.validate().is_err());

    let dto = CreateTenantDto {
        name: String::from("AgroCore"),
        slug: String::from("agrocore"),
        config: Some(TenantConfig {
            default_language: String::from("de"),
            supported_languages: vec![String::from("de"), String::from("en")],
            timezone: String::from("Europe/Lisbon"),
            enabled_modules: vec![Module::FieldManagement],
            custom_field_schemas: None,
            logo_url: None,
            primary_color: None,
            validation_rules: Some(TenantValidationRules {
                require_bbch_on_protection: true,
                lock_completed_orders: false,
                allow_future_tasks: false,
            }),
        }),
    };
    assert!(dto.validate().is_ok());
}