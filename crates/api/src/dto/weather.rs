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
