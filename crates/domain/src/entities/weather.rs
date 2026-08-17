use crate::entities::BbchStage;
use crate::entities::tenant::TenantId;
// use crate::entities::user::UserRole;
use crate::repositories::VisibilityAwareEntity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, strum::Display, strum::EnumString,
)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Growing Degree Day tracking for crop maturity prediction
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct GrowingDegreeDay {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub date: DateTime<Utc>,
    pub base_temp_c: f64,
    pub actual_mean_temp_c: f64,
    pub gdd: f64,
    pub accumulated_gdd: f64,
    pub crop_type: String,
    pub created_at: DateTime<Utc>,
}

/// Disease and pest risk assessment
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct PestRisk {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub assessment_date: DateTime<Utc>,
    #[sqlx(json)]
    pub risk_level: RiskLevel,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema, Default)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    #[default]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("Unknown risk level: {}", s)),
        }
    }
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
pub struct CreateFrostWarningDto {
    pub station_id: Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateFrostWarningDto {
    pub threshold_temp_c: Option<f64>,
    pub is_active: Option<bool>,
    pub notify_email: Option<bool>,
    pub notify_sms: Option<bool>,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateGrowingDegreeDayDto {
    pub site_id: Uuid,
    pub date: DateTime<Utc>,
    pub base_temp_c: f64,
    pub actual_mean_temp_c: f64,
    pub crop_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePestRiskDto {
    pub site_id: Uuid,
    pub assessment_date: DateTime<Utc>,
    pub risk_level: RiskLevel,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
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
