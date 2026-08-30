use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum GroupType {
    #[default]
    Herd,
    Flock,
    Grove,
    Coop,
    Barn,
    Other(String),
}

impl GroupType {
    pub fn as_str(&self) -> &str {
        match self {
            GroupType::Herd => "Herd",
            GroupType::Flock => "Flock",
            GroupType::Grove => "Grove",
            GroupType::Coop => "Coop",
            GroupType::Barn => "Barn",
            GroupType::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Herd" => GroupType::Herd,
            "Flock" => GroupType::Flock,
            "Grove" => GroupType::Grove,
            "Coop" => GroupType::Coop,
            "Barn" => GroupType::Barn,
            other => GroupType::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Group {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub parent_group_id: Option<Uuid>,
    pub group_type: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateGroupDto {
    pub plot_id: Uuid,
    pub parent_group_id: Option<Uuid>,
    pub group_type: GroupType,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateGroupDto {
    pub parent_group_id: Option<Uuid>,
    pub group_type: Option<GroupType>,
    pub label: Option<String>,
}
