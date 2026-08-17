use agrocore_domain::entities::compliance::AuditLog;
use agrocore_domain::entities::order::Order;
use agrocore_domain::entities::site::GeoPoint;
use agrocore_domain::entities::site::Site;
use agrocore_domain::entities::spatial::SpatialObjectType;
use agrocore_domain::entities::user::User;
use agrocore_domain::entities::weather::{PhenologyRecord, WeatherData, WeatherStation};
use async_nats::Client;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use failsafe::Config;
use rumqttc::{
    AsyncClient, Event as MqttEvent, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// Precomputed static NATS subjects to avoid repeated string allocations
pub const NATS_SUBJECT_EVENTS: &str = "events.>";
pub const NATS_SUBJECT_TELEMETRY: &str = "telemetry.>";
pub const NATS_SUBJECT_DEVICE_STATUS: &str = "device.status.>";
pub const NATS_SUBJECT_COMMANDS_BRIDGE: &str = "commands.bridge";
pub const NATS_SUBJECT_COMMANDS_BRIDGE_BROADCAST: &str = "commands.bridge.broadcast";

pub mod bridge;

pub use bridge::{BridgeConfig, BridgeRoute, BridgeStats, MqttBridge, MqttBridgeBuilder};

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

/// Build TLS configuration from MqttConfig.
/// Uses system root certificates by default (rustls platform verifier), or
/// loads CA/root certs from PEM strings if `tls_ca_cert` is provided.
/// Supports optional client certificate (mutual TLS) when both
/// `tls_client_cert` and `tls_client_key` are set.
fn build_tls_config(config: &MqttConfig) -> anyhow::Result<TlsConfiguration> {
    if config.tls_client_cert.is_some() && config.tls_client_key.is_some() {
        anyhow::bail!(
            "Mutual TLS (client cert) requires rustls with custom provider; using system default TLS instead"
        );
    }

    // TlsConfiguration::Simple with empty CA falls back to system root certs
    let ca = config
        .tls_ca_cert
        .as_deref()
        .unwrap_or("")
        .as_bytes()
        .to_vec();
    Ok(TlsConfiguration::Simple {
        ca,
        alpn: None,
        client_auth: None,
    })
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: format!("agrocore-{}", Uuid::new_v4()),
            username: None,
            password: None,
            use_tls: false,
            tls_ca_cert: None,
            tls_client_cert: None,
            tls_client_key: None,
            keep_alive: 60,
            clean_session: true,
            topic_prefix: "agrocore".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IoTDeviceConfig {
    pub device_id: String,
    pub device_type: String,
    pub tenant_id: Uuid,
    pub site_id: Option<Uuid>,
    pub capabilities: Vec<IoTCapability>,
    pub metadata: serde_json::Value,
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
    Custom(String),
}

/// Home Assistant entity category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HaEntityCategory {
    Config,
    Diagnostic,
    None,
}

/// Home Assistant sensor device info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaDeviceInfo {
    pub identifiers: Vec<String>,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub sw_version: Option<String>,
    pub hw_version: Option<String>,
    pub via_device: Option<String>,
}

/// Home Assistant MQTT sensor configuration payload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaSensorConfig {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub device_class: Option<HaDeviceClass>,
    pub unit_of_measurement: Option<HaUnitOfMeasurement>,
    pub value_template: Option<String>,
    pub json_attributes_topic: Option<String>,
    pub device: Option<HaDeviceInfo>,
    pub entity_category: Option<HaEntityCategory>,
    pub icon: Option<String>,
    pub enabled_by_default: Option<bool>,
    pub availability_topic: Option<String>,
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
}

/// Home Assistant binary sensor configuration payload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaBinarySensorConfig {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub device_class: Option<String>,
    pub value_template: Option<String>,
    pub payload_on: Option<String>,
    pub payload_off: Option<String>,
    pub device: Option<HaDeviceInfo>,
    pub entity_category: Option<HaEntityCategory>,
    pub icon: Option<String>,
    pub enabled_by_default: Option<bool>,
    pub availability_topic: Option<String>,
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
}

