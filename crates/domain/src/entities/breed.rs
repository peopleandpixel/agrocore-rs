use serde::{Deserialize, Serialize};
use std::str::FromStr;
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
}

impl FromStr for Species {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Goat" => Ok(Species::Goat),
            "Sheep" => Ok(Species::Sheep),
            "Cattle" => Ok(Species::Cattle),
            "Chicken" => Ok(Species::Chicken),
            "Pig" => Ok(Species::Pig),
            "Horse" => Ok(Species::Horse),
            other => Ok(Species::Other(other.to_string())),
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
