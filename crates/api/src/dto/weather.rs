//! Weather DTOs

use agrocore_domain::entities::BbchStage;
use agrocore_domain::entities::weather::{
    CreatePhenologyRecordDto as DomainCreatePhenologyRecordDto,
    CreateWeatherDataDto as DomainCreateWeatherDataDto,
    CreateWeatherStationDto as DomainCreateWeatherStationDto, PhenologyRecord,
    UpdatePhenologyRecordDto as DomainUpdatePhenologyRecordDto,
    UpdateWeatherDataDto as DomainUpdateWeatherDataDto,
    UpdateWeatherStationDto as DomainUpdateWeatherStationDto, WeatherData, WeatherStation,
    WeatherStationType,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWeatherStationResponse {
    pub data: Vec<WeatherStationDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WeatherStationDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub station_type: String,
    pub location: Option<serde_json::Value>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
    pub is_active: bool,
    pub sensor_metadata: Option<serde_json::Value>,
    pub firmware_version: Option<String>,
}

impl From<WeatherStation> for WeatherStationDto {
    fn from(w: WeatherStation) -> Self {
        Self {
            id: w.id,
            tenant_id: w.tenant_id.into(),
            label: w.label,
            station_type: w.station_type.to_string(),
            location: w.location.and_then(|g| serde_json::to_value(g).ok()),
            manufacturer: w.manufacturer,
            model: w.model,
            serial_number: w.serial_number,
            api_key_config: w.api_key_config,
            is_active: w.is_active,
            sensor_metadata: w.sensor_metadata,
            firmware_version: w.firmware_version,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWeatherStationDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub station_type: String,
    pub location: Option<serde_json::Value>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
}

impl From<CreateWeatherStationDto> for DomainCreateWeatherStationDto {
    fn from(dto: CreateWeatherStationDto) -> Self {
        Self {
            label: dto.label,
            station_type: dto.station_type.parse().unwrap_or(WeatherStationType::Iot),
            location: dto.location.and_then(|v| serde_json::from_value(v).ok()),
            manufacturer: dto.manufacturer,
            model: dto.model,
            serial_number: dto.serial_number,
            api_key_config: dto.api_key_config,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateWeatherStationDto {
    #[validate(length(min = 1, max = 200))]
    pub label: Option<String>,
    pub station_type: Option<String>,
    pub location: Option<serde_json::Value>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
    pub is_active: Option<bool>,
    pub sensor_metadata: Option<serde_json::Value>,
    pub firmware_version: Option<String>,
}

impl From<UpdateWeatherStationDto> for DomainUpdateWeatherStationDto {
    fn from(dto: UpdateWeatherStationDto) -> Self {
        Self {
            label: dto.label,
            station_type: dto
                .station_type
                .map(|s| s.parse().unwrap_or(WeatherStationType::Iot)),
            location: dto.location.and_then(|v| serde_json::from_value(v).ok()),
            manufacturer: dto.manufacturer,
            model: dto.model,
            serial_number: dto.serial_number,
            api_key_config: dto.api_key_config,
            is_active: dto.is_active,
            sensor_metadata: dto.sensor_metadata,
            firmware_version: dto.firmware_version,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedWeatherDataResponse {
    pub data: Vec<WeatherDataDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WeatherDataDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub station_id: Uuid,
    pub timestamp: String,
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

impl From<WeatherData> for WeatherDataDto {
    fn from(w: WeatherData) -> Self {
        Self {
            id: w.id,
            tenant_id: w.tenant_id.into(),
            station_id: w.station_id,
            timestamp: w.timestamp.to_rfc3339(),
            temperature_c: w.temperature_c,
            humidity_percent: w.humidity_percent,
            precipitation_mm: w.precipitation_mm,
            wind_speed_kmh: w.wind_speed_kmh,
            wind_direction_deg: w.wind_direction_deg,
            solar_radiation_wm2: w.solar_radiation_wm2,
            pressure_hpa: w.pressure_hpa,
            soil_temperature_c: w.soil_temperature_c,
            soil_moisture_percent: w.soil_moisture_percent,
            leaf_wetness: w.leaf_wetness,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateWeatherDataDto {
    pub station_id: Uuid,
    pub timestamp: String,
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

impl From<CreateWeatherDataDto> for DomainCreateWeatherDataDto {
    fn from(dto: CreateWeatherDataDto) -> Self {
        Self {
            station_id: dto.station_id,
            timestamp: chrono::DateTime::parse_from_rfc3339(&dto.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            temperature_c: dto.temperature_c,
            humidity_percent: dto.humidity_percent,
            precipitation_mm: dto.precipitation_mm,
            wind_speed_kmh: dto.wind_speed_kmh,
            wind_direction_deg: dto.wind_direction_deg,
            solar_radiation_wm2: dto.solar_radiation_wm2,
            pressure_hpa: dto.pressure_hpa,
            soil_temperature_c: dto.soil_temperature_c,
            soil_moisture_percent: dto.soil_moisture_percent,
            leaf_wetness: dto.leaf_wetness,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWeatherDataDto {
    pub station_id: Option<Uuid>,
    pub timestamp: Option<String>,
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

impl From<UpdateWeatherDataDto> for DomainUpdateWeatherDataDto {
    fn from(dto: UpdateWeatherDataDto) -> Self {
        Self {
            station_id: dto.station_id,
            timestamp: dto.timestamp.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            temperature_c: dto.temperature_c,
            humidity_percent: dto.humidity_percent,
            precipitation_mm: dto.precipitation_mm,
            wind_speed_kmh: dto.wind_speed_kmh,
            wind_direction_deg: dto.wind_direction_deg,
            solar_radiation_wm2: dto.solar_radiation_wm2,
            pressure_hpa: dto.pressure_hpa,
            soil_temperature_c: dto.soil_temperature_c,
            soil_moisture_percent: dto.soil_moisture_percent,
            leaf_wetness: dto.leaf_wetness,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPhenologyResponse {
    pub data: Vec<PhenologyRecordDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PhenologyRecordDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub observation_date: String,
    pub stage: BbchStage,
    pub forecast_next_stage_date: Option<String>,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
    pub observer_id: Option<Uuid>,
    pub observation_date_str: String,
}

impl From<PhenologyRecord> for PhenologyRecordDto {
    fn from(p: PhenologyRecord) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id.into(),
            site_id: p.site_id,
            observation_date: p.observation_date.to_rfc3339(),
            stage: p.stage,
            forecast_next_stage_date: p.forecast_next_stage_date.map(|d| d.to_rfc3339()),
            notes: p.notes,
            photo_url: p.photo_url,
            observer_id: p.observer_id,
            observation_date_str: p.observation_date.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreatePhenologyRecordDto {
    pub site_id: Uuid,
    pub observation_date: String,
    pub stage: BbchStage,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
}

impl From<CreatePhenologyRecordDto> for DomainCreatePhenologyRecordDto {
    fn from(dto: CreatePhenologyRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            observation_date: chrono::DateTime::parse_from_rfc3339(&dto.observation_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            stage: dto.stage,
            notes: dto.notes,
            photo_url: dto.photo_url,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePhenologyRecordDto {
    pub site_id: Option<Uuid>,
    pub observation_date: Option<String>,
    pub stage: Option<agrocore_domain::entities::BbchStage>,
    pub forecast_next_stage_date: Option<String>,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
    pub observer_id: Option<Uuid>,
}

impl From<UpdatePhenologyRecordDto> for DomainUpdatePhenologyRecordDto {
    fn from(dto: UpdatePhenologyRecordDto) -> Self {
        Self {
            site_id: dto.site_id,
            observation_date: dto.observation_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            stage: dto.stage,
            forecast_next_stage_date: dto.forecast_next_stage_date.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .ok()
            }),
            notes: dto.notes,
            photo_url: dto.photo_url,
            observer_id: dto.observer_id,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedFrostWarningResponse {
    pub data: Vec<FrostWarningDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema, validator::Validate)]
pub struct FrostWarningDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub station_id: Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<agrocore_domain::entities::weather::FrostWarning> for FrostWarningDto {
    fn from(f: agrocore_domain::entities::weather::FrostWarning) -> Self {
        Self {
            id: f.id,
            tenant_id: f.tenant_id.into(),
            station_id: f.station_id,
            threshold_temp_c: f.threshold_temp_c,
            is_active: f.is_active,
            notify_email: f.notify_email,
            notify_sms: f.notify_sms,
            last_triggered_at: f.last_triggered_at.map(|t| t.to_rfc3339()),
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateFrostWarningDto {
    pub station_id: Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
}

impl From<CreateFrostWarningDto> for agrocore_domain::entities::weather::CreateFrostWarningDto {
    fn from(dto: CreateFrostWarningDto) -> Self {
        Self {
            station_id: dto.station_id,
            threshold_temp_c: dto.threshold_temp_c,
            is_active: dto.is_active,
            notify_email: dto.notify_email,
            notify_sms: dto.notify_sms,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateFrostWarningDto {
    pub threshold_temp_c: Option<f64>,
    pub is_active: Option<bool>,
    pub notify_email: Option<bool>,
    pub notify_sms: Option<bool>,
}

impl From<UpdateFrostWarningDto> for agrocore_domain::entities::weather::UpdateFrostWarningDto {
    fn from(dto: UpdateFrostWarningDto) -> Self {
        Self {
            threshold_temp_c: dto.threshold_temp_c,
            is_active: dto.is_active,
            notify_email: dto.notify_email,
            notify_sms: dto.notify_sms,
            last_triggered_at: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedGrowingDegreeDayResponse {
    pub data: Vec<GrowingDegreeDayDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GrowingDegreeDayDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub date: String,
    pub base_temp_c: f64,
    pub actual_mean_temp_c: f64,
    pub gdd: f64,
    pub accumulated_gdd: f64,
    pub crop_type: String,
}

impl From<agrocore_domain::entities::weather::GrowingDegreeDay> for GrowingDegreeDayDto {
    fn from(g: agrocore_domain::entities::weather::GrowingDegreeDay) -> Self {
        Self {
            id: g.id,
            tenant_id: g.tenant_id.into(),
            site_id: g.site_id,
            date: g.date.to_rfc3339(),
            base_temp_c: g.base_temp_c,
            actual_mean_temp_c: g.actual_mean_temp_c,
            gdd: g.gdd,
            accumulated_gdd: g.accumulated_gdd,
            crop_type: g.crop_type,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreateGrowingDegreeDayDto {
    pub site_id: Uuid,
    pub date: String,
    pub base_temp_c: f64,
    pub actual_mean_temp_c: f64,
    pub crop_type: String,
}

impl From<CreateGrowingDegreeDayDto>
    for agrocore_domain::entities::weather::CreateGrowingDegreeDayDto
{
    fn from(dto: CreateGrowingDegreeDayDto) -> Self {
        Self {
            site_id: dto.site_id,
            date: chrono::DateTime::parse_from_rfc3339(&dto.date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            base_temp_c: dto.base_temp_c,
            actual_mean_temp_c: dto.actual_mean_temp_c,
            crop_type: dto.crop_type,
        }
    }
}

/// Response for accumulated GDD query
#[derive(Debug, Serialize, ToSchema)]
pub struct AccumulatedGddResponse {
    pub site_id: Uuid,
    pub crop_type: String,
    pub accumulated_gdd: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedPestRiskResponse {
    pub data: Vec<PestRiskDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PestRiskDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub site_id: Uuid,
    pub assessment_date: String,
    pub risk_level: String,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
}

impl From<agrocore_domain::entities::weather::PestRisk> for PestRiskDto {
    fn from(p: agrocore_domain::entities::weather::PestRisk) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id.into(),
            site_id: p.site_id,
            assessment_date: p.assessment_date.to_rfc3339(),
            risk_level: p.risk_level.to_string(),
            pest_type: p.pest_type,
            confidence: p.confidence,
            recommended_action: p.recommended_action,
            model_version: p.model_version,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct CreatePestRiskDto {
    pub site_id: Uuid,
    pub assessment_date: String,
    pub risk_level: String,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
}

impl From<CreatePestRiskDto> for agrocore_domain::entities::weather::CreatePestRiskDto {
    fn from(dto: CreatePestRiskDto) -> Self {
        Self {
            site_id: dto.site_id,
            assessment_date: chrono::DateTime::parse_from_rfc3339(&dto.assessment_date)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            risk_level: dto
                .risk_level
                .parse()
                .unwrap_or(agrocore_domain::entities::weather::RiskLevel::Low),
            pest_type: dto.pest_type,
            confidence: dto.confidence,
            recommended_action: dto.recommended_action,
            model_version: dto.model_version,
        }
    }
}

/// Query parameters for accumulated GDD lookup
#[derive(Debug, Deserialize, ToSchema)]
pub struct GddQuery {
    pub site_id: Uuid,
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
    pub crop_type: String,
}

/// Pagination with a site_id filter
#[derive(Debug, Deserialize, ToSchema)]
pub struct SitePagination {
    pub site_id: Uuid,
    #[serde(flatten)]
    pub pagination: agrocore_shared::Pagination,
}
