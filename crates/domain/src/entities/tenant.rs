use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Tenant {
    pub id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
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
    Vineyard,
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

pub type TenantId = uuid::Uuid;
