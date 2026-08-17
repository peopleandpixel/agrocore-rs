//! Open-Meteo provider (free, no API key required).
//!
//! Uses the Open-Meteo Forecast API with current weather variables.

use agrocore_domain::WeatherDataProvider;
use agrocore_domain::services::weather::{WeatherFetchResult, WeatherServiceType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenMeteoCurrent {
    time: chrono::DateTime<chrono::Utc>,
    temperature_2m: Option<f64>,
    relative_humidity_2m: Option<f64>,
    precipitation: Option<f64>,
    wind_speed_10m: Option<f64>,
    wind_direction_10m: Option<f64>,
    pressure_msl: Option<f64>,
    surface_pressure: Option<f64>,
    shortwave_radiation: Option<f64>,
    direct_normal_illuminance: Option<f64>,
    diffuse_radiation: Option<f64>,
    soil_temperature_0cm: Option<f64>,
    soil_moisture_0_to_1cm: Option<f64>,
}

/// Provider struct — stateless, holds no data.
pub struct OpenMeteoProvider;

impl WeatherDataProvider for OpenMeteoProvider {
    fn service_type(&self) -> WeatherServiceType {
        WeatherServiceType::OpenMeteo
    }

    fn fetch_current(
        &self,
        lat: f64,
        lon: f64,
        _api_key: Option<&str>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<WeatherFetchResult, String>> + Send + '_>,
    > {
        Box::pin(async move {
            let url = format!(
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m,wind_direction_10m,pressure_msl,shortwave_radiation,soil_temperature_0cm,soil_moisture_0_to_1cm",
                lat, lon
            );
            let client = reqwest::Client::new();
            let resp: OpenMeteoResponse = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?
                .json()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            let current = match resp.current {
                Some(c) => c,
                None => return Err("No current weather data in response".to_string()),
            };

            Ok(WeatherFetchResult {
                temperature_c: current.temperature_2m,
                humidity_percent: current.relative_humidity_2m,
                precipitation_mm: current.precipitation,
                wind_speed_kmh: current.wind_speed_10m,
                wind_direction_deg: current.wind_direction_10m.map(|v| v as i32),
                solar_radiation_wm2: current.shortwave_radiation,
                pressure_hpa: current.pressure_msl.or(current.surface_pressure),
                soil_temperature_c: current.soil_temperature_0cm,
                soil_moisture_percent: current.soil_moisture_0_to_1cm,
                leaf_wetness: None,
                timestamp: current.time,
            })
        })
    }
}
