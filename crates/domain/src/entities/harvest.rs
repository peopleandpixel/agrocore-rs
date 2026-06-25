use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;
use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct HarvestSeason {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub year: i32,
    pub label: String, // e.g., "Ernte 2026"
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct HarvestLot {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub season_id: Uuid,
    pub lot_number: String, // Eindeutige Chargennummer
    pub site_ids: Vec<Uuid>, // Herkunft (Parzellen)
    pub crop_type: String,   // z.B. "Grape", "Olive"
    pub variety: Option<String>,
    pub quality_target: Option<String>,
    pub total_weight_kg: f64,
    pub status: LotStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum LotStatus {
    #[serde(rename = "collecting")] Collecting,
    #[serde(rename = "processed")] Processed,
    #[serde(rename = "shipped")] Shipped,
    #[serde(rename = "stored")] Stored,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct HarvestDelivery {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub lot_id: Uuid,
    pub delivery_date: DateTime<Utc>,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ColdChainLog {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub lot_id: Uuid,
    pub sensor_id: String,
    pub recorded_at: DateTime<Utc>,
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}

// DTOs for Creation

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateHarvestSeasonDto {
    pub year: i32,
    #[validate(length(min = 1))]
    pub label: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateHarvestLotDto {
    pub season_id: Uuid,
    #[validate(length(min = 1))]
    pub lot_number: String,
    pub site_ids: Vec<Uuid>,
    pub crop_type: String,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateHarvestDeliveryDto {
    pub lot_id: Uuid,
    pub delivery_date: DateTime<Utc>,
    #[validate(range(min = 0.0))]
    pub gross_weight_kg: f64,
    #[validate(range(min = 0.0))]
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateColdChainLogDto {
    pub lot_id: Uuid,
    pub sensor_id: String,
    pub recorded_at: DateTime<Utc>,
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}
