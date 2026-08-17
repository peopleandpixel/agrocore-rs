//! Weather Underground provider (paid service).
//!
//! Uses the Weather Underground API (PWS/history).
//! Requires an API key (passed via `api_key` parameter).

use agrocore_domain::WeatherDataProvider;
use agrocore_domain::services::weather::{WeatherFetchResult, WeatherServiceType};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WundergroundResponse {
    current_observation: Option<WundergroundCurrent>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WundergroundCurrent {
    temp_c: Option<f64>,
    relative_humidity: Option<String>,
    pressure_mb: Option<f64>,
    wind_kph: Option<f64>,
    wind_degrees: Option<i32>,
    wind_dir: Option<String>,
    #[serde(rename = "precip_1hr_metric")]
    precip_1hr: Option<f64>,
    solarradiation: Option<f64>,
    #[serde(rename = "UV")]
    uv: Option<String>,
    dewpoint_c: Option<f64>,
    heat_index_c: Option<f64>,
    #[serde(default)]
    leaf_wetness: Option<String>,
    observation_time: Option<String>,
}

/// Provider struct — stateless, holds no data.
pub struct WeatherUndergroundProvider;

impl WeatherDataProvider for WeatherUndergroundProvider {
    fn service_type(&self) -> WeatherServiceType {
        WeatherServiceType::WeatherUnderground
    }

    fn fetch_current(
        &self,
        lat: f64,
        lon: f64,
        api_key: Option<&str>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<WeatherFetchResult, String>> + Send + '_>,
    > {
        let api_key = api_key.map(|s| s.to_string());
        Box::pin(async move {
            let key = api_key.ok_or("Weather Underground API key is required")?;
            let url = format!(
                "https://{}.api.weather.com/integrations/v1/observations.json?geocode={},{},18&product=1H&format=json&apiKey={}",
                key, lat, lon, key
            );
            let client = reqwest::Client::new();
            let resp: WundergroundResponse = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?
                .json()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            let obs = match resp.current_observation {
                Some(o) => o,
                None => return Err("No current observation in response".to_string()),
            };

            let parse_pct = |s: &Option<String>| {
                s.as_ref()
                    .and_then(|v| v.trim_end_matches('%').parse::<f64>().ok())
            };

            // Parse timestamp — wunderground returns like "2024-01-15 12:30:00"
            let timestamp = chrono::Utc::now();

            Ok(WeatherFetchResult {
                temperature_c: obs.temp_c,
                humidity_percent: parse_pct(&obs.relative_humidity),
                precipitation_mm: obs.precip_1hr,
                wind_speed_kmh: obs.wind_kph,
                wind_direction_deg: obs.wind_degrees,
                solar_radiation_wm2: obs.solarradiation,
                pressure_hpa: obs.pressure_mb,
                soil_temperature_c: None,
                soil_moisture_percent: None,
                leaf_wetness: obs
                    .leaf_wetness
                    .as_ref()
                    .and_then(|s| s.parse::<f64>().ok().map(|v| v > 0.0)),
                timestamp,
            })
        })
    }
}
