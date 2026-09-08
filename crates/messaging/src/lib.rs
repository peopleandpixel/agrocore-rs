use agrocore_domain::entities::compliance::AuditLog;
use agrocore_domain::entities::order::Order;
use agrocore_domain::entities::site::{GeoPoint, Site};
use agrocore_domain::entities::spatial::SpatialObjectType;
use agrocore_domain::entities::user::User;
use agrocore_domain::entities::weather::{PhenologyRecord, WeatherData, WeatherStation};
use agrocore_logging::{debug, error, info, warn};
use async_nats::Client;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use failsafe::Config;
use futures_util::StreamExt;
use rumqttc::{
    AsyncClient, Event as MqttEvent, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::RwLock as AsyncRwLock;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// Publisher trait for sending messages
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    async fn publish_raw(&self, subject: String, payload: Vec<u8>) -> anyhow::Result<()>;
    async fn publish<T: serde::Serialize + Send + Sync>(
        &self,
        subject: String,
        payload: &T,
    ) -> anyhow::Result<()>;
    async fn request<
        T: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send,
    >(
        &self,
        subject: &str,
        payload: &T,
    ) -> anyhow::Result<R>;
    async fn request_with_headers<
        T: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send,
    >(
        &self,
        subject: &str,
        headers: async_nats::HeaderMap,
        payload: &T,
    ) -> anyhow::Result<R>;
}

// Subscriber trait for receiving messages
#[async_trait::async_trait]
pub trait Subscriber: Send + Sync {
    async fn subscribe(&self, subject: String) -> anyhow::Result<Box<dyn MessageStream>>;
}

// Message stream trait
#[async_trait::async_trait]
pub trait MessageStream: Send + Sync {
    async fn next(&mut self) -> Option<async_nats::Message>;
}

// Precomputed static NATS subjects to avoid repeated string allocations
pub const NATS_SUBJECT_EVENTS: &str = "events.>";
pub const NATS_SUBJECT_TELEMETRY: &str = "telemetry.>";
pub const NATS_SUBJECT_DEVICE_STATUS: &str = "device.status.>";
pub const NATS_SUBJECT_COMMANDS_BRIDGE: &str = "commands.bridge";
pub const NATS_SUBJECT_COMMANDS_BRIDGE_BROADCAST: &str = "commands.bridge.broadcast";

pub mod bridge;

pub use bridge::{BridgeConfig, BridgeRoute, BridgeStats, MqttBridge, MqttBridgeBuilder};

/// Simple NATS messaging client wrapper
#[derive(Clone)]
pub struct MessagingClient {
    pub client: async_nats::Client,
}

impl MessagingClient {
    pub async fn connect(nats_url: &str) -> anyhow::Result<Self> {
        let client = async_nats::connect(nats_url).await?;
        Ok(Self { client })
    }

    pub async fn subscribe(&self, subject: String) -> anyhow::Result<async_nats::Subscriber> {
        self.client.subscribe(subject).await.map_err(Into::into)
    }

    pub async fn publish_raw(&self, subject: String, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }

    pub async fn publish<T: serde::Serialize>(
        &self,
        subject: String,
        payload: &T,
    ) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(payload)?;
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    /// Send a request and wait for a reply (request-reply pattern)
    pub async fn request<T: serde::Serialize, R: for<'de> serde::Deserialize<'de>>(
        &self,
        subject: &str,
        payload: &T,
    ) -> anyhow::Result<R> {
        let payload = serde_json::to_vec(payload)?;

        // Send request and wait for response
        let response = self
            .client
            .request(subject.to_string(), payload.into())
            .await?;

        // Deserialize response
        serde_json::from_slice(&response.payload).map_err(Into::into)
    }

