//! Plant Protection DTOs

use agrocore_domain::entities::plant_protection::{
    ApplicatorLicense, CreateApplicatorLicenseDto as DomainCreateApplicatorLicenseDto,
    CreatePlantProtectionDto as DomainCreatePlantProtectionDto, PlantProtectionRecord,
    UpdateApplicatorLicenseDto as DomainUpdateApplicatorLicenseDto,
    UpdatePlantProtectionDto as DomainUpdatePlantProtectionDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPlantProtectionResponse {
    pub data: Vec<PlantProtectionRecordDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlantProtectionRecordDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    pub product_name: String,
    pub active_substance: String,
    pub dosage_per_ha: f64,
    pub total_quantity: f64,
    pub area_ha: f64,
    pub application_date: String,
    pub pre_harvest_days: i32,
    pub re_entry_days: i32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PlantProtectionRecord> for PlantProtectionRecordDto {
    fn from(p: PlantProtectionRecord) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id.into(),
            site_id: p.site_id,
            order_id: p.order_id,
            product_name: p.product_name,
            active_substance: p.active_substance,
            dosage_per_ha: p.dosage_per_ha,
            total_quantity: p.total_quantity,
            area_ha: p.area_ha,
            application_date: p.application_date.to_rfc3339(),
            pre_harvest_days: p.pre_harvest_days,
            re_entry_days: p.re_entry_days,
            weather_conditions: p.weather_conditions,
            applicator_license: p.applicator_license,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
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
    pub application_date: String,
    pub pre_harvest_days: i32,
    pub re_entry_days: i32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
}

impl From<CreatePlantProtectionDto> for DomainCreatePlantProtectionDto {
    fn from(dto: CreatePlantProtectionDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            active_substance: dto.active_substance,
            dosage_per_ha: dto.dosage_per_ha,
            total_quantity: dto.total_quantity,
            area_ha: dto.area_ha,
            application_date: chrono::DateTime::parse_from_rfc3339(&dto.application_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            pre_harvest_days: dto.pre_harvest_days,
            re_entry_days: dto.re_entry_days,
            weather_conditions: dto.weather_conditions,
            applicator_license: dto.applicator_license,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdatePlantProtectionDto {
    pub site_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub active_substance: Option<String>,
    pub dosage_per_ha: Option<f64>,
    pub total_quantity: Option<f64>,
    pub area_ha: Option<f64>,
    pub application_date: Option<String>,
    pub pre_harvest_days: Option<u32>,
    pub re_entry_days: Option<u32>,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
}

impl From<UpdatePlantProtectionDto> for DomainUpdatePlantProtectionDto {
    fn from(dto: UpdatePlantProtectionDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            active_substance: dto.active_substance,
            dosage_per_ha: dto.dosage_per_ha,
            total_quantity: dto.total_quantity,
            area_ha: dto.area_ha,
            application_date: dto.application_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            pre_harvest_days: dto.pre_harvest_days.map(|v| v as u32),
            re_entry_days: dto.re_entry_days.map(|v| v as u32),
            weather_conditions: dto.weather_conditions,
            applicator_license: dto.applicator_license,
        }
    }
}

// Applicator License DTOs
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedApplicatorLicenseResponse {
    pub data: Vec<ApplicatorLicenseDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicatorLicenseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub license_type: agrocore_domain::entities::plant_protection::LicenseType,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
    pub is_active: bool,
    pub created_at: String,
}

impl From<ApplicatorLicense> for ApplicatorLicenseDto {
    fn from(a: ApplicatorLicense) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            license_type: a.license_type,
            license_number: a.license_number,
            issued_by: a.issued_by,
            valid_from: a.valid_from.to_rfc3339(),
            valid_until: a.valid_until.to_rfc3339(),
            is_active: a.is_active,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateApplicatorLicenseDto {
    pub user_id: Uuid,
    pub license_type: agrocore_domain::entities::plant_protection::LicenseType,
    #[validate(length(min = 1))]
    pub license_number: String,
    #[validate(length(min = 1))]
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
}

impl From<CreateApplicatorLicenseDto> for DomainCreateApplicatorLicenseDto {
    fn from(dto: CreateApplicatorLicenseDto) -> Self {
        Self {
            user_id: dto.user_id,
            license_type: dto.license_type,
            license_number: dto.license_number,
            issued_by: dto.issued_by,
            valid_from: chrono::DateTime::parse_from_rfc3339(&dto.valid_from)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            valid_until: chrono::DateTime::parse_from_rfc3339(&dto.valid_until)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateApplicatorLicenseDto {
    pub license_type: Option<agrocore_domain::entities::plant_protection::LicenseType>,
    pub license_number: Option<String>,
    pub issued_by: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub is_active: Option<bool>,
}

impl From<UpdateApplicatorLicenseDto> for DomainUpdateApplicatorLicenseDto {
    fn from(dto: UpdateApplicatorLicenseDto) -> Self {
        Self {
            license_type: dto.license_type,
            license_number: dto.license_number,
            issued_by: dto.issued_by,
            valid_from: dto.valid_from.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            valid_until: dto.valid_until.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            is_active: dto.is_active,
        }
    }
}
