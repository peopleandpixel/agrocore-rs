use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum TreeType {
    #[default]
    CorkOak,
    Olive,
    Almond,
    Other(String),
}

impl TreeType {
    pub fn as_str(&self) -> &str {
        match self {
            TreeType::CorkOak => "CorkOak",
            TreeType::Olive => "Olive",
            TreeType::Almond => "Almond",
            TreeType::Other(s) => s,
        }
    }
}

impl FromStr for TreeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CorkOak" => Ok(TreeType::CorkOak),
            "Olive" => Ok(TreeType::Olive),
            "Almond" => Ok(TreeType::Almond),
            other => Ok(TreeType::Other(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Tree {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub group_id: Option<String>,
    pub tree_type: String,
    pub count: i32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateTreeDto {
    pub plot_id: Uuid,
    pub group_id: Option<String>,
    pub tree_type: TreeType,
    pub count: u32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateTreeDto {
    pub group_id: Option<String>,
    pub tree_type: Option<TreeType>,
    pub count: Option<u32>,
    pub label: Option<String>,
}
