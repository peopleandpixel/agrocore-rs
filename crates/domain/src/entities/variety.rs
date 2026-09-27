use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum VarietyCategory {
    #[default]
    Grape,
    Olive,
    Apple,
    Citrus,
    Nut,
    Berry,
    Vegetable,
    Grain,
    Other,
}

impl VarietyCategory {
    pub fn as_str(&self) -> &str {
        match self {
            VarietyCategory::Grape => "Grape",
            VarietyCategory::Olive => "Olive",
            VarietyCategory::Apple => "Apple",
            VarietyCategory::Citrus => "Citrus",
            VarietyCategory::Nut => "Nut",
            VarietyCategory::Berry => "Berry",
            VarietyCategory::Vegetable => "Vegetable",
            VarietyCategory::Grain => "Grain",
            VarietyCategory::Other => "Other",
        }
    }
}

impl FromStr for VarietyCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Grape" => Ok(VarietyCategory::Grape),
            "Olive" => Ok(VarietyCategory::Olive),
            "Apple" => Ok(VarietyCategory::Apple),
            "Citrus" => Ok(VarietyCategory::Citrus),
            "Nut" => Ok(VarietyCategory::Nut),
            "Berry" => Ok(VarietyCategory::Berry),
            "Vegetable" => Ok(VarietyCategory::Vegetable),
            "Grain" => Ok(VarietyCategory::Grain),
            _ => Ok(VarietyCategory::Other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Variety {
    pub id: Uuid,
    pub category: String,
    pub name: String,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateVarietyDto {
    pub category: VarietyCategory,
    pub name: String,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateVarietyDto {
    pub category: Option<VarietyCategory>,
    pub name: Option<String>,
    pub origin: Option<String>,
}
