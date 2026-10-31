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
    /// Whether this tenant is a demo tenant (auto-seeded, resettable).
    pub is_demo: bool,
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
    // Company profile with address validation and geocoding
    pub company_profile: Option<CompanyProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CompanyProfile {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 300))]
    pub address: String,
    #[validate(length(min = 1, max = 100))]
    pub country: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 5, max = 30))]
    pub phone: String,
    /// Mobile phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<String>,
    /// Fax number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fax: Option<String>,
    /// Company website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Tax ID / NIF (Portugal), CIF (Spain), VAT ID, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    /// VAT identification number (EU format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_id: Option<String>,
    /// Commercial registry / registration number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<String>,
    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Postal code / ZIP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// State / Province
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    // Filled by reverse geocoding after validation or map picker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<GeoCoordinates>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoCoordinates {
    pub lat: f64,
    pub lng: f64,
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
    #[serde(rename = "inventory", alias = "Inventory")]
    Inventory,
    #[serde(rename = "livestock", alias = "Livestock")]
    Livestock,
    #[serde(rename = "equipment", alias = "Equipment")]
    Equipment,
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
            is_demo: false,
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
                company_profile: None,
            }),
        };
        assert!(dto.validate().is_ok());
    }
}