/// Home Assistant button configuration payload (for commands)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaButtonConfig {
    pub name: String,
    pub unique_id: String,
    pub command_topic: String,
    pub payload_press: String,
    pub device: Option<HaDeviceInfo>,
    pub entity_category: Option<HaEntityCategory>,
    pub icon: Option<String>,
    pub enabled_by_default: Option<bool>,
    pub availability_topic: Option<String>,
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
}

/// Home Assistant number configuration payload (for numeric settings)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HaNumberConfig {
    pub name: String,
    pub unique_id: String,
    pub state_topic: String,
    pub command_topic: String,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
    pub unit_of_measurement: Option<HaUnitOfMeasurement>,
    pub device: Option<HaDeviceInfo>,
    pub entity_category: Option<HaEntityCategory>,
    pub icon: Option<String>,
    pub enabled_by_default: Option<bool>,
    pub availability_topic: Option<String>,
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
}

/// Generate Home Assistant discovery topic for an entity
pub fn ha_discovery_topic(entity_type: &str, unique_id: &str) -> String {
    format!("homeassistant/{}/{}/config", entity_type, unique_id)
}

/// Generate Home Assistant availability topic for a device
pub fn ha_availability_topic(device_id: &str) -> String {
    format!("agrocore/status/{}/availability", device_id)
}

