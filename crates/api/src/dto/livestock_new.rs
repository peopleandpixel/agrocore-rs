//! Livestock DTOs (for herds/groups of animals - separate from individual Animal)

use agrocore_domain::entities::livestock::{
    CreateLivestockDto as DomainCreateLivestockDto, Livestock,
    UpdateLivestockDto as DomainUpdateLivestockDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedLivestockResponse {
    pub data: Vec<LivestockDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LivestockDto {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub herd_id: Option<String>,
    pub livestock_type: String,
    pub count: i32,
    pub label: Option<String>,
    pub created_at: String,
}

impl From<Livestock> for LivestockDto {
    fn from(l: Livestock) -> Self {
        Self {
            id: l.id,
            plot_id: l.plot_id,
            herd_id: l.herd_id,
            livestock_type: l.livestock_type,
            count: l.count,
            label: l.label,
            created_at: l.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateLivestockDto {
    pub plot_id: Uuid,
    pub herd_id: Option<String>,
    pub livestock_type: String,
    pub count: i32,
    pub label: Option<String>,
}

impl From<CreateLivestockDto> for DomainCreateLivestockDto {
    fn from(dto: CreateLivestockDto) -> Self {
        Self {
            plot_id: dto.plot_id,
            herd_id: dto.herd_id,
            livestock_type: dto.livestock_type,
            count: dto.count,
            label: dto.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateLivestockDto {
    pub herd_id: Option<String>,
    pub livestock_type: Option<String>,
    pub count: Option<i32>,
    pub label: Option<String>,
}

impl From<UpdateLivestockDto> for DomainUpdateLivestockDto {
    fn from(dto: UpdateLivestockDto) -> Self {
        Self {
            herd_id: dto.herd_id,
            livestock_type: dto.livestock_type,
            count: dto.count,
            label: dto.label,
        }
    }
}
