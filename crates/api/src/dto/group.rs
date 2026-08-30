//! Group DTOs

use agrocore_domain::entities::group::{
    CreateGroupDto as DomainCreateGroupDto, Group, GroupType,
    UpdateGroupDto as DomainUpdateGroupDto,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedGroupResponse {
    pub data: Vec<GroupDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GroupDto {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub parent_group_id: Option<Uuid>,
    pub group_type: GroupType,
    pub label: String,
}

impl From<Group> for GroupDto {
    fn from(g: Group) -> Self {
        Self {
            id: g.id,
            plot_id: g.plot_id,
            parent_group_id: g.parent_group_id,
            group_type: GroupType::from_str(&g.group_type),
            label: g.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateGroupDto {
    pub plot_id: Uuid,
    pub parent_group_id: Option<Uuid>,
    pub group_type: GroupType,
    #[validate(length(min = 1, max = 255))]
    pub label: String,
}

impl From<CreateGroupDto> for DomainCreateGroupDto {
    fn from(dto: CreateGroupDto) -> Self {
        Self {
            plot_id: dto.plot_id,
            parent_group_id: dto.parent_group_id,
            group_type: dto.group_type,
            label: dto.label,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateGroupDto {
    pub parent_group_id: Option<Uuid>,
    pub group_type: Option<GroupType>,
    pub label: Option<String>,
}

impl From<UpdateGroupDto> for DomainUpdateGroupDto {
    fn from(dto: UpdateGroupDto) -> Self {
        Self {
            parent_group_id: dto.parent_group_id,
            group_type: dto.group_type,
            label: dto.label,
        }
    }
}
