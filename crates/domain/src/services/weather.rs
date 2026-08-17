//! Weather service provider abstraction.
//!
//! Provides a trait-based interface for fetching weather data from multiple
//! weather service providers (Open-Meteo, OpenWeather, Weather Underground, etc.).

use crate::entities::weather::CreateWeatherDataDto;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

/// Enum of supported weather service providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherServiceType {
    /// Free, no API key required.
    OpenMeteo,
    /// Free tier available with API key.
    OpenWeather,
    /// Paid service, requires API key.
    WeatherUnderground,
}

impl std::fmt::Display for WeatherServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenMeteo => write!(f, "openmeteo"),
            Self::OpenWeather => write!(f, "openweather"),
            Self::WeatherUnderground => write!(f, "wunderground"),
        }
    }
}

impl std::str::FromStr for WeatherServiceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openmeteo" | "open-meteo" | "open_meteo" => Ok(Self::OpenMeteo),
            "openweather" => Ok(Self::OpenWeather),
            "wunderground" | "weatherunderground" => Ok(Self::WeatherUnderground),
            _ => Err(format!("Unknown weather service type: {}", s)),
        }
    }
}

/// Result of a weather service fetch — raw response fields
/// that can be mapped into a `WeatherData` entity.
#[derive(Debug, Clone)]
pub struct WeatherFetchResult {
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<WeatherFetchResult> for CreateWeatherDataDto {
    fn from(r: WeatherFetchResult) -> Self {
        CreateWeatherDataDto {
            station_id: Uuid::nil(),
            timestamp: r.timestamp,
            temperature_c: r.temperature_c,
            humidity_percent: r.humidity_percent,
            precipitation_mm: r.precipitation_mm,
            wind_speed_kmh: r.wind_speed_kmh,
            wind_direction_deg: r.wind_direction_deg,
            solar_radiation_wm2: r.solar_radiation_wm2,
            pressure_hpa: r.pressure_hpa,
            soil_temperature_c: r.soil_temperature_c,
            soil_moisture_percent: r.soil_moisture_percent,
            leaf_wetness: r.leaf_wetness,
        }
    }
}

/// A trait for weather service providers.
///
/// Each implementation knows how to fetch current weather data
/// for a given lat/long coordinate pair from its respective service.
pub trait WeatherDataProvider: Send + Sync {
    /// The service type this provider implements.
    fn service_type(&self) -> WeatherServiceType;

    /// Fetch current weather data for the given coordinates.
    fn fetch_current(
        &self,
        lat: f64,
        lon: f64,
        api_key: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<WeatherFetchResult, String>> + Send + '_>>;
}
