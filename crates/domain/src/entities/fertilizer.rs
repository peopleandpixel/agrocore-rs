use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct FertilizerRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFertilizerRecordDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub product_name: String,
    #[validate(range(min = 0.0))]
    pub nutrient_n: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_p: f64,
    #[validate(range(min = 0.0))]
    pub nutrient_k: f64,
    #[validate(range(min = 0.0))]
    pub quantity_kg: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateFertilizerRecordDto {
    pub site_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub nutrient_n: Option<f64>,
    pub nutrient_p: Option<f64>,
    pub nutrient_k: Option<f64>,
    pub quantity_kg: Option<f64>,
    pub area_ha: Option<f64>,
    pub application_date: Option<DateTime<Utc>>,
}