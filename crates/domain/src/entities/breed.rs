use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum Species {
    #[default]
    Goat,
    Sheep,
    Cattle,
    Chicken,
    Pig,
    Horse,
    Other(String),
}

impl Species {
    pub fn as_str(&self) -> &str {
        match self {
            Species::Goat => "Goat",
            Species::Sheep => "Sheep",
            Species::Cattle => "Cattle",
            Species::Chicken => "Chicken",
            Species::Pig => "Pig",
            Species::Horse => "Horse",
            Species::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Goat" => Species::Goat,
            "Sheep" => Species::Sheep,
            "Cattle" => Species::Cattle,
            "Chicken" => Species::Chicken,
            "Pig" => Species::Pig,
            "Horse" => Species::Horse,
            other => Species::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Breed {
    pub id: Uuid,
    pub species: String,
    pub name: String,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateBreedDto {
    pub species: Species,
    pub name: String,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateBreedDto {
    pub species: Option<Species>,
    pub name: Option<String>,
    pub origin: Option<String>,
}
