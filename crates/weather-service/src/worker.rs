use crate::providers::{OpenMeteoProvider, OpenWeatherProvider, WeatherUndergroundProvider};
use agrocore_domain::entities::weather::{
    CreateWeatherDataDto, CreateWeatherStationDto, WeatherStationType,
};
use agrocore_domain::services::weather::{
    WeatherDataProvider, WeatherFetchResult, WeatherServiceType,
};
use agrocore_domain::{TenantId, entities::tenant::Tenant};
use agrocore_infrastructure::Database;
use agrocore_logging::{error, info, warn};
use agrocore_messaging::{
    Event, GlobalEvent, IrrigationCommand, IrrigationCommandEvent, MessagingClient,
    NATS_SUBJECT_COMMANDS_IRRIGATION, NATS_SUBJECT_TELEMETRY_SOIL, NATS_SUBJECT_TELEMETRY_WEATHER,
    SoilMoistureAlertEvent,
};
use agrocore_shared::Pagination;
use futures::StreamExt;
use serde::Deserialize;
use std::time::Duration;
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

impl Clone for ProviderRegistry {
    fn clone(&self) -> Self {
        Self::default()
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
                    .and_then(|v| Some(v.to_string()))
                    .and_then(|s| s.parse::<WeatherServiceType>().ok())?;
                let api_key = parsed
                    .get("api_key")
                    .and_then(|v| Some(v.to_string()))
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
    let mut subscriber = messaging.subscribe("\">".to_string()).await?;
    // Also subscribe to specific telemetry subjects forwarded by the MQTT bridge
    let weather_telemetry_sub = messaging
        .subscribe(NATS_SUBJECT_TELEMETRY_WEATHER.to_string())
        .await?;
    let soil_telemetry_sub = messaging
        .subscribe(NATS_SUBJECT_TELEMETRY_SOIL.to_string())
        .await?;

    let registry = ProviderRegistry::default();

    info!(
        "Weather Service worker started, listening on all subjects (>), \\
         providers registered: OpenMeteo (free), OpenWeather (paid), Weather Underground (paid). \\
         Also subscribed to MQTT-bridged telemetry: {}, {}",
        NATS_SUBJECT_TELEMETRY_WEATHER, NATS_SUBJECT_TELEMETRY_SOIL
    );

    let db_clone = db.clone();
    let registry_clone = registry.clone();
    let messaging_clone = messaging.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1800)); // Every 30 minutes
        loop {
            interval.tick().await;
            if let Err(e) =
                fetch_weather_for_tenants(&db_clone, &registry_clone, &messaging_clone).await
            {
                error!("Error fetching weather for tenants: {}", e);
            }
        }
    });

    tokio::pin!(weather_telemetry_sub, soil_telemetry_sub);

    loop {
        tokio::select! {
            // Handle general NATS messages (health, tenant created, etc.)
            message = subscriber.next() => {
                let Some(message) = message else { break; };
                let subject = message.subject.clone();

                if subject.to_string() == "system.tenant.created" {
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
                        let messaging_clone = messaging.clone();
                        let client = reqwest::Client::new();
                        tokio::spawn(async move {
                            // Timeout 30s für Tenant-Wetter-Verarbeitung (OPT-010)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                process_tenant_weather(
                                    &db_clone, &tenant, &registry_clone, &client, &messaging_clone,
                                ),
                            ).await {
                                Ok(Ok(())) => info!("Tenant weather processed: {}", tenant.id),
                                Ok(Err(e)) => error!("Error processing weather: {}", e),
                                Err(_) => error!(
                                    "Timeout: Weather processing exceeded 30s for tenant {}",
                                    tenant.id
                                ),
                            }
                        });
                    }
                    continue;
                }

                if subject.to_string() == "weather.health" {
                    if let Some(reply_to) = message.reply {
                        let response = serde_json::json!({"status": "ok", "service": "weather"});
                        let _ = messaging
                            .publish_raw(reply_to.to_string(), serde_json::to_vec(&response)?)
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
                                .publish_raw(reply_to.to_string(), serde_json::to_vec(&response)?)
                                .await;
                        }
                    }
                    GlobalEvent::WeatherDataCollected(data) => {
                        info!(
                            "Received WeatherDataCollected event for station {} (tenant {})",
                            data.station_id, data.tenant_id
                        );
                    }
                    GlobalEvent::SoilMoistureAlert(_) => {
                        info!("Received soil moisture alert event");
                    }
                    GlobalEvent::IrrigationTriggered(_) => {
                        info!("Received irrigation command event");
                    }
                    _ => {}
                }
            }

            // Handle weather telemetry forwarded from MQTT by the bridge
            weather_msg = weather_telemetry_sub.next() => {
                let Some(message) = weather_msg else { break; };
                if let Ok(event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload)
                    && let GlobalEvent::WeatherDataCollected(data) = event.payload
                {
                    info!(
                        "Received IoT weather telemetry from NATS (bridge from MQTT) for station {}",
                        data.station_id
                    );
                }
            }

            // Handle soil moisture telemetry forwarded from MQTT by the bridge
            soil_msg = soil_telemetry_sub.next() => {
                let Some(message) = soil_msg else { break; };
                if let Ok(event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload) {
                    match event.payload {
                        GlobalEvent::WeatherDataCollected(data) => {
                            if let Some(moisture) = data.soil_moisture_percent {
                                process_soil_moisture_alerts(&db, data.tenant_id, data.station_id, moisture, &messaging).await;
                            }
                        }
                        GlobalEvent::SoilMoistureAlert(alert) => {
                            info!(
                                "Received soil alert via MQTT bridge: {}% below {}%",
                                alert.moisture_percent, alert.threshold_percent
                            );
                        }
                        _ => {}
                    }
                }
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
    messaging: &MessagingClient,
) -> anyhow::Result<()> {
    let tenants = db.tenant_repo().find_all(Pagination::default()).await?;
    let client = reqwest::Client::new();

    for tenant in tenants.data {
        if let Err(e) = process_tenant_weather(db, &tenant, registry, &client, messaging).await {
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
    messaging: &MessagingClient,
) -> anyhow::Result<()> {
    let stations = db
        .weather_station_repo()
        .find_all(TenantId(tenant.id), Pagination::default())
        .await?;

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
            (0.0, 0.0)
        };

        if station.station_type == WeatherStationType::ExternalApi {
            match provider.fetch_current(lat, lon, api_key.as_deref()).await {
                Ok(result) => {
                    store_weather_data(db, TenantId(tenant.id), station.id, result, messaging)
                        .await?;
                }
                Err(e) => {
                    warn!("Provider fetch failed for station {}: {}", station.id, e);
                }
            }
            continue;
        }

        let address = match tenant
            .config
            .custom_field_schemas
            .as_ref()
            .and_then(|v| v.get("company_profile"))
            .and_then(|v| v.get("address"))
            .and_then(|v| Some(v.to_string()))
        {
            Some(a) if !a.trim().is_empty() => a,
            _ => continue,
        };

        let geocoding_url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            urlencoding::encode(&address)
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

        match provider
            .fetch_current(location.latitude, location.longitude, None)
            .await
        {
            Ok(result) => {
                store_weather_data(db, TenantId(tenant.id), station.id, result, messaging).await?;
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

/// Store weather fetch results into the database via the weather_data repository,
/// then publish the event via NATS (the MqttBridge forwards it to MQTT for
/// Home Assistant and other MQTT consumers).
async fn store_weather_data(
    db: &Database,
    tid: TenantId,
    station_id: Uuid,
    result: WeatherFetchResult,
    messaging: &MessagingClient,
) -> anyhow::Result<()> {
    let weather_data = db
        .weather_data_repo()
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

    // Publish the WeatherDataCollected event via NATS.
    // The MqttBridge (in agrocore-messaging) forwards this to MQTT topics
    // for Home Assistant auto-discovery and other MQTT consumers.
    let event = Event::new(
        format!("weather.station.{}", station_id),
        GlobalEvent::WeatherDataCollected(weather_data.clone()),
    );
    let payload = serde_json::to_vec(&event)?;
    let _ = messaging
        .publish_raw(NATS_SUBJECT_TELEMETRY_WEATHER.to_string(), payload)
        .await;

    // If soil moisture data is present, check thresholds and publish alerts
    if let Some(moisture) = result.soil_moisture_percent {
        process_soil_moisture_alerts(db, tid, station_id, moisture, messaging).await;
    }

    Ok(())
}

/// Check soil moisture configurations and publish alerts + irrigation commands if thresholds are breached.
async fn process_soil_moisture_alerts(
    db: &Database,
    tid: TenantId,
    station_id: Uuid,
    moisture_percent: f64,
    messaging: &MessagingClient,
) {
    let configs = match db
        .soil_moisture_config_repo()
        .find_by_station(tid, station_id)
        .await
    {
        Ok(configs) => configs,
        Err(e) => {
            warn!(
                "Failed to load soil moisture configs for station {}: {}",
                station_id, e
            );
            return;
        }
    };

    for config in configs {
        if !config.is_active {
            continue;
        }

        if moisture_percent < config.moisture_threshold_percent {
            let alert = SoilMoistureAlertEvent {
                device_id: format!("station:{}", station_id),
                tenant_id: tid.0,
                site_id: config.site_id,
                station_id: Some(station_id),
                moisture_percent,
                threshold_percent: config.moisture_threshold_percent,
                timestamp: chrono::Utc::now(),
                recommended_action: format!(
                    "Soil moisture {:.1}% below threshold {:.1}%. \\
                     Irrigation recommended for {} minutes.",
                    moisture_percent,
                    config.moisture_threshold_percent,
                    config.irrigation_duration_minutes
                ),
            };

            // Publish alert via NATS (MqttBridge forwards to MQTT)
            let event = Event::new(
                format!("soil.moisture.alert.station.{}", station_id),
                GlobalEvent::SoilMoistureAlert(alert.clone()),
            );
            let payload = serde_json::to_vec(&event).unwrap_or_default();
            let _ = messaging
                .publish_raw(NATS_SUBJECT_TELEMETRY_SOIL.to_string(), payload)
                .await;

            // Publish irrigation command via NATS
            let irrigation_cmd = IrrigationCommandEvent {
                device_id: format!("station:{}", station_id),
                tenant_id: tid.0,
                site_id: config.site_id,
                station_id: Some(station_id),
                moisture_percent,
                threshold_percent: config.moisture_threshold_percent,
                command: IrrigationCommand::SetDuration {
                    minutes: config.irrigation_duration_minutes as u32,
                },
                triggered_at: chrono::Utc::now(),
            };

            let cmd_event = Event::new(
                format!("irrigation.command.station.{}", station_id),
                GlobalEvent::IrrigationTriggered(irrigation_cmd.clone()),
            );
            let cmd_payload = serde_json::to_vec(&cmd_event).unwrap_or_default();
            let _ = messaging
                .publish_raw(NATS_SUBJECT_COMMANDS_IRRIGATION.to_string(), cmd_payload)
                .await;

            info!(
                "Soil moisture alert: {:.1}% < threshold {:.1}% for station {} — \\
                 irrigation commanded for {} minutes",
                moisture_percent,
                config.moisture_threshold_percent,
                station_id,
                config.irrigation_duration_minutes
            );
        }
    }
}

/// Start the weather service worker (for NATS message handling only, no periodic updates)
pub async fn start_nats_listener(db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("\">".to_string()).await?;
    // Also subscribe to specific telemetry subjects forwarded by the MQTT bridge
    let weather_telemetry_sub = messaging
        .subscribe(NATS_SUBJECT_TELEMETRY_WEATHER.to_string())
        .await?;
    let soil_telemetry_sub = messaging
        .subscribe(NATS_SUBJECT_TELEMETRY_SOIL.to_string())
        .await?;

    let registry = ProviderRegistry::default();

    info!(
        "Weather Service NATS listener started, listening on all subjects (>), \\
         providers registered: OpenMeteo (free), OpenWeather (paid), Weather Underground (paid). \\
         Also subscribed to MQTT-bridged telemetry: {}, {}",
        NATS_SUBJECT_TELEMETRY_WEATHER, NATS_SUBJECT_TELEMETRY_SOIL
    );

    tokio::pin!(weather_telemetry_sub, soil_telemetry_sub);

    loop {
        tokio::select! {
            // Handle general NATS messages (health, tenant created, etc.)
            message = subscriber.next() => {
                let Some(message) = message else { break; };
                let subject = message.subject.clone();

                if subject.to_string() == "system.tenant.created" {
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
                        let messaging_clone = messaging.clone();
                        let client = reqwest::Client::new();
                        tokio::spawn(async move {
                            // Timeout 30s für Tenant-Wetter-Verarbeitung (OPT-010)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(30),
                                process_tenant_weather(
                                    &db_clone, &tenant, &registry_clone, &client, &messaging_clone,
                                ),
                            ).await {
                                Ok(Ok(())) => info!("Tenant weather processed: {}", tenant.id),
                                Ok(Err(e)) => error!("Error processing weather: {}", e),
                                Err(_) => error!(
                                    "Timeout: Weather processing exceeded 30s for tenant {}",
                                    tenant.id
                                ),
                            }
                        });
                    }
                    continue;
                }

                if subject.to_string() == "weather.health" {
                    if let Some(reply_to) = message.reply {
                        let response = serde_json::json!({"status": "ok", "service": "weather"});
                        let _ = messaging
                            .publish_raw(reply_to.to_string(), serde_json::to_vec(&response)?)
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
                                .publish_raw(reply_to.to_string(), serde_json::to_vec(&response)?)
                                .await;
                        }
                    }
                    GlobalEvent::WeatherDataCollected(data) => {
                        info!(
                            "Received WeatherDataCollected event for station {} (tenant {})",
                            data.station_id, data.tenant_id
                        );
                    }
                    GlobalEvent::SoilMoistureAlert(_) => {
                        info!("Received soil moisture alert event");
                    }
                    GlobalEvent::IrrigationTriggered(_) => {
                        info!("Received irrigation command event");
                    }
                    _ => {}
                }
            }

            // Handle weather telemetry forwarded from MQTT by the bridge
            weather_msg = weather_telemetry_sub.next() => {
                let Some(message) = weather_msg else { break; };
                if let Ok(event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload)
                    && let GlobalEvent::WeatherDataCollected(data) = event.payload
                {
                    info!(
                        "Received IoT weather telemetry from NATS (bridge from MQTT) for station {}",
                        data.station_id
                    );
                }
            }

            // Handle soil moisture telemetry forwarded from MQTT by the bridge
            soil_msg = soil_telemetry_sub.next() => {
                let Some(message) = soil_msg else { break; };
                if let Ok(event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload) {
                    match event.payload {
                        GlobalEvent::WeatherDataCollected(data) => {
                            if let Some(moisture) = data.soil_moisture_percent {
                                process_soil_moisture_alerts(&db, data.tenant_id, data.station_id, moisture, &messaging).await;
                            }
                        }
                        GlobalEvent::SoilMoistureAlert(alert) => {
                            info!(
                                "Received soil alert via MQTT bridge: {}% below {}%",
                                alert.moisture_percent, alert.threshold_percent
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Run weather update for all tenants (called by scheduler)
pub async fn run_weather_update(db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let registry = ProviderRegistry::default();

    let tenants = db.tenant_repo().find_all(Pagination::default()).await?;
    let client = reqwest::Client::new();

    for tenant in tenants.data {
        if let Err(e) = process_tenant_weather(&db, &tenant, &registry, &client, &messaging).await {
            error!("Error processing weather for tenant {}: {}", tenant.id, e);
        }
    }

    Ok(())
}
