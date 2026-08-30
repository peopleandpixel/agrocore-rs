use serde::{Deserialize, Serialize};
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

    pub fn from_str(s: &str) -> Self {
        match s {
            "Grape" => VarietyCategory::Grape,
            "Olive" => VarietyCategory::Olive,
            "Apple" => VarietyCategory::Apple,
            "Citrus" => VarietyCategory::Citrus,
            "Nut" => VarietyCategory::Nut,
            "Berry" => VarietyCategory::Berry,
            "Vegetable" => VarietyCategory::Vegetable,
            "Grain" => VarietyCategory::Grain,
            _ => VarietyCategory::Other,
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
