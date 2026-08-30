use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum BuildingType {
    #[default]
    ChickenCoop,
    GoatStable,
    Barn,
    Other(String),
}

impl BuildingType {
    pub fn as_str(&self) -> &str {
        match self {
            BuildingType::ChickenCoop => "ChickenCoop",
            BuildingType::GoatStable => "GoatStable",
            BuildingType::Barn => "Barn",
            BuildingType::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ChickenCoop" => BuildingType::ChickenCoop,
            "GoatStable" => BuildingType::GoatStable,
            "Barn" => BuildingType::Barn,
            other => BuildingType::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Building {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub building_type: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateBuildingDto {
    pub plot_id: Uuid,
    pub building_type: BuildingType,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateBuildingDto {
    pub building_type: Option<BuildingType>,
    pub label: Option<String>,
}
