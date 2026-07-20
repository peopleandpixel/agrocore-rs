//! Finance DTOs

use agrocore_domain::entities::finance::{
    CostCenter, CostCenterType, CreateCostCenterDto as DomainCreateCostCenterDto,
    CreateFinancialRecordDto as DomainCreateFinancialRecordDto,
    CreatePACApplicationDto as DomainCreatePACApplicationDto, EcoSchemeParticipation,
    FinancialRecord, FinancialRecordType, PACApplication, UpdateCostCenterDto as DomainUpdateCostCenterDto,
    UpdateFinancialRecordDto as DomainUpdateFinancialRecordDto,
    UpdatePACApplicationDto as DomainUpdatePACApplicationDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPACApplicationResponse {
    pub data: Vec<PACApplicationDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PACApplicationDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub year: i32,
    pub application_number: String,
    pub total_eligible_area: f64,
    pub eco_schemes: serde_json::Value,
    pub status: String,
    pub submitted_at: Option<String>,
    pub approved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PACApplication> for PACApplicationDto {
    fn from(p: PACApplication) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id.into(),
            year: p.year,
            application_number: p.application_number,
            total_eligible_area: p.total_eligible_area,
            eco_schemes: p.eco_schemes,
            status: p.status.to_string(),
            submitted_at: p.submitted_at.map(|d| d.to_rfc3339()),
            approved_at: p.approved_at.map(|d| d.to_rfc3339()),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreatePACApplicationDto {
    pub year: i32,
    pub application_number: String,
    #[validate(range(min = 0.0))]
    pub total_eligible_area: f64,
    pub eco_schemes: serde_json::Value,
}

impl From<CreatePACApplicationDto> for DomainCreatePACApplicationDto {
    fn from(dto: CreatePACApplicationDto) -> Self {
        let eco_schemes: Option<Vec<agrocore_domain::entities::finance::EcoSchemeParticipation>> = 
            if dto.eco_schemes.is_null() {
                None
            } else {
                serde_json::from_value(dto.eco_schemes).ok()
            };

        Self {
            year: dto.year,
            application_number: dto.application_number,
            total_eligible_area: dto.total_eligible_area,
            eco_schemes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePACApplicationDto {
    pub status: Option<String>,
    pub eco_schemes: Option<serde_json::Value>,
}

impl From<UpdatePACApplicationDto> for DomainUpdatePACApplicationDto {
    fn from(dto: UpdatePACApplicationDto) -> Self {
        let eco_schemes: Option<Vec<agrocore_domain::entities::finance::EcoSchemeParticipation>> = 
            if dto.eco_schemes.as_ref().map_or(true, |v| v.is_null()) {
                None
            } else {
                serde_json::from_value(dto.eco_schemes.unwrap_or(serde_json::Value::Null)).ok()
            };

        Self {
            application_number: dto.application_number,
            status: dto.status,
            total_eligible_area: dto.total_eligible_area,
            eco_schemes,
            documents_urls: dto.documents_urls,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedCostCenterResponse {
    pub data: Vec<CostCenterDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CostCenterDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub code: String,
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CostCenter> for CostCenterDto {
    fn from(c: CostCenter) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id.into(),
            label: c.label,
            code: c.code,
            cost_center_type: c.cost_center_type,
            reference_id: c.reference_id,
            parent_id: c.parent_id,
            is_active: c.is_active,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateCostCenterDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[validate(length(min = 1, max = 50))]
    pub code: String,
    pub cost_center_type: CostCenterType,
    pub reference_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
}

impl From<CreateCostCenterDto> for DomainCreateCostCenterDto {
    fn from(dto: CreateCostCenterDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            cost_center_type: dto.cost_center_type,
            reference_id: dto.reference_id,
            parent_id: dto.parent_id,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateCostCenterDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub code: Option<String>,
    pub cost_center_type: Option<CostCenterType>,
    pub reference_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

impl From<UpdateCostCenterDto> for DomainUpdateCostCenterDto {
    fn from(dto: UpdateCostCenterDto) -> Self {
        Self {
            label: dto.label,
            code: dto.code,
            cost_center_type: dto.cost_center_type,
            reference_id: dto.reference_id,
            parent_id: dto.parent_id,
            is_active: dto.is_active,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedFinancialRecordResponse {
    pub data: Vec<FinancialRecordDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FinancialRecordDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub cost_center_id: Uuid,
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub record_type: FinancialRecordType,
    pub category: String,
    pub description: String,
    pub reference_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::finance::FinancialRecord> for FinancialRecordDto {
    fn from(f: FinancialRecord) -> Self {
        Self {
            id: f.id,
            tenant_id: f.tenant_id.into(),
            cost_center_id: f.cost_center_id,
            date: f.date.to_rfc3339(),
            amount: f.amount,
            currency: f.currency,
            record_type: f.record_type,
            category: f.category,
            description: f.description,
            reference_id: f.reference_id,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateFinancialRecordDto {
    pub cost_center_id: Uuid,
    pub date: String,
    #[validate(range(min = -999999999.0))]
    pub amount: f64,
    pub currency: String,
    pub record_type: String,
    pub category: String,
    pub description: String,
    pub reference_id: Option<Uuid>,
}

impl From<CreateFinancialRecordDto> for DomainCreateFinancialRecordDto {
    fn from(dto: CreateFinancialRecordDto) -> Self {
        Self {
            cost_center_id: dto.cost_center_id,
            date: chrono::DateTime::parse_from_rfc3339(&dto.date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            amount: dto.amount,
            currency: dto.currency,
            record_type: dto.record_type,
            category: dto.category,
            description: dto.description,
            reference_id: dto.reference_id,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateFinancialRecordDto {
    pub cost_center_id: Option<Uuid>,
    pub date: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub record_type: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub reference_id: Option<Uuid>,
}

impl From<UpdateFinancialRecordDto> for DomainUpdateFinancialRecordDto {
    fn from(dto: UpdateFinancialRecordDto) -> Self {
        Self {
            cost_center_id: dto.cost_center_id,
            date: dto.date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            amount: dto.amount,
            currency: dto.currency,
            record_type: dto.record_type,
            category: dto.category,
            description: dto.description,
            reference_id: dto.reference_id,
        }
    }
}
