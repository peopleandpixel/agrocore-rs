use crate::providers::{OpenMeteoProvider, OpenWeatherProvider, WeatherUndergroundProvider};
use agrocore_domain::entities::weather::{
    CreateWeatherDataDto, CreateWeatherStationDto, WeatherStationType,
};
use agrocore_domain::services::weather::{
    WeatherDataProvider, WeatherFetchResult, WeatherServiceType,
};
use agrocore_domain::{TenantId, entities::tenant::Tenant};
use agrocore_infrastructure::Database;
use agrocore_messaging::{Event, GlobalEvent, MessagingClient};
use agrocore_shared::Pagination;
use futures::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// A registry of all available weather service providers.
pub struct ProviderRegistry {
    providers: Vec<(WeatherServiceType, Box<dyn WeatherDataProvider>)>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self {
            providers: vec![
                (WeatherServiceType::OpenMeteo, Box::new(OpenMeteoProvider)),
                (
                    WeatherServiceType::OpenWeather,
                    Box::new(OpenWeatherProvider),
                ),
                (
                    WeatherServiceType::WeatherUnderground,
                    Box::new(WeatherUndergroundProvider),
                ),
            ],
        }
    }
}

impl ProviderRegistry {
    /// Resolve the provider for a given station based on its station_type and api_key_config.
    pub fn resolve_for_station(
        &self,
        station_type: &WeatherStationType,
        api_key_config: &Option<String>,
    ) -> Option<(WeatherServiceType, Option<String>)> {
        match station_type {
            WeatherStationType::Virtual => Some((WeatherServiceType::OpenMeteo, None)),
            WeatherStationType::ExternalApi => {
                let config = api_key_config.as_ref()?;
                let parsed: serde_json::Value = serde_json::from_str(config).ok()?;
                let provider_type = parsed
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<WeatherServiceType>().ok())?;
                let api_key = parsed
                    .get("api_key")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Some((provider_type, api_key))
            }
            WeatherStationType::Iot | WeatherStationType::Manual => None,
        }
    }

    /// Get the provider instance for a service type.
    pub fn get(&self, service_type: WeatherServiceType) -> Option<&dyn WeatherDataProvider> {
        self.providers
            .iter()
            .find(|(t, _)| *t == service_type)
            .map(|(_, p)| p.as_ref())
    }
}

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

