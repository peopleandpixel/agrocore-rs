//! OpenWeather provider (free tier + paid plans).
//!
//! Uses the OpenWeather One Call API 3.0.
//! Requires an API key (passed via `api_key` parameter).

use agrocore_domain::WeatherDataProvider;
use agrocore_domain::services::weather::{WeatherFetchResult, WeatherServiceType};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherResponse {
    current: Option<OpenWeatherCurrent>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherCurrent {
    #[serde(default)]
    temp: Option<f64>,
    #[serde(default)]
    humidity: Option<f64>,
    #[serde(default)]
    pressure: Option<f64>,
    wind_speed: Option<f64>,
    wind_deg: Option<f64>,
    #[serde(default)]
    uvi: Option<f64>,
    clouds: Option<OpenWeatherClouds>,
    #[serde(default)]
    rain: Option<OpenWeatherRain>,
    #[serde(default)]
    snow: Option<OpenWeatherSnow>,
    weather: Option<Vec<OpenWeatherWeather>>,
    // soil data is not directly in One Call but in soil API endpoint
    // for simplicity we leave these as None
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherClouds {
    all: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherRain {
    #[serde(default)]
    #[serde(rename = "1h")]
    one_hour: Option<f64>,
    #[serde(default)]
    #[serde(rename = "3h")]
    three_hour: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherSnow {
    #[serde(default)]
    #[serde(rename = "1h")]
    one_hour: Option<f64>,
    #[serde(default)]
    #[serde(rename = "3h")]
    three_hour: Option<f64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenWeatherWeather {
    id: u32,
    main: String,
    description: String,
}

/// Provider struct — stateless, holds no data.
pub struct OpenWeatherProvider;

impl WeatherDataProvider for OpenWeatherProvider {
    fn service_type(&self) -> WeatherServiceType {
        WeatherServiceType::OpenWeather
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
            let key = api_key.ok_or("OpenWeather API key is required")?;
            let url = format!(
                "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=minutely,daily,alerts&appid={}&units=metric",
                lat, lon, key
            );
            let client = reqwest::Client::new();
            let resp: OpenWeatherResponse = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?
                .json()
                .await
                .map_err(|e| format!("JSON parse error: {}", e))?;

            let current = resp.current.ok_or("No current weather data in response")?;

            // Convert wind speed from m/s to km/h
            let wind_speed_kmh = current.wind_speed.map(|v| v * 3.6);

            Ok(WeatherFetchResult {
                temperature_c: current.temp,
                humidity_percent: current.humidity,
                precipitation_mm: current
                    .rain
                    .as_ref()
                    .and_then(|r| r.one_hour)
                    .or_else(|| current.snow.as_ref().and_then(|s| s.one_hour)),
                wind_speed_kmh,
                wind_direction_deg: current.wind_deg.map(|v| v as i32),
                solar_radiation_wm2: None, // Not in standard One Call response
                pressure_hpa: current.pressure,
                soil_temperature_c: None, // Requires separate soil API call
                soil_moisture_percent: None,
                leaf_wetness: None,
                timestamp: chrono::Utc::now(),
            })
        })
    }
}