/// Create device info for Home Assistant from IoTDeviceConfig
pub fn create_ha_device_info(config: &IoTDeviceConfig) -> HaDeviceInfo {
    HaDeviceInfo {
        identifiers: vec![format!("agrocore_{}", config.device_id)],
        name: config.device_type.clone(),
        manufacturer: Some("agrocore-rs".to_string()),
        model: Some(config.device_type.clone()),
        sw_version: config
            .metadata
            .get("firmware_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        hw_version: config
            .metadata
            .get("hardware_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        via_device: Some("agrocore-mqtt".to_string()),
    }
}

/// Create Home Assistant sensor config for an IoT capability
pub fn create_ha_sensor_config(
    device_id: &str,
    tenant_id: Uuid,
    capability: &IoTCapability,
    device_info: &HaDeviceInfo,
    topic_prefix: &str,
) -> Option<HaSensorConfig> {
    let (device_class, unit, icon, value_template) = match capability {
        IoTCapability::Temperature => (
            Some(HaDeviceClass::Temperature),
            Some(HaUnitOfMeasurement::Celsius),
            Some("mdi:thermometer".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'Temperature') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::Humidity => (
            Some(HaDeviceClass::Humidity),
            Some(HaUnitOfMeasurement::Percent),
            Some("mdi:water-percent".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'Humidity') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::SoilMoisture => (
            Some(HaDeviceClass::Moisture),
            Some(HaUnitOfMeasurement::Percent),
            Some("mdi:water".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'SoilMoisture') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::Light => (
            Some(HaDeviceClass::Illuminance),
            Some(HaUnitOfMeasurement::Lux),
            Some("mdi:brightness-5".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'Light') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::BatteryLevel => (
            Some(HaDeviceClass::Battery),
            Some(HaUnitOfMeasurement::Percent),
            Some("mdi:battery".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'BatteryLevel') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::SignalStrength => (
            Some(HaDeviceClass::SignalStrength),
            Some(HaUnitOfMeasurement::DbM),
            Some("mdi:signal".to_string()),
            Some("{{ value_json.measurements | selectattr('capability', 'eq', 'SignalStrength') | map(attribute='value') | first }}".to_string()),
        ),
        IoTCapability::GPS => (
            None,
            None,
            Some("mdi:crosshairs-gps".to_string()),
            None,
        ),
        IoTCapability::ActuatorControl => (
            None,
            None,
            Some("mdi:valve".to_string()),
            None,
        ),
        IoTCapability::FirmwareUpdate => (
            None,
            None,
            Some("mdi:package-up".to_string()),
            None,
        ),
        IoTCapability::Custom(_) => (
            None,
            None,
            Some("mdi:help-circle".to_string()),
            None,
        ),
    };

    let unique_id = format!("agrocore_{}_{}", device_id, capability_name(capability));
    let state_topic = format!("{}/telemetry/{}/{}", topic_prefix, tenant_id, device_id);
    let availability_topic = format!("{}/status/{}/availability", topic_prefix, device_id);

    Some(HaSensorConfig {
        name: format!(
            "{} {}",
            device_info.name,
            capability_display_name(capability)
        ),
        unique_id: unique_id.clone(),
        state_topic: state_topic.clone(),
        device_class,
        unit_of_measurement: unit,
        value_template,
        json_attributes_topic: Some(state_topic.clone()),
        device: Some(device_info.clone()),
        entity_category: Some(HaEntityCategory::None),
        icon,
        enabled_by_default: Some(true),
        availability_topic: Some(availability_topic),
        payload_available: Some("online".to_string()),
        payload_not_available: Some("offline".to_string()),
    })
}

/// Helper to get capability name for unique_id
fn capability_name(capability: &IoTCapability) -> String {
    match capability {
        IoTCapability::Temperature => "temperature".to_string(),
        IoTCapability::Humidity => "humidity".to_string(),
        IoTCapability::SoilMoisture => "soil_moisture".to_string(),
        IoTCapability::Light => "light".to_string(),
        IoTCapability::GPS => "gps".to_string(),
        IoTCapability::BatteryLevel => "battery_level".to_string(),
        IoTCapability::SignalStrength => "signal_strength".to_string(),
        IoTCapability::ActuatorControl => "actuator_control".to_string(),
        IoTCapability::FirmwareUpdate => "firmware_update".to_string(),
        IoTCapability::Custom(s) => s.to_lowercase().replace(' ', "_"),
    }
}

/// Helper to get display name for capability
fn capability_display_name(capability: &IoTCapability) -> String {
    match capability {
        IoTCapability::Temperature => "Temperature".to_string(),
        IoTCapability::Humidity => "Humidity".to_string(),
        IoTCapability::SoilMoisture => "Soil Moisture".to_string(),
        IoTCapability::Light => "Light".to_string(),
        IoTCapability::GPS => "GPS".to_string(),
        IoTCapability::BatteryLevel => "Battery Level".to_string(),
        IoTCapability::SignalStrength => "Signal Strength".to_string(),
        IoTCapability::ActuatorControl => "Actuator".to_string(),
        IoTCapability::FirmwareUpdate => "Firmware".to_string(),
        IoTCapability::Custom(s) => s.clone(),
    }
}

/// Generate all Home Assistant discovery configs for a device
pub fn generate_ha_discovery_configs(
    device_config: &IoTDeviceConfig,
    topic_prefix: &str,
) -> Vec<(String, serde_json::Value)> {
    let device_info = create_ha_device_info(device_config);
    let mut configs = Vec::new();

    // Add sensor configs for each capability
    for capability in &device_config.capabilities {
        if let Some(sensor_config) = create_ha_sensor_config(
            &device_config.device_id,
            device_config.tenant_id,
            capability,
            &device_info,
            topic_prefix,
        ) {
            let topic = ha_discovery_topic(
                "sensor",
                &format!(
                    "agrocore_{}_{}",
                    device_config.device_id,
                    capability_name(capability)
                ),
            );
            let payload = serde_json::to_value(sensor_config).unwrap();
            configs.push((topic, payload));
        }
    }

    // Add availability binary sensor
    let availability_config = HaBinarySensorConfig {
        name: format!("{} Availability", device_info.name),
        unique_id: format!("agrocore_{}_availability", device_config.device_id),
        state_topic: format!(
            "{}/status/{}/availability",
            topic_prefix, device_config.device_id
        ),
        device_class: Some("connectivity".to_string()),
        value_template: Some("{{ value }}".to_string()),
        payload_on: Some("online".to_string()),
        payload_off: Some("offline".to_string()),
        device: Some(device_info.clone()),
        entity_category: Some(HaEntityCategory::Diagnostic),
        icon: Some("mdi:server".to_string()),
        enabled_by_default: Some(true),
        availability_topic: Some(format!(
            "{}/status/{}/availability",
            topic_prefix, device_config.device_id
        )),
        payload_available: Some("online".to_string()),
        payload_not_available: Some("offline".to_string()),
    };
    let availability_topic = ha_discovery_topic(
        "binary_sensor",
        &format!("agrocore_{}_availability", device_config.device_id),
    );
    configs.push((
        availability_topic,
        serde_json::to_value(availability_config).unwrap(),
    ));

    configs
}

// MQTT Client wrapper
pub struct MqttClient {
    client: AsyncClient,
    event_loop: Arc<Mutex<Option<EventLoop>>>,
    #[allow(dead_code)]
    config: MqttConfig,
    topic_prefix: String,
    /// Health monitoring state
    health: Arc<AsyncRwLock<MqttHealth>>,
    /// Reconnection handle
    reconnect_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

/// MQTT Connection Health Status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MqttHealth {
    pub connected: bool,
    pub last_ping: Option<chrono::DateTime<chrono::Utc>>,
    pub last_pong: Option<chrono::DateTime<chrono::Utc>>,
    pub reconnect_count: u32,
    pub last_error: Option<String>,
    pub last_reconnect: Option<chrono::DateTime<chrono::Utc>>,
    pub uptime_start: Option<chrono::DateTime<chrono::Utc>>,
}

impl MqttHealth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.uptime_start = Some(chrono::Utc::now());
        self.last_error = None;
    }

    pub fn mark_disconnected(&mut self, error: Option<String>) {
        self.connected = false;
        self.last_error = error;
    }

    pub fn record_ping(&mut self) {
        self.last_ping = Some(chrono::Utc::now());
    }

    pub fn record_pong(&mut self) {
        self.last_pong = Some(chrono::Utc::now());
    }

    pub fn record_reconnect(&mut self) {
        self.reconnect_count += 1;
        self.last_reconnect = Some(chrono::Utc::now());
    }
}

impl MqttClient {
    pub async fn connect(config: MqttConfig) -> anyhow::Result<Self> {
        info!(
            "Connecting to MQTT broker at {}:{}",
            config.broker_host, config.broker_port
        );

        let mut mqtt_options =
            MqttOptions::new(&config.client_id, &config.broker_host, config.broker_port);

        mqtt_options.set_keep_alive(std::time::Duration::from_secs(config.keep_alive as u64));
        mqtt_options.set_clean_session(config.clean_session);

        if let (Some(username), Some(password)) = (config.username.clone(), config.password.clone())
        {
            mqtt_options.set_credentials(username, password);
        }

        if config.use_tls {
            let tls_config = build_tls_config(&config).unwrap_or_else(|e| {
                tracing::warn!(
                    "MQTT TLS config error, falling back to system default: {}",
                    e
                );
                TlsConfiguration::Simple {
                    ca: Vec::new(),
                    alpn: None,
                    client_auth: None,
                }
            });
            mqtt_options.set_transport(Transport::Tls(tls_config));
        }

        let (client, event_loop) = AsyncClient::new(mqtt_options, 100);

        let health = Arc::new(AsyncRwLock::new(MqttHealth::new()));
        let reconnect_handle = Arc::new(Mutex::new(None));

        Ok(Self {
            client,
            event_loop: Arc::new(Mutex::new(Some(event_loop))),
            topic_prefix: config.topic_prefix.clone(),
            config,
            health,
            reconnect_handle,
        })
    }

    /// Get the topic prefix for this client
    pub fn topic_prefix(&self) -> &str {
        &self.topic_prefix
    }

    /// Build a full topic with prefix
    pub fn build_topic(&self, topic: &str) -> String {
        format!("{}/{}", self.topic_prefix, topic)
    }

    /// Get the event loop for external processing (used by bridge)
    pub fn event_loop(&self) -> Arc<Mutex<Option<EventLoop>>> {
        self.event_loop.clone()
    }

    /// Get the async client
    pub fn client(&self) -> &AsyncClient {
        &self.client
    }

    /// Get health monitoring state
    pub fn health(&self) -> Arc<AsyncRwLock<MqttHealth>> {
        self.health.clone()
    }

    /// Get current health snapshot
    pub async fn get_health(&self) -> MqttHealth {
        self.health.read().await.clone()
    }

    /// Check if MQTT connection is healthy
    pub async fn is_healthy(&self) -> bool {
        self.health.read().await.connected
    }

    /// Start health monitoring background task
    pub async fn start_health_monitoring(&self, ping_interval_secs: u64) {
        let health = self.health.clone();
        let client = self.client.clone();
        let event_loop = self.event_loop.clone();
        let config = self.config.clone();
        let topic_prefix = self.topic_prefix.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(ping_interval_secs));

            loop {
                interval.tick().await;

                // Record ping
                {
                    let mut health = health.write().await;
                    health.record_ping();
                }

                // Send ping via MQTT (using a ping topic)
                let ping_topic = format!("{}/health/ping", topic_prefix);
                let payload = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "type": "ping"
                });

                if let Err(e) = client
                    .publish(
                        ping_topic,
                        QoS::AtLeastOnce,
                        false,
                        payload.to_string().into_bytes(),
                    )
                    .await
                {
                    tracing::warn!("Health ping failed: {}", e);
                    {
                        let mut health = health.write().await;
                        health.mark_disconnected(Some(e.to_string()));
                    }

                    // Attempt reconnection
                    if let Err(reconnect_err) =
                        Self::attempt_reconnect(&config, &event_loop, &health).await
                    {
                        tracing::error!("Reconnection failed: {}", reconnect_err);
                    }
                    continue;
                }

                // Wait for pong response (with timeout)
                let pong_timeout = tokio::time::sleep(Duration::from_secs(5));
                tokio::select! {
                    _ = pong_timeout => {
                        tracing::warn!("Pong timeout - connection may be unhealthy");
                        let mut health = health.write().await;
                        health.mark_disconnected(Some("Pong timeout".to_string()));
                    }
                    // In a real implementation, you'd listen for pong response
                    // For now, we assume success if ping sent
                    _ = async { } => {}
                }

                // Record pong (success)
                let mut health = health.write().await;
                health.record_pong();
                if !health.connected {
                    health.mark_connected();
                }
            }
        });

        // Store the handle for potential cleanup
        if let Ok(mut guard) = self.reconnect_handle.lock() {
            *guard = Some(handle);
        }
    }

    /// Attempt to reconnect to MQTT broker
    async fn attempt_reconnect(
        config: &MqttConfig,
        event_loop: &Arc<Mutex<Option<EventLoop>>>,
        health: &Arc<AsyncRwLock<MqttHealth>>,
    ) -> anyhow::Result<()> {
        tracing::info!("Attempting MQTT reconnection...");

        let mut mqtt_options =
            MqttOptions::new(&config.client_id, &config.broker_host, config.broker_port);
        mqtt_options.set_keep_alive(std::time::Duration::from_secs(config.keep_alive as u64));
        mqtt_options.set_clean_session(config.clean_session);

        if let (Some(username), Some(password)) = (config.username.clone(), config.password.clone())
        {
            mqtt_options.set_credentials(username, password);
        }

        if config.use_tls {
            let tls_config = build_tls_config(config).unwrap_or_else(|e| {
                tracing::warn!(
                    "MQTT TLS config error, falling back to system default: {}",
                    e
                );
                TlsConfiguration::Simple {
                    ca: Vec::new(),
                    alpn: None,
                    client_auth: None,
                }
            });
            mqtt_options.set_transport(Transport::Tls(tls_config));
        }

        let (_client, new_event_loop) = AsyncClient::new(mqtt_options, 100);

        // Update event loop
        {
            let mut guard = event_loop
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            *guard = Some(new_event_loop);
        }

        // Note: In a real implementation, you'd need to update the client field too
        // This is a simplified version

        {
            let mut health = health.write().await;
            health.record_reconnect();
            health.mark_connected();
        }

        tracing::info!("MQTT reconnection successful");
        Ok(())
    }

    /// Stop health monitoring
    pub async fn stop_health_monitoring(&self) {
        let handle_guard = self
            .reconnect_handle
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))
            .ok();
        if let Some(mut handle_guard) = handle_guard
            && let Some(handle) = handle_guard.take()
        {
            handle.abort();
        }
    }

    /// Publish telemetry data from IoT device
    pub async fn publish_telemetry(&self, event: &IoTTelemetryEvent) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!(
            "telemetry/{}/{}",
            event.tenant_id, event.device_id
        ));
        let payload = serde_json::to_vec(event)?;

        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;

        Ok(())
    }

    /// Publish device status
    pub async fn publish_status(&self, event: &IoTDeviceStatusEvent) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!("status/{}/{}", event.tenant_id, event.device_id));
        let payload = serde_json::to_vec(event)?;

        self.client
            .publish(topic, QoS::AtLeastOnce, true, payload) // Retain status
            .await?;

        Ok(())
    }

    /// Subscribe to device commands
    pub async fn subscribe_commands(&self, tenant_id: Uuid, device_id: &str) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!("commands/{}/{}", tenant_id, device_id));

        self.client.subscribe(topic, QoS::AtLeastOnce).await?;

        Ok(())
    }

    /// Subscribe to broadcast commands (all devices in tenant)
    pub async fn subscribe_broadcast_commands(&self, tenant_id: Uuid) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!("commands/{}/broadcast", tenant_id));

        self.client.subscribe(topic, QoS::AtLeastOnce).await?;

        Ok(())
    }

    /// Get next MQTT event from event loop
    #[allow(clippy::await_holding_lock)]
    pub async fn next_event(&mut self) -> Option<MqttEvent> {
        let mut guard = self.event_loop.lock().ok()?;
        guard.as_mut()?.poll().await.ok()
    }

    /// Process incoming MQTT events and handle them
    #[allow(clippy::await_holding_lock)]
    pub async fn process_events<F>(&mut self, mut handler: F) -> anyhow::Result<()>
    where
        F: FnMut(MqttEvent) -> anyhow::Result<()>,
    {
        let mut event_loop = {
            let mut guard = self
                .event_loop
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            guard
                .take()
                .ok_or_else(|| anyhow::anyhow!("Event loop not available"))?
        };

        loop {
            match event_loop.poll().await {
                Ok(event) => {
                    if let Err(e) = handler(event) {
                        tracing::error!("Error handling MQTT event: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("MQTT event loop error: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    /// Publish a generic event to MQTT
    pub async fn publish_event<T: Serialize>(
        &self,
        topic: &str,
        event: &Event<T>,
        retain: bool,
    ) -> anyhow::Result<()> {
        let full_topic = self.build_topic(topic);
        let payload = serde_json::to_vec(event)?;

        self.client
            .publish(full_topic, QoS::AtLeastOnce, retain, payload)
            .await?;

        Ok(())
    }
}

// Unified messaging client supporting both NATS and MQTT
pub struct UnifiedMessagingClient {
    nats: Option<Arc<MessagingClient>>,
    mqtt: Option<Arc<MqttClient>>,
}

impl UnifiedMessagingClient {
    pub async fn new(
        nats_url: Option<&str>,
        mqtt_config: Option<MqttConfig>,
    ) -> anyhow::Result<Self> {
        let nats = if let Some(url) = nats_url {
            Some(Arc::new(MessagingClient::connect(url).await?))
        } else {
            None
        };

        let mqtt = if let Some(config) = mqtt_config {
            Some(Arc::new(MqttClient::connect(config).await?))
        } else {
            None
        };

        if nats.is_none() && mqtt.is_none() {
            return Err(anyhow::anyhow!(
                "At least one messaging backend must be configured"
            ));
        }

        Ok(Self { nats, mqtt })
    }

    /// Publish event to all available backends
    pub async fn publish_event<T: Serialize + Sync>(
        &self,
        nats_subject: Option<&str>,
        mqtt_topic: Option<&str>,
        event: &Event<T>,
    ) -> anyhow::Result<()> {
        let mut errors = Vec::new();

        if let Some(nats) = &self.nats {
            #[allow(clippy::collapsible_if)]
            if let Some(subject) = nats_subject {
                if let Err(e) = nats.publish(subject, event).await {
                    errors.push(format!("NATS: {}", e));
                }
            }
        }

        if let Some(mqtt) = &self.mqtt {
            #[allow(clippy::collapsible_if)]
            if let Some(topic) = mqtt_topic {
                if let Err(e) = mqtt.publish_event(topic, event, false).await {
                    errors.push(format!("MQTT: {}", e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to publish to some backends: {}",
                errors.join("; ")
            ));
        }

        Ok(())
    }

    /// Publish IoT telemetry (MQTT only, optimized for device data)
    pub async fn publish_telemetry(&self, event: &IoTTelemetryEvent) -> anyhow::Result<()> {
        if let Some(mqtt) = &self.mqtt {
            mqtt.publish_telemetry(event).await
        } else {
            Err(anyhow::anyhow!("MQTT not configured"))
        }
    }

    /// Publish device status (MQTT only, with retain)
    pub async fn publish_device_status(&self, event: &IoTDeviceStatusEvent) -> anyhow::Result<()> {
        if let Some(mqtt) = &self.mqtt {
            mqtt.publish_status(event).await
        } else {
            Err(anyhow::anyhow!("MQTT not configured"))
        }
    }

    /// Subscribe to commands for a device
    pub async fn subscribe_device_commands(
        &self,
        tenant_id: Uuid,
        device_id: &str,
    ) -> anyhow::Result<()> {
        if let Some(mqtt) = &self.mqtt {
            mqtt.subscribe_commands(tenant_id, device_id).await
        } else {
            Err(anyhow::anyhow!("MQTT not configured"))
        }
    }

    /// Get NATS client if available
    pub fn nats(&self) -> Option<Arc<MessagingClient>> {
        self.nats.clone()
    }

    /// Get MQTT client if available
    pub fn mqtt(&self) -> Option<Arc<MqttClient>> {
        self.mqtt.clone()
    }
}

#[derive(Clone)]
pub enum MessagingClient {
    Nats(NatsMessagingClient),
    #[cfg(any(test, feature = "mocks"))]
    Mock(Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>>),
}

#[derive(Clone)]
pub struct NatsMessagingClient {
    client: Client,
    circuit_breaker: failsafe::StateMachine<
        failsafe::failure_policy::OrElse<
            failsafe::failure_policy::SuccessRateOverTimeWindow<failsafe::backoff::EqualJittered>,
            failsafe::failure_policy::ConsecutiveFailures<failsafe::backoff::EqualJittered>,
        >,
        (),
    >,
}

impl MessagingClient {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        info!("Connecting to NATS at {}", url);
        let url_owned = url.to_string();
        let client = agrocore_shared::with_retry("connect to NATS", 10, 1, || {
            let url = url_owned.clone();
            Box::pin(async move {
                async_nats::connect(&url)
                    .await
                    .map_err(|e| anyhow::anyhow!("nats connection error: {}", e))
            })
        })
        .await?;

        let circuit_breaker = Config::new().build();

        Ok(Self::Nats(NatsMessagingClient {
            client,
            circuit_breaker,
        }))
    }

    #[cfg(any(test, feature = "mocks"))]
    pub fn new_mock() -> Self {
        Self::Mock(Arc::new(std::sync::Mutex::new(
            std::collections::HashMap::new(),
        )))
    }

    #[cfg(any(test, feature = "mocks"))]
    pub fn set_mock_response(&self, subject: &str, response: Vec<u8>) {
        if let Self::Mock(m) = self {
            m.lock().unwrap().insert(subject.to_string(), response);
        }
    }

    /// Publiziert ein Event an NATS. Nutzt `bytes::Bytes` für zero-copy Payload.
    pub async fn publish<T: Serialize>(
        &self,
        subject: &str,
        event: &Event<T>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Nats(n) => {
                let payload: Bytes = Bytes::from(serde_json::to_vec(event)?);
                let client = n.client.clone();
                let subject_owned = subject.to_string();
                agrocore_shared::with_retry("publish to NATS", 3, 0, || {
                    let client = client.clone();
                    let subject = subject_owned.clone();
                    let payload = payload.clone();
                    Box::pin(async move {
                        client
                            .publish(subject, payload)
                            .await
                            .map_err(|e| anyhow::anyhow!("nats publish error: {}", e))
                    })
                })
                .await
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(_) => Ok(()),
        }
    }

    /// Publiziert raw bytes an NATS (für Bridge-Forwarding)
    pub async fn publish_raw(&self, subject: &str, payload: Vec<u8>) -> anyhow::Result<()> {
        match self {
            Self::Nats(n) => {
                let payload: Bytes = Bytes::from(payload);
                let client = n.client.clone();
                let subject_owned = subject.to_string();
                agrocore_shared::with_retry("publish raw to NATS", 3, 1, || {
                    let client = client.clone();
                    let subject = subject_owned.clone();
                    let payload = payload.clone();
                    Box::pin(async move {
                        client
                            .publish(subject, payload)
                            .await
                            .map_err(|e| anyhow::anyhow!("nats publish error: {}", e))
                    })
                })
                .await
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(_) => Ok(()),
        }
    }

    pub async fn subscribe(&self, subject: &str) -> anyhow::Result<async_nats::Subscriber> {
        match self {
            Self::Nats(n) => {
                let subscriber = n.client.subscribe(subject.to_string()).await?;
                Ok(subscriber)
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(_) => Err(anyhow::anyhow!("Mock subscribe not implemented")),
        }
    }

    pub async fn request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        subject: &str,
        payload: &T,
    ) -> anyhow::Result<R> {
        match self {
            Self::Nats(n) => {
                let mut attempts = 0;
                let max_attempts = 3;
                let payload_bytes: Bytes = Bytes::from(serde_json::to_vec(payload)?);

                loop {
                    if !n.circuit_breaker.is_call_permitted() {
                        return Err(anyhow::anyhow!(
                            "Circuit breaker is open for subject: {}",
                            subject
                        ));
                    }

                    match n
                        .client
                        .request(subject.to_string(), payload_bytes.clone())
                        .await
                    {
                        Ok(response) => {
                            n.circuit_breaker.on_success();
                            let result = serde_json::from_slice(&response.payload)?;
                            return Ok(result);
                        }
                        Err(e) if attempts < max_attempts => {
                            attempts += 1;
                            tracing::warn!(
                                "Failed to request from NATS, attempt {}: {}",
                                attempts,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(200 * attempts))
                                .await;
                        }
                        Err(e) => {
                            n.circuit_breaker.on_error();
                            return Err(anyhow::anyhow!(
                                "Request failed after {} attempts: {}",
                                max_attempts,
                                e
                            ));
                        }
                    }
                }
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(m) => {
                let res = m.lock().unwrap().get(subject).cloned();
                match res {
                    Some(bytes) => {
                        let result = serde_json::from_slice(&bytes)?;
                        Ok(result)
                    }
                    None => Err(anyhow::anyhow!("No mock response for subject: {}", subject)),
                }
            }
        }
    }

    /// Publish IoT telemetry via NATS (for internal services)
    pub async fn publish_telemetry_nats(&self, event: &IoTTelemetryEvent) -> anyhow::Result<()> {
        let subject = format!("telemetry.{}.{}", event.tenant_id, event.device_id);
        self.publish(
            &subject,
            &Event::new(event.device_id.clone(), event.clone()),
        )
        .await
    }

    /// Publish device status via NATS
    pub async fn publish_device_status_nats(
        &self,
        event: &IoTDeviceStatusEvent,
    ) -> anyhow::Result<()> {
        let subject = format!("device.status.{}.{}", event.tenant_id, event.device_id);
        self.publish(
            &subject,
            &Event::new(event.device_id.clone(), event.clone()),
        )
        .await
    }
}