pub async fn start(db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe(">").await?;

    let registry = ProviderRegistry::default();

    info!(
        "Weather Service worker started, listening on all subjects (>), providers registered: OpenMeteo, OpenWeather, WeatherUnderground"
    );

    let db_clone = db.clone();
    let registry_clone = registry.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800)); // Every 30 minutes
        loop {
            interval.tick().await;
            if let Err(e) = fetch_weather_for_tenants(&db_clone, &registry_clone).await {
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
                let registry_clone = registry.clone();
                let client = reqwest::Client::new();
                tokio::spawn(async move {
                    if let Err(e) =
                        process_tenant_weather(&db_clone, &tenant, &registry_clone, &client).await
                    {
                        error!(
                            "Error processing weather for new tenant {}: {}",
                            tenant.id, e
                        );
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

/// Register an external API weather station (e.g. OpenWeather, Weather Underground).
pub async fn register_external_provider_station(
    db: &Database,
    tid: TenantId,
    provider_type: WeatherServiceType,
    api_key: &str,
    lat: f64,
    lon: f64,
) -> anyhow::Result<()> {
    let api_key_config = serde_json::json!({
        "provider": provider_type.to_string(),
        "api_key": api_key,
        "lat": lat,
        "lon": lon,
    })
    .to_string();

    db.weather_station_repo()
        .create(
            tid,
            CreateWeatherStationDto {
                label: format!("{} Station", provider_type),
                station_type: WeatherStationType::ExternalApi,
                location: None,
                manufacturer: Some(provider_type.to_string()),
                model: Some("API".to_string()),
                serial_number: None,
                api_key_config: Some(api_key_config),
            },
            Uuid::nil(),
        )
        .await?;

    info!(
        "Registered external weather station for provider {}",
        provider_type
    );
    Ok(())
}

async fn fetch_weather_for_tenants(
    db: &Database,
    registry: &ProviderRegistry,
) -> anyhow::Result<()> {
    let tenants = db.tenant_repo().find_all(Pagination::default()).await?;
    let client = reqwest::Client::new();

    for tenant in tenants.data {
        if let Err(e) = process_tenant_weather(db, &tenant, registry, &client).await {
            error!("Error processing weather for tenant {}: {}", tenant.id, e);
        }
    }

    Ok(())
}

async fn process_tenant_weather(
    db: &Database,
    tenant: &Tenant,
    registry: &ProviderRegistry,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let stations = db
        .weather_station_repo()
        .find_all(TenantId(tenant.id), Pagination::default())
        .await?;

    // Process external API stations via provider trait
    for station in &stations.data {
        if !station.is_active {
            continue;
        }

        let Some((service_type, api_key)) =
            registry.resolve_for_station(&station.station_type, &station.api_key_config)
        else {
            continue;
        };

        let provider = match registry.get(service_type) {
            Some(p) => p,
            None => continue,
        };

        // For ExternalApi stations, extract lat/lon from api_key_config
        let (lat, lon) = if station.station_type == WeatherStationType::ExternalApi {
            let config = station
                .api_key_config
                .as_ref()
                .and_then(|c| serde_json::from_str::<serde_json::Value>(c).ok());
            match config {
                Some(v) => {
                    let lat = v.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let lon = v.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    (lat, lon)
                }
                None => {
                    warn!("Station {} has no valid api_key_config", station.id);
                    continue;
                }
            }
        } else {
            // Virtual stations — geocode from tenant address
            (0.0, 0.0) // will be handled below
        };

        if station.station_type == WeatherStationType::ExternalApi {
            match provider.fetch_current(lat, lon, api_key.as_deref()).await {
                Ok(result) => {
                    store_weather_data(db, TenantId(tenant.id), station.id, result).await?;
                }
                Err(e) => {
                    warn!("Provider fetch failed for station {}: {}", station.id, e);
                }
            }
            continue;
        }

        // Virtual stations — geocode address and use provider
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
                // Check if we already have data from external providers — skip geocoding
                continue;
            }
        };

        let geocoding_url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            urlencoding::encode(address)
        );
        let geocoding_resp: GeocodingResponse =
            client.get(&geocoding_url).send().await?.json().await?;

        let location = match geocoding_resp.results.and_then(|r| r.into_iter().next()) {
            Some(l) => l,
            None => {
                warn!(
                    "Could not geocode address '{}' for tenant {}",
                    address, tenant.id
                );
                continue;
            }
        };

        // Fetch current weather via the provider
        match provider
            .fetch_current(location.latitude, location.longitude, None)
            .await
        {
            Ok(result) => {
                store_weather_data(db, TenantId(tenant.id), station.id, result).await?;
                info!("Stored weather data for tenant {}", tenant.id);
            }
            Err(e) => {
                warn!(
                    "Weather fetch failed for virtual station {} (tenant {}): {}",
                    station.id, tenant.id, e
                );
            }
        }
    }

    Ok(())
}

/// Store weather fetch results into the database via the weather_data repository.
async fn store_weather_data(
    db: &Database,
    tid: TenantId,
    station_id: Uuid,
    result: WeatherFetchResult,
) -> anyhow::Result<()> {
    db.weather_data_repo()
        .create(
            tid,
            CreateWeatherDataDto {
                station_id,
                timestamp: result.timestamp,
                temperature_c: result.temperature_c,
                humidity_percent: result.humidity_percent,
                precipitation_mm: result.precipitation_mm,
                wind_speed_kmh: result.wind_speed_kmh,
                wind_direction_deg: result.wind_direction_deg,
                solar_radiation_wm2: result.solar_radiation_wm2,
                pressure_hpa: result.pressure_hpa,
                soil_temperature_c: result.soil_temperature_c,
                soil_moisture_percent: result.soil_moisture_percent,
                leaf_wetness: result.leaf_wetness,
            },
        )
        .await?;
    Ok(())
}

// Implement Clone for ProviderRegistry since it only holds a Vec of trait objects
// We need a manual Clone because Box<dyn Trait> doesn't implement Clone.
// Since all providers are stateless, we can just recreate the registry.
impl Clone for ProviderRegistry {
    fn clone(&self) -> Self {
        Self {
            providers: vec![
                (WeatherServiceType::OpenMeteo, Box::new(OpenMeteoProvider)),
                (
                    WeatherServiceType::OpenWeather,
                    Box::new(OpenWeatherProvider),
                ),
                (
                    WeatherServiceType::WeatherUnderground,
                    Box::new(WeatherUndergroundProvider),
                ),
            ],
        }
    }
}
