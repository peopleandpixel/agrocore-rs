//! Building DTOs

use agrocore_domain::entities::building::{
    Building, BuildingType, CreateBuildingDto as DomainCreateBuildingDto,
    UpdateBuildingDto as DomainUpdateBuildingDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedBuildingResponse {
    pub data: Vec<BuildingDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BuildingDto {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub building_type: BuildingType,
    pub label: Option<String>,
}

impl From<Building> for BuildingDto {
    fn from(b: Building) -> Self {
        Self {
            id: b.id,
            plot_id: b.plot_id,
            building_type: BuildingType::from_str(&b.building_type),
            label: b.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateBuildingDto {
    pub plot_id: Uuid,
    pub building_type: BuildingType,
    pub label: Option<String>,
}

impl From<CreateBuildingDto> for DomainCreateBuildingDto {
    fn from(dto: CreateBuildingDto) -> Self {
        Self {
            plot_id: dto.plot_id,
            building_type: dto.building_type,
            label: dto.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateBuildingDto {
    pub building_type: Option<BuildingType>,
    pub label: Option<String>,
}

impl From<UpdateBuildingDto> for DomainUpdateBuildingDto {
    fn from(dto: UpdateBuildingDto) -> Self {
        Self {
            building_type: dto.building_type,
            label: dto.label,
        }
    }
}