    /// Send a request with headers and wait for a reply
    pub async fn request_with_headers<T: serde::Serialize, R: for<'de> serde::Deserialize<'de>>(
        &self,
        subject: &str,
        headers: async_nats::HeaderMap,
        payload: &T,
    ) -> anyhow::Result<R> {
        let payload = serde_json::to_vec(payload)?;

        // Send request with headers and wait for response
        let response = self
            .client
            .request_with_headers(subject.to_string(), headers, payload.into())
            .await?;

        // Deserialize response
        serde_json::from_slice(&response.payload).map_err(Into::into)
    }

    pub fn nats(&self) -> &async_nats::Client {
        &self.client
    }
}

// Implement Publisher trait for MessagingClient
#[async_trait::async_trait]
impl Publisher for MessagingClient {
    async fn publish_raw(&self, subject: String, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn publish<T: serde::Serialize + Send + Sync>(
        &self,
        subject: String,
        payload: &T,
    ) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(payload)?;
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
    }

    async fn request<
        T: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send,
    >(
        &self,
        subject: &str,
        payload: &T,
    ) -> anyhow::Result<R> {
        let payload = serde_json::to_vec(payload)?;

        let response = self
            .client
            .request(subject.to_string(), payload.into())
            .await?;

        serde_json::from_slice(&response.payload).map_err(Into::into)
    }

    async fn request_with_headers<
        T: serde::Serialize + Send + Sync,
        R: for<'de> serde::Deserialize<'de> + Send,
    >(
        &self,
        subject: &str,
        headers: async_nats::HeaderMap,
        payload: &T,
    ) -> anyhow::Result<R> {
        let payload = serde_json::to_vec(payload)?;

        let response = self
            .client
            .request_with_headers(subject.to_string(), headers, payload.into())
            .await?;

        serde_json::from_slice(&response.payload).map_err(Into::into)
    }
}

// Subscriber implementation for async_nats::Subscriber
struct NatsSubscriber {
    inner: async_nats::Subscriber,
}

#[async_trait::async_trait]
impl MessageStream for NatsSubscriber {
    async fn next(&mut self) -> Option<async_nats::Message> {
        self.inner.next().await
    }
}

#[async_trait::async_trait]
impl Subscriber for MessagingClient {
    async fn subscribe(&self, subject: String) -> anyhow::Result<Box<dyn MessageStream>> {
        let subscriber = self.client.subscribe(subject).await?;
        Ok(Box::new(NatsSubscriber { inner: subscriber }))
    }
}

