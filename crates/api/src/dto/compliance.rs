//! Compliance DTOs

use agrocore_domain::entities::compliance::{
    ChecklistItem, ChecklistType, ComplianceChecklist, ComplianceStatus,
    CreateComplianceChecklistDto as DomainCreateComplianceChecklistDto,
    UpdateComplianceChecklistDto as DomainUpdateComplianceChecklistDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedComplianceChecklistResponse {
    pub data: Vec<ComplianceChecklistDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ComplianceChecklistDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub checklist_type: ChecklistType,
    pub status: ComplianceStatus,
    pub items: Vec<ChecklistItemDto>,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ChecklistItemDto {
    pub id: Uuid,
    pub label: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub completed_by: Option<uuid::Uuid>,
    pub evidence_url: Option<String>,
}

impl From<ComplianceChecklist> for ComplianceChecklistDto {
    fn from(c: ComplianceChecklist) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id.into(),
            site_id: c.site_id,
            checklist_type: c.checklist_type,
            status: c.status,
            items: c.items.into_iter().map(ChecklistItemDto::from).collect(),
            due_date: c.due_date.map(|d| d.to_rfc3339()),
            completed_at: c.completed_at.map(|d| d.to_rfc3339()),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

impl From<ChecklistItem> for ChecklistItemDto {
    fn from(item: ChecklistItem) -> Self {
        Self {
            id: item.id,
            label: item.label,
            description: item.description,
            is_completed: item.is_completed,
            completed_at: item.completed_at.map(|d| d.to_rfc3339()),
            completed_by: item.completed_by,
            evidence_url: item.evidence_url,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateComplianceChecklistDto {
    pub site_id: Uuid,
    pub checklist_type: ChecklistType,
    pub items: Vec<ChecklistItemDto>,
    pub due_date: Option<String>,
}

impl From<CreateComplianceChecklistDto> for DomainCreateComplianceChecklistDto {
    fn from(dto: CreateComplianceChecklistDto) -> Self {
        Self {
            site_id: dto.site_id,
            checklist_type: dto.checklist_type,
            items: dto.items.into_iter().map(Into::into).collect(),
            due_date: dto.due_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

impl From<ChecklistItemDto> for agrocore_domain::entities::compliance::ChecklistItem {
    fn from(item: ChecklistItemDto) -> Self {
        Self {
            id: item.id,
            label: item.label,
            description: item.description,
            is_completed: item.is_completed,
            completed_at: item.completed_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            completed_by: item.completed_by,
            evidence_url: item.evidence_url,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateComplianceChecklistDto {
    pub status: Option<ComplianceStatus>,
    pub items: Option<Vec<ChecklistItemDto>>,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
}

impl From<UpdateComplianceChecklistDto> for DomainUpdateComplianceChecklistDto {
    fn from(dto: UpdateComplianceChecklistDto) -> Self {
        Self {
            status: dto.status,
            items: dto
                .items
                .map(|items| items.into_iter().map(Into::into).collect()),
            due_date: dto.due_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            completed_at: dto.completed_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}

// Fertilizer Record DTOs
use agrocore_domain::entities::compliance::FertilizerRecord;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedFertilizerRecordResponse {
    pub data: Vec<FertilizerRecordDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FertilizerRecordDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    pub product_name: String,
    pub nutrient_n: f64,
    pub nutrient_p: f64,
    pub nutrient_k: f64,
    pub quantity_kg: f64,
    pub area_ha: f64,
    pub application_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FertilizerRecord> for FertilizerRecordDto {
    fn from(f: FertilizerRecord) -> Self {
        Self {
            id: f.id,
            tenant_id: f.tenant_id.into(),
            site_id: f.site_id,
            order_id: f.order_id,
            product_name: f.product_name,
            nutrient_n: f.nutrient_n,
            nutrient_p: f.nutrient_p,
            nutrient_k: f.nutrient_k,
            quantity_kg: f.quantity_kg,
            area_ha: f.area_ha,
            application_date: f.application_date.to_rfc3339(),
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateFertilizerRecordDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: String,
}

impl From<CreateFertilizerRecordDto>
    for agrocore_domain::entities::compliance::CreateFertilizerRecordDto
{
    fn from(dto: CreateFertilizerRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            nutrient_n: dto.nutrient_n,
            nutrient_p: dto.nutrient_p,
            nutrient_k: dto.nutrient_k,
            quantity_kg: dto.quantity_kg,
            area_ha: dto.area_ha,
            application_date: chrono::DateTime::parse_from_rfc3339(&dto.application_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateFertilizerRecordDto {
    pub site_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub nutrient_n: Option<f64>,
    pub nutrient_p: Option<f64>,
    pub nutrient_k: Option<f64>,
    pub quantity_kg: Option<f64>,
    pub area_ha: Option<f64>,
    pub application_date: Option<String>,
}

impl From<UpdateFertilizerRecordDto>
    for agrocore_domain::entities::fertilizer::UpdateFertilizerRecordDto
{
    fn from(dto: UpdateFertilizerRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            order_id: dto.order_id,
            product_name: dto.product_name,
            nutrient_n: dto.nutrient_n,
            nutrient_p: dto.nutrient_p,
            nutrient_k: dto.nutrient_k,
            quantity_kg: dto.quantity_kg,
            area_ha: dto.area_ha,
            application_date: dto.application_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
        }
    }
}
