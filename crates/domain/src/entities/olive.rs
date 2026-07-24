use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
pub struct OliveGrove {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub label: String,
    pub variety: String,
    pub tree_count: Option<i32>,
    pub planting_year: Option<i32>,
    pub area_ha: f64,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
    pub organic_certified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow, ToSchema)]
pub struct OliveOilRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub grove_id: Uuid,
    pub harvest_year: i32,
    #[sqlx(json)]
    pub oil_grade: OilGrade,
    pub acidity_pct: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub sensory_score: Option<f64>,
    pub liters_produced: Option<f64>,
    pub mill_name: Option<String>,
    pub lot_number: Option<String>,
    pub harvest_date: DateTime<Utc>,
    pub quantity_kg: Option<f64>,
    pub oil_yield_kg: Option<f64>,
    pub oil_yield_percent: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum OilGrade {
    ExtraVirgin,
    Virgin,
    Lampante,
    Refined,
    Pomace,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateOliveGroveDto {
    pub site_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[validate(length(min = 1))]
    pub variety: String,
    pub planting_year: Option<i32>,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateOliveGroveDto {
    pub label: Option<String>,
    pub variety: Option<String>,
    pub planting_year: Option<i32>,
    pub area_ha: Option<f64>,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateOliveOilRecordDto {
    pub grove_id: Uuid,
    pub harvest_year: i32,
    pub oil_grade: OilGrade,
    pub acidity_pct: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub sensory_score: Option<f64>,
    pub liters_produced: Option<f64>,
    pub mill_name: Option<String>,
    pub lot_number: Option<String>,
    pub harvest_date: DateTime<Utc>,
    pub quantity_kg: Option<f64>,
    pub oil_yield_kg: Option<f64>,
    pub oil_yield_percent: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateOliveOilRecordDto {
    pub grove_id: Option<Uuid>,
    pub harvest_year: Option<i32>,
    pub oil_grade: Option<OilGrade>,
    pub acidity_pct: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub sensory_score: Option<f64>,
    pub liters_produced: Option<f64>,
    pub mill_name: Option<String>,
    pub lot_number: Option<String>,
    pub harvest_date: Option<DateTime<Utc>>,
    pub quantity_kg: Option<f64>,
    pub oil_yield_kg: Option<f64>,
    pub oil_yield_percent: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}
