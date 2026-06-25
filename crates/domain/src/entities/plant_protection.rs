use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PlantProtectionRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    pub product_name: String,
    pub active_substance: String,
    pub dosage_per_ha: f64,
    pub total_quantity: f64,
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub pre_harvest_days: u32,
    pub re_entry_days: u32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePlantProtectionDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub product_name: String,
    #[validate(length(min = 1))]
    pub active_substance: String,
    #[validate(range(min = 0.0))]
    pub dosage_per_ha: f64,
    #[validate(range(min = 0.0))]
    pub total_quantity: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub pre_harvest_days: u32,
    pub re_entry_days: u32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplicatorLicense {
    pub id: Uuid,
    pub user_id: Uuid,
    pub license_type: LicenseType,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicenseType {
    Basic,
    Advanced,
    Professional,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateApplicatorLicenseDto {
    pub user_id: Uuid,
    pub license_type: LicenseType,
    #[validate(length(min = 1))]
    pub license_number: String,
    #[validate(length(min = 1))]
    pub issued_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}
