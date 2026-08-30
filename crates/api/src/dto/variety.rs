//! Variety DTOs

use agrocore_domain::entities::variety::{
    CreateVarietyDto as DomainCreateVarietyDto, UpdateVarietyDto as DomainUpdateVarietyDto,
    Variety, VarietyCategory,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedVarietyResponse {
    pub data: Vec<VarietyDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VarietyDto {
    pub id: Uuid,
    pub category: VarietyCategory,
    pub name: String,
    pub origin: Option<String>,
}

impl From<Variety> for VarietyDto {
    fn from(v: Variety) -> Self {
        Self {
            id: v.id,
            category: VarietyCategory::from_str(&v.category),
            name: v.name,
            origin: v.origin,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateVarietyDto {
    pub category: VarietyCategory,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub origin: Option<String>,
}

impl From<CreateVarietyDto> for DomainCreateVarietyDto {
    fn from(dto: CreateVarietyDto) -> Self {
        Self {
            category: dto.category,
            name: dto.name,
            origin: dto.origin,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateVarietyDto {
    pub category: Option<VarietyCategory>,
    pub name: Option<String>,
    pub origin: Option<String>,
}

impl From<UpdateVarietyDto> for DomainUpdateVarietyDto {
    fn from(dto: UpdateVarietyDto) -> Self {
        Self {
            category: dto.category,
            name: dto.name,
            origin: dto.origin,
        }
    }
}
