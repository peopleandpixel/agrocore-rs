use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AnimalSpecies {
    Goat,
    Chicken,
    Sheep,
    Cattle,
    Pig,
    Horse,
    Duck,
    Turkey,
    Goose,
    Other(String),
}

impl AnimalSpecies {
    pub fn as_str(&self) -> &str {
        match self {
            AnimalSpecies::Goat => "goat",
            AnimalSpecies::Chicken => "chicken",
            AnimalSpecies::Sheep => "sheep",
            AnimalSpecies::Cattle => "cattle",
            AnimalSpecies::Pig => "pig",
            AnimalSpecies::Horse => "horse",
            AnimalSpecies::Duck => "duck",
            AnimalSpecies::Turkey => "turkey",
            AnimalSpecies::Goose => "goose",
            AnimalSpecies::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "goat" => AnimalSpecies::Goat,
            "chicken" => AnimalSpecies::Chicken,
            "sheep" => AnimalSpecies::Sheep,
            "cattle" => AnimalSpecies::Cattle,
            "pig" => AnimalSpecies::Pig,
            "horse" => AnimalSpecies::Horse,
            "duck" => AnimalSpecies::Duck,
            "turkey" => AnimalSpecies::Turkey,
            "goose" => AnimalSpecies::Goose,
            other => AnimalSpecies::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AnimalStatus {
    Active,
    Inactive,
    Sold,
    Deceased,
    Other(String),
}

impl AnimalStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AnimalStatus::Active => "Active",
            AnimalStatus::Inactive => "Inactive",
            AnimalStatus::Sold => "Sold",
            AnimalStatus::Deceased => "Deceased",
            AnimalStatus::Other(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Active" => AnimalStatus::Active,
            "Inactive" => AnimalStatus::Inactive,
            "Sold" => AnimalStatus::Sold,
            "Deceased" => AnimalStatus::Deceased,
            other => AnimalStatus::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct TreatmentRecord {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub treatment_type: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub medication: String,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct GrazingRecord {
    pub id: Uuid,
    pub animal_id: Uuid,
    pub plot_id: Uuid,
    pub site_id: Option<Uuid>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow, ToSchema)]
pub struct Animal {
    pub id: Uuid,
    pub identifier: String,
    pub plot_id: Option<Uuid>,
    pub livestock_type: String,
    pub species: String,
    pub breed: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub status: String,
    pub current_site_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateAnimalDto {
    pub identifier: String,
    pub plot_id: Option<Uuid>,
    pub livestock_type: String,
    pub species: String,
    pub breed: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub status: String,
    pub current_site_id: Option<Uuid>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub tenant_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, Default)]
pub struct UpdateAnimalDto {
    pub identifier: Option<String>,
    pub plot_id: Option<Uuid>,
    pub livestock_type: Option<String>,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub status: Option<String>,
    pub current_site_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub weight_kg: Option<f64>,
    pub mother_id: Option<Uuid>,
    pub father_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateGrazingRecordDto {
    pub plot_id: Uuid,
    pub site_id: Option<Uuid>,
    pub animal_id: Option<Uuid>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub start_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateTreatmentRecordDto {
    pub animal_id: Uuid,
    pub treatment_type: String,
    pub medication: String,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum LivestockType {
    #[serde(rename = "goat")]
    Goat,
    #[serde(rename = "chicken")]
    Chicken,
    #[serde(rename = "sheep")]
    Sheep,
    #[serde(rename = "cattle")]
    Cattle,
    #[serde(rename = "pig")]
    Pig,
    #[serde(rename = "horse")]
    Horse,
    #[serde(rename = "duck")]
    Duck,
    #[serde(rename = "turkey")]
    Turkey,
    #[serde(rename = "goose")]
    Goose,
    #[serde(rename = "other")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, sqlx::FromRow)]
pub struct Livestock {
    pub id: Uuid,
    pub plot_id: Uuid,
    pub herd_id: Option<String>,
    pub livestock_type: String,
    pub count: i32,
    pub label: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct CreateLivestockDto {
    pub plot_id: Uuid,
    pub herd_id: Option<String>,
    pub livestock_type: String,
    pub count: i32,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub struct UpdateLivestockDto {
    pub herd_id: Option<String>,
    pub livestock_type: Option<String>,
    pub count: Option<i32>,
    pub label: Option<String>,
}
