use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;
use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AnimalSpecies {
    #[serde(rename = "cattle")] Cattle,
    #[serde(rename = "sheep")] Sheep,
    #[serde(rename = "goat")] Goat,
    #[serde(rename = "pig")] Pig,
    #[serde(rename = "poultry")] Poultry,
    #[serde(rename = "horse")] Horse,
    #[serde(rename = "other")] Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum AnimalStatus {
    #[serde(rename = "active")] Active,
    #[serde(rename = "sold")] Sold,
    #[serde(rename = "deceased")] Deceased,
    #[serde(rename = "quarantine")] Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Animal {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub species: AnimalSpecies,
    pub breed: Option<String>,
    #[validate(length(min = 1))]
    pub identifier: String, // Ohrmarke, Name, etc.
    pub birth_date: Option<DateTime<Utc>>,
    pub gender: Option<String>,
    pub status: AnimalStatus,
    pub current_site_id: Option<Uuid>, // Aktuelle Weide/Stall
    pub group_id: Option<Uuid>,
    pub weight_kg: Option<f64>,
    pub last_weight_date: Option<DateTime<Utc>>,
    pub treatments: Vec<TreatmentRecord>,
    pub grazing_history: Vec<GrazingRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct TreatmentRecord {
    pub id: Uuid,
    pub date: DateTime<Utc>,
    pub treatment_type: String, // Impfung, Entwurmung, etc.
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<u32>, // Wartezeit
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct GrazingRecord {
    pub site_id: Uuid,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAnimalDto {
    pub species: AnimalSpecies,
    pub breed: Option<String>,
    #[validate(length(min = 1))]
    pub identifier: String,
    pub birth_date: Option<DateTime<Utc>>,
    pub gender: Option<String>,
    pub current_site_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, Default)]
pub struct UpdateAnimalDto {
    pub breed: Option<String>,
    pub identifier: Option<String>,
    pub status: Option<AnimalStatus>,
    pub current_site_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub weight_kg: Option<f64>,
}