#[cfg(feature = "mocks")]
impl MessagingClient {
    /// Create a mock messaging client for testing
    pub fn new_mock() -> Self {
        // Create a mock client that doesn't actually connect to NATS
        // This is used for testing purposes only
        panic!("new_mock() should only be used in tests with a proper mock implementation")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<T> {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub aggregate_id: String,
    pub payload: T,
    pub trace_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GlobalEvent {
    AuditLogCreated(AuditLog),
    WeatherStationCreated(WeatherStation),
    WeatherDataCollected(WeatherData),
    WeatherStationDataReceived(IoTTelemetryEvent),
    SoilMoistureAlert(SoilMoistureAlertEvent),
    IrrigationTriggered(IrrigationCommandEvent),
    PhenologyRecordCreated(PhenologyRecord),
    HealthCheckRequested,
    SiteCreated(Site),
    SiteUpdated(Site),
    SiteDeleted(Uuid),
    OrderCreated(Order),
    OrderUpdated(Order),
    OrderDeleted(Uuid),
    UserCreated(User),
    UserUpdated(User),
    UserDeleted(Uuid),
    TenantCreated(agrocore_domain::entities::tenant::Tenant),
    SpatialPolygonEntered(SpatialPresenceEvent),
    SpatialPolygonIn(SpatialPresenceEvent),
    SpatialPolygonLeft(SpatialPresenceEvent),
    // Webhook events
    WebhookDeliveryAttempted(WebhookDeliveryAttempt),
    WebhookDeliverySucceeded(WebhookDeliverySuccess),
    WebhookDeliveryFailed(WebhookDeliveryFailure),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SpatialPolygonEventKind {
    EnteredPolygon,
    InPolygon,
    LeftPolygon,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpatialPresenceEvent {
    pub tenant_id: Uuid,
    pub worker_id: Uuid,
    pub spatial_object_id: Uuid,
    pub spatial_object_type: SpatialObjectType,
    pub spatial_object_label: String,
    pub site_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub location: GeoPoint,
    pub observed_at: DateTime<Utc>,
    pub kind: SpatialPolygonEventKind,
}

// Webhook types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookSubscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub url: String,
    pub secret: Option<String>, // HMAC secret for signature verification
    pub events: Vec<String>,    // Event types to subscribe to
    pub is_active: bool,
    pub retry_policy: WebhookRetryPolicy,
    pub headers: Option<serde_json::Value>, // Custom headers
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WebhookRetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub dead_letter_url: Option<String>, // Optional dead letter queue URL
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookDeliveryAttempt {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub attempt: u32,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookDeliverySuccess {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub attempt: u32,
    pub response_status: u16,
    pub response_body: Option<String>,
    pub latency_ms: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookDeliveryFailure {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub attempts: u32,
    pub last_error: String,
    pub last_status: Option<u16>,
    pub moved_to_dead_letter: bool,
    pub created_at: DateTime<Utc>,
}

// Webhook subscription request/response DTOs
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateWebhookSubscriptionDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(url)]
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub retry_policy: Option<WebhookRetryPolicy>,
    pub headers: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate, Default)]
pub struct UpdateWebhookSubscriptionDto {
    pub name: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub retry_policy: Option<WebhookRetryPolicy>,
    pub headers: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebhookSubscriptionResponse {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub retry_policy: WebhookRetryPolicy,
    pub headers: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WebhookDeliveryLogResponse {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub attempt: u32,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl<T> Event<T> {
    pub fn new(aggregate_id: String, payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            aggregate_id,
            payload,
            trace_id: None,
        }
    }

    pub fn with_trace(aggregate_id: String, payload: T, trace_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            aggregate_id,
            payload,
            trace_id,
        }
    }
}

impl Default for WebhookRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 300000, // 5 minutes
            backoff_multiplier: 2.0,
            dead_letter_url: None,
        }
    }
}

/// Webhook-Event-Handler mit exponentiellem Retry-Backoff (max 3 Versuche).
pub async fn handle_webhook_event(
    event: &WebhookDeliveryAttempt,
    policy: &WebhookRetryPolicy,
) -> anyhow::Result<()> {
    let max_attempts = policy.max_attempts.min(3);
    for attempt in 1..=max_attempts {
        match try_deliver(event).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == max_attempts {
                    return Err(anyhow::anyhow!(
                        "Webhook delivery failed after {} attempts: {}",
                        max_attempts,
                        e
                    ));
                }
                let delay = std::time::Duration::from_millis(
                    policy.initial_delay_ms * 2u64.pow(attempt - 1),
                );
                warn!(
                    "Webhook attempt {}/{} failed: {}. Retrying in {:?}...",
                    attempt, max_attempts, e, delay
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
    Ok(())
}

async fn try_deliver(_event: &WebhookDeliveryAttempt) -> anyhow::Result<()> {
    // Delivery-Logik hier einfügen; als Stub Ok
    Ok(())
}

// ============================================================
// MQTT Configuration and IoT Event Types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub tls_ca_cert: Option<String>,
    pub tls_client_cert: Option<String>,
    pub tls_client_key: Option<String>,
    pub keep_alive: u16,
    pub clean_session: bool,
    pub topic_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceConfig {
    pub device_id: String,
    pub device_type: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub capabilities: Vec<IoTCapability>,
    pub metadata: serde_json::Value,
    pub topic_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
pub enum IoTCapability {
    Temperature,
    Humidity,
    SoilMoisture,
    Light,
    GPS,
    BatteryLevel,
    SignalStrength,
    ActuatorControl,
    FirmwareUpdate,
    Power,
    Energy,
    Pressure,
    Voltage,
    Current,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTTelemetryEvent {
    pub device_id: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub measurements: Vec<IoTMeasurement>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTMeasurement {
    pub capability: IoTCapability,
    pub value: f64,
    pub unit: String,
    pub quality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTCommandEvent {
    pub device_id: String,
    pub command_id: Uuid,
    pub command_type: String,
    pub payload: serde_json::Value,
    pub requested_at: DateTime<Utc>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceStatusEvent {
    pub device_id: String,
    pub tenant_id: Uuid,
    pub status: DeviceStatus,
    pub last_seen: DateTime<Utc>,
    pub firmware_version: Option<String>,
    pub battery_level: Option<f64>,
    pub signal_strength: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Error,
    Maintenance,
    Updating,
}

/// Event published when soil moisture drops below a configured threshold.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SoilMoistureAlertEvent {
    pub device_id: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub station_id: Option<Uuid>,
    pub moisture_percent: f64,
    pub threshold_percent: f64,
    pub timestamp: DateTime<Utc>,
    pub recommended_action: String,
}

/// Event published when an irrigation command is triggered.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IrrigationCommandEvent {
    pub device_id: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub station_id: Option<Uuid>,
    pub moisture_percent: f64,
    pub threshold_percent: f64,
    pub command: IrrigationCommand,
    pub triggered_at: DateTime<Utc>,
}

/// The type of irrigation action to take.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum IrrigationCommand {
    Start,
    Stop,
    SetDuration { minutes: u32 },
    SetMoistureThreshold { threshold_percent: f64 },
}

/// NATS subjects for IoT telemetry and alerts
pub const NATS_SUBJECT_TELEMETRY_WEATHER: &str = "telemetry.weather";
pub const NATS_SUBJECT_TELEMETRY_SOIL: &str = "telemetry.soil";
pub const NATS_SUBJECT_ALERTS_SOIL: &str = "alerts.soil_moisture";
pub const NATS_SUBJECT_COMMANDS_IRRIGATION: &str = "commands.irrigation";

// ============================================================
// Home Assistant MQTT Auto-Discovery
// ============================================================
// See: https://www.home-assistant.io/integrations/mqtt/#device-discovery

/// Home Assistant device class for sensors
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HaDeviceClass {
    Temperature,
    Humidity,
    Moisture,
    Illuminance,
    Battery,
    SignalStrength,
    Power,
    Energy,
    Pressure,
    Voltage,
    Current,
    TemperatureDevice,
    HumidityDevice,
}

/// Home Assistant unit of measurement
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HaUnitOfMeasurement {
    Celsius,
    Fahrenheit,
    Percent,
    Lux,
    Volts,
    Amperes,
    Watts,
    KilowattHours,
    Pascal,
    Hectopascal,
    Meter,
    Kilometer,
    MeterPerSecond,
    Db,
    DbM,
}

/// Generate Home Assistant MQTT auto-discovery configurations for an IoT device.
///
/// This function generates the JSON configuration payloads that Home Assistant
/// expects for MQTT auto-discovery. See: https://www.home-assistant.io/integrations/mqtt/#device-discovery
///
/// # Arguments
/// * `device` - The IoT device configuration
/// * `measurements` - The measurements this device provides
///
/// # Returns
/// A vector of (topic, payload) tuples where each tuple represents one sensor/entity
/// configuration for Home Assistant MQTT auto-discovery.
pub fn generate_ha_discovery_configs(
    device: &IoTDeviceConfig,
    measurements: &[IoTMeasurement],
) -> Vec<(String, serde_json::Value)> {
    let mut configs = Vec::new();
    let device_id = &device.device_id;
    let device_name = format!("AgroCore {}", device_id);

    // Device info for Home Assistant
    let device_info = serde_json::json!({
        "identifiers": [device_id],
        "name": device_name,
        "model": device.device_type,
        "manufacturer": "AgroCore",
        "sw_version": "1.0",
    });

    for measurement in measurements {
        let capability = &measurement.capability;
        let unit = &measurement.unit;

        // Map capability to Home Assistant device class and unit
        let (device_class, ha_unit) = match capability {
            IoTCapability::Temperature => {
                (HaDeviceClass::Temperature, HaUnitOfMeasurement::Celsius)
            }
            IoTCapability::Humidity => (HaDeviceClass::Humidity, HaUnitOfMeasurement::Percent),
            IoTCapability::SoilMoisture => (HaDeviceClass::Moisture, HaUnitOfMeasurement::Percent),
            IoTCapability::Light => (HaDeviceClass::Illuminance, HaUnitOfMeasurement::Lux),
            IoTCapability::BatteryLevel => (HaDeviceClass::Battery, HaUnitOfMeasurement::Percent),
            IoTCapability::SignalStrength => {
                (HaDeviceClass::SignalStrength, HaUnitOfMeasurement::DbM)
            }
            IoTCapability::Power => (HaDeviceClass::Power, HaUnitOfMeasurement::Watts),
            IoTCapability::Energy => (HaDeviceClass::Energy, HaUnitOfMeasurement::KilowattHours),
            IoTCapability::Pressure => (HaDeviceClass::Pressure, HaUnitOfMeasurement::Hectopascal),
            IoTCapability::Voltage => (HaDeviceClass::Voltage, HaUnitOfMeasurement::Volts),
            IoTCapability::Current => (HaDeviceClass::Current, HaUnitOfMeasurement::Amperes),
            IoTCapability::GPS => (HaDeviceClass::TemperatureDevice, HaUnitOfMeasurement::Meter),
            IoTCapability::ActuatorControl => (
                HaDeviceClass::TemperatureDevice,
                HaUnitOfMeasurement::Percent,
            ),
            IoTCapability::FirmwareUpdate => (
                HaDeviceClass::TemperatureDevice,
                HaUnitOfMeasurement::Percent,
            ),
            IoTCapability::Custom(_) => (
                HaDeviceClass::TemperatureDevice,
                HaUnitOfMeasurement::Percent,
            ),
        };

        let unique_id = format!("{}_{:?}", device_id, capability).to_lowercase();
        let state_topic = format!(
            "{}/{}",
            device.topic_prefix,
            capability_to_topic(capability)
        );

        let config = serde_json::json!({
            "name": format!("{} {}", device_name, format!("{:?}", capability)),
            "unique_id": unique_id,
            "device": device_info,
            "state_topic": state_topic,
            "device_class": format!("{:?}", device_class).to_lowercase(),
            "unit_of_measurement": format!("{:?}", ha_unit).to_lowercase(),
            "value_template": "{{ value_json.value }}",
            "availability_topic": format!("{}/status", device.topic_prefix),
            "payload_available": "online",
            "payload_not_available": "offline",
            "qos": 1,
            "retain": true,
        });

        let topic = format!("homeassistant/sensor/{}/config", unique_id);
        configs.push((topic, config));
    }

    configs
}

/// Convert IoTCapability to MQTT topic suffix
fn capability_to_topic(capability: &IoTCapability) -> String {
    match capability {
        IoTCapability::Temperature => "temperature".to_string(),
        IoTCapability::Humidity => "humidity".to_string(),
        IoTCapability::SoilMoisture => "soil_moisture".to_string(),
        IoTCapability::Light => "light".to_string(),
        IoTCapability::GPS => "gps".to_string(),
        IoTCapability::BatteryLevel => "battery".to_string(),
        IoTCapability::SignalStrength => "signal".to_string(),
        IoTCapability::ActuatorControl => "actuator".to_string(),
        IoTCapability::FirmwareUpdate => "firmware".to_string(),
        IoTCapability::Power => "power".to_string(),
        IoTCapability::Energy => "energy".to_string(),
        IoTCapability::Pressure => "pressure".to_string(),
        IoTCapability::Voltage => "voltage".to_string(),
        IoTCapability::Current => "current".to_string(),
        IoTCapability::Custom(s) => s.to_lowercase().replace(' ', "_"),
    }
}
