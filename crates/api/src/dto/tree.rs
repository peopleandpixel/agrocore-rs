//! Tree DTOs

use agrocore_domain::entities::tree::{
    CreateTreeDto as DomainCreateTreeDto, Tree, TreeType, UpdateTreeDto as DomainUpdateTreeDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedTreeResponse {
    pub data: Vec<TreeDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TreeDto {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub group_id: Option<String>,
    pub tree_type: TreeType,
    pub count: i32,
    pub label: Option<String>,
}

impl From<Tree> for TreeDto {
    fn from(t: Tree) -> Self {
        Self {
            id: t.id,
            plot_id: t.plot_id,
            group_id: t.group_id,
            tree_type: TreeType::from_str(&t.tree_type),
            count: t.count,
            label: t.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateTreeDto {
    pub plot_id: Uuid,
    pub group_id: Option<String>,
    pub tree_type: TreeType,
    pub count: u32,
    pub label: Option<String>,
}

impl From<CreateTreeDto> for DomainCreateTreeDto {
    fn from(dto: CreateTreeDto) -> Self {
        Self {
            plot_id: dto.plot_id,
            group_id: dto.group_id,
            tree_type: dto.tree_type,
            count: dto.count,
            label: dto.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateTreeDto {
    pub group_id: Option<String>,
    pub tree_type: Option<TreeType>,
    pub count: Option<u32>,
    pub label: Option<String>,
}

impl From<UpdateTreeDto> for DomainUpdateTreeDto {
    fn from(dto: UpdateTreeDto) -> Self {
        Self {
            group_id: dto.group_id,
            tree_type: dto.tree_type,
            count: dto.count,
            label: dto.label,
        }
    }
}
