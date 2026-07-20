use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct Tenant {
    pub id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    #[sqlx(json)]
    pub config: TenantConfig,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct TenantConfig {
    pub default_language: String,
    pub supported_languages: Vec<String>,
    pub timezone: String,
    pub enabled_modules: Vec<Module>,
    pub custom_field_schemas: Option<serde_json::Value>,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub validation_rules: Option<TenantValidationRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct TenantValidationRules {
    pub require_bbch_on_protection: bool,
    pub lock_completed_orders: bool,
    pub allow_future_tasks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum Module {
    #[serde(rename = "field_management")]
    FieldManagement,
    PlantProtection,
    Fertilization,
    Harvest,
    WorkLog,
    CostTracking,
    Maps,
    Reports,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateTenantDto {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    pub config: Option<TenantConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateTenantDto {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub config: Option<TenantConfig>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct TenantId(pub Uuid);

impl From<Uuid> for TenantId {
    fn from(uuid: Uuid) -> Self {
        TenantId(uuid)
    }
}

impl From<TenantId> for Uuid {
    fn from(id: TenantId) -> Uuid {
        id.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
}
