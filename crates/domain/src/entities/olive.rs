use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct OliveGrove {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub variety: Option<String>,
    pub tree_count: Option<i32>,
    pub planting_year: Option<i32>,
    pub organic_certified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum OilGrade {
    ExtraVirgin,
    Virgin,
    Lampante,
    Refined,
    Pomace,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOliveGroveDto {
    pub site_id: Uuid,
    pub variety: Option<String>,
    pub tree_count: Option<i32>,
    pub planting_year: Option<i32>,
    pub organic_certified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateOliveGroveDto {
    pub variety: Option<String>,
    pub tree_count: Option<i32>,
    pub planting_year: Option<i32>,
    pub organic_certified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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
}
