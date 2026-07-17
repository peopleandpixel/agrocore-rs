use agrocore_domain::entities::tenant::Tenant;
use agrocore_domain::entities::weather::{
    CreateWeatherDataDto, CreateWeatherStationDto, WeatherStationType,
};
use agrocore_infrastructure::Database;
use agrocore_messaging::{Event, GlobalEvent, MessagingClient};
use agrocore_shared::Pagination;
use futures::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoWeatherResponse {
    current: Option<OpenMeteoCurrentWeather>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrentWeather {
    time: chrono::DateTime<chrono::Utc>,
    temperature_2m: f64,
    relative_humidity_2m: f64,
    precipitation: f64,
    wind_speed_10m: f64,
}

pub async fn start(db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe(">").await?;

    info!("Weather Service worker started, listening on all subjects (>)");

    let db_clone = db.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800)); // Every 30 minutes
        loop {
            interval.tick().await;
            if let Err(e) = fetch_weather_for_tenants(&db_clone).await {
                error!("Error fetching weather for tenants: {}", e);
            }
        }
    });

    while let Some(message) = subscriber.next().await {
        let subject = message.subject.clone();

        if subject.as_str() == "system.tenant.created" {
            let event: Event<GlobalEvent> = match serde_json::from_slice(&message.payload) {
                Ok(e) => e,
                Err(e) => {
                    error!("Failed to deserialize tenant created event: {}", e);
                    continue;
                }
            };
            if let GlobalEvent::TenantCreated(tenant) = event.payload {
                info!("Received TenantCreated event for tenant {}", tenant.id);
                let db_clone = db.clone();
                let client = reqwest::Client::new();
                tokio::spawn(async move {
                    if let Err(e) = process_tenant_weather(&db_clone, &tenant, &client).await {
                        error!("Error processing weather for new tenant {}: {}", tenant.id, e);
                    }
                });
            }
            continue;
        }

        if subject.as_str() == "weather.health" {
            if let Some(reply_to) = message.reply {
                let response = serde_json::json!({"status": "ok", "service": "weather"});
                let _ = messaging
                    .publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?)
                    .await;
            }
            continue;
        }

        let event: Event<GlobalEvent> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(e) => {
                error!(
                    "Failed to deserialize weather event on subject {}: {}",
                    subject, e
                );
                continue;
            }
        };

        info!("Received message on subject: {}", subject);

        match event.payload {
            GlobalEvent::HealthCheckRequested => {
                if let Some(reply_to) = message.reply {
                    let response = serde_json::json!({"status": "ok", "service": "weather"});
                    let _ = messaging
                        .publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?)
                        .await;
                }
            }
            _ => {
                // Handle other weather events
            }
        }
    }

    Ok(())
}

async fn fetch_weather_for_tenants(db: &Database) -> anyhow::Result<()> {
    let tenants = db.tenant_repo().find_all(Pagination::default()).await?;
    let client = reqwest::Client::new();

    for tenant in tenants.data {
        if let Err(e) = process_tenant_weather(db, &tenant, &client).await {
            error!("Error processing weather for tenant {}: {}", tenant.id, e);
        }
    }

    Ok(())
}

async fn process_tenant_weather(
    db: &Database,
    tenant: &Tenant,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let stations = db
        .weather_station_repo()
        .find_all(tenant.id, Pagination::default())
        .await?;

    // Check for active stations
    if stations.data.iter().any(|s| s.is_active) {
        return Ok(());
    }

    // No active station, check for company address in config
    let address = match tenant
        .config
        .custom_field_schemas
        .as_ref()
        .and_then(|v| v.get("company_profile"))
        .and_then(|v| v.get("address"))
        .and_then(|v| v.as_str())
    {
        Some(a) if !a.trim().is_empty() => a,
        _ => {
            info!(
                "Tenant {} has no weather stations and no company address.",
                tenant.id
            );
            return Ok(());
        }
    };

    // Geocode address
    let geocoding_url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding::encode(address)
    );
    let geocoding_resp: GeocodingResponse = client.get(geocoding_url).send().await?.json().await?;

    let location = match geocoding_resp.results.and_then(|r| r.into_iter().next()) {
        Some(l) => l,
        None => {
            warn!("Could not geocode address '{}' for tenant {}", address, tenant.id);
            return Ok(());
        }
    };

    // Find or create virtual station
    let virtual_station_label = "Company Address Weather";
    let station = if let Some(s) = stations
        .data
        .iter()
        .find(|s| s.label == virtual_station_label && s.station_type == WeatherStationType::Virtual)
    {
        s.clone()
    } else {
        info!("Creating virtual weather station for tenant {}", tenant.id);
        db.weather_station_repo()
            .create(
                tenant.id,
                CreateWeatherStationDto {
                    label: virtual_station_label.to_string(),
                    station_type: WeatherStationType::Virtual,
                    location: None, // We have the coordinates but the DTO/Repo might need update to support it properly if it's PostGIS. For now, we use the coordinates in the fetch.
                    manufacturer: Some("Open-Meteo".to_string()),
                    model: Some("Virtual Station".to_string()),
                    serial_number: None,
                    api_key_config: None,
                },
                Uuid::nil(),
            )
            .await?
    };

    // Fetch current weather
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,precipitation,wind_speed_10m",
        location.latitude, location.longitude
    );
    let weather_resp: OpenMeteoWeatherResponse = client.get(weather_url).send().await?.json().await?;

    if let Some(current) = weather_resp.current {
        db.weather_data_repo()
            .create(
                tenant.id,
                CreateWeatherDataDto {
                    station_id: station.id,
                    timestamp: current.time,
                    temperature_c: Some(current.temperature_2m),
                    humidity_percent: Some(current.relative_humidity_2m),
                    precipitation_mm: Some(current.precipitation),
                    wind_speed_kmh: Some(current.wind_speed_10m),
                    wind_direction_deg: None,
                    solar_radiation_wm2: None,
                    pressure_hpa: None,
                },
            )
            .await?;
        info!("Stored weather data for tenant {}", tenant.id);
    }

    Ok(())
}
