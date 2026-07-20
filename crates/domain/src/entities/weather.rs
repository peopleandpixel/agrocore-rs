use crate::entities::BbchStage;
use crate::entities::tenant::TenantId;
// use crate::entities::user::UserRole;
use crate::repositories::VisibilityAwareEntity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum WeatherStationType {
    #[serde(rename = "iot")]
    Iot,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "virtual")]
    Virtual,
    #[serde(rename = "external_api")]
    ExternalApi,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
#[schema(example = json!({"id": "550e8400-e29b-41d4-a716-446655440000", "tenant_id": "550e8400-e29b-41d4-a716-446655440000", "label": "Main Station", "station_type": "iot", "is_active": true, "created_at": "2023-01-01T00:00:00Z", "updated_at": "2023-01-01T00:00:00Z"}))]
pub struct WeatherStation {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    #[sqlx(json)]
    pub station_type: WeatherStationType,
    #[sqlx(skip)]
    pub location: Option<crate::entities::site::GeoPoint>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
    pub is_active: bool,
    #[sqlx(json)]
    pub sensor_metadata: Option<serde_json::Value>,
    pub firmware_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for WeatherStation {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct WeatherData {
    pub id: Uuid,
    pub station_id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub timestamp: DateTime<Utc>,
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_direction_deg: Option<i32>,
    pub solar_radiation_wm2: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub soil_temperature_c: Option<f64>,
    pub soil_moisture_percent: Option<f64>,
    pub leaf_wetness: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct FrostWarning {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub station_id: Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct PhenologyRecord {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub observation_date: DateTime<Utc>,
    #[sqlx(json)]
    pub stage: BbchStage,
    pub forecast_next_stage_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
    pub observer_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWeatherStationDto {
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    pub station_type: WeatherStationType,
    pub location: Option<crate::entities::site::GeoPoint>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWeatherDataDto {
    pub station_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_direction_deg: Option<i32>,
    pub solar_radiation_wm2: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub soil_temperature_c: Option<f64>,
    pub soil_moisture_percent: Option<f64>,
    pub leaf_wetness: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePhenologyRecordDto {
    pub site_id: Uuid,
    pub observation_date: DateTime<Utc>,
    pub stage: BbchStage,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateWeatherStationDto {
    pub label: Option<String>,
    pub station_type: Option<WeatherStationType>,
    pub location: Option<crate::entities::site::GeoPoint>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
    pub is_active: Option<bool>,
    pub sensor_metadata: Option<serde_json::Value>,
    pub firmware_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateWeatherDataDto {
    pub station_id: Option<Uuid>,
    pub timestamp: Option<DateTime<Utc>>,
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_direction_deg: Option<i32>,
    pub solar_radiation_wm2: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub soil_temperature_c: Option<f64>,
    pub soil_moisture_percent: Option<f64>,
    pub leaf_wetness: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePhenologyRecordDto {
    pub site_id: Option<Uuid>,
    pub observation_date: Option<DateTime<Utc>>,
    pub stage: Option<BbchStage>,
    pub forecast_next_stage_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
    pub observer_id: Option<Uuid>,
}
