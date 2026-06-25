use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Vineyard {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<QualityGrade>,
    pub kelter_delivery: Option<KelterDelivery>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum DocArea {
    Douro,
    Alentejo,
    VinhoVerde,
    Dao,
    Bairrada,
    Lisboa,
    PeninsulaDeSetubal,
    Algarve,
    Tejo,
    Minho,
    Trasmontes,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum QualityGrade {
    Reserva,
    GrandeReserva,
    Garrafeira,
    Superior,
    Classic,
    LateHarvest,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct KelterDelivery {
    pub id: Uuid,
    pub vineyard_id: Uuid,
    pub delivery_date: DateTime<Utc>,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub lot_number: String,
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateVineyardDto {
    pub site_id: Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<QualityGrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateKelterDeliveryDto {
    pub vineyard_id: Uuid,
    pub delivery_date: DateTime<Utc>,
    #[validate(range(min = 0.0))]
    pub gross_weight_kg: f64,
    #[validate(range(min = 0.0))]
    pub net_weight_kg: f64,
    #[validate(length(min = 1))]
    pub lot_number: String,
    #[validate(length(min = 1))]
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}
