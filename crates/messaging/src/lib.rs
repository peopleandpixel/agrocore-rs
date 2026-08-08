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
use rumqttc::{AsyncClient, Event as MqttEvent, EventLoop, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
    pub keep_alive: u16,
    pub clean_session: bool,
    pub topic_prefix: String,
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

// MQTT Client wrapper
pub struct MqttClient {
    client: AsyncClient,
    event_loop: EventLoop,
    #[allow(dead_code)]
    config: MqttConfig,
    topic_prefix: String,
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
            // TLS configuration would go here
            // mqtt_options.set_transport(rumqttc::Transport::Tls(tls_config));
        }

        let (client, event_loop) = AsyncClient::new(mqtt_options, 100);

        Ok(Self {
            client,
            event_loop,
            topic_prefix: config.topic_prefix.clone(),
            config,
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
    pub async fn subscribe_commands(
        &mut self,
        tenant_id: Uuid,
        device_id: &str,
    ) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!("commands/{}/{}", tenant_id, device_id));

        self.client.subscribe(topic, QoS::AtLeastOnce).await?;

        Ok(())
    }

    /// Subscribe to broadcast commands (all devices in tenant)
    pub async fn subscribe_broadcast_commands(&mut self, tenant_id: Uuid) -> anyhow::Result<()> {
        let topic = self.build_topic(&format!("commands/{}/broadcast", tenant_id));

        self.client.subscribe(topic, QoS::AtLeastOnce).await?;

        Ok(())
    }

    /// Get next MQTT event from event loop
    pub async fn next_event(&mut self) -> Option<MqttEvent> {
        self.event_loop.poll().await.ok()
    }

    /// Process incoming MQTT events and handle them
    pub async fn process_events<F>(&mut self, mut handler: F) -> anyhow::Result<()>
    where
        F: FnMut(MqttEvent) -> anyhow::Result<()>,
    {
        loop {
            match self.event_loop.poll().await {
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
    nats: Option<MessagingClient>,
    mqtt: Option<MqttClient>,
}

impl UnifiedMessagingClient {
    pub async fn new(
        nats_url: Option<&str>,
        mqtt_config: Option<MqttConfig>,
    ) -> anyhow::Result<Self> {
        let nats = if let Some(url) = nats_url {
            Some(MessagingClient::connect(url).await?)
        } else {
            None
        };

        let mqtt = if let Some(config) = mqtt_config {
            Some(MqttClient::connect(config).await?)
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

        if let (Some(nats), Some(subject)) = (&self.nats, nats_subject)
            && let Err(e) = nats.publish(subject, event).await
        {
            errors.push(format!("NATS: {}", e));
        }

        if let (Some(mqtt), Some(topic)) = (&self.mqtt, mqtt_topic)
            && let Err(e) = mqtt.publish_event(topic, event, false).await
        {
            errors.push(format!("MQTT: {}", e));
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
        &mut self,
        tenant_id: Uuid,
        device_id: &str,
    ) -> anyhow::Result<()> {
        if let Some(mqtt) = &mut self.mqtt {
            mqtt.subscribe_commands(tenant_id, device_id).await
        } else {
            Err(anyhow::anyhow!("MQTT not configured"))
        }
    }

    /// Get NATS client if available
    pub fn nats(&self) -> Option<&MessagingClient> {
        self.nats.as_ref()
    }

    /// Get MQTT client if available
    pub fn mqtt(&self) -> Option<&MqttClient> {
        self.mqtt.as_ref()
    }
}

pub struct MessagingClient {
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
        let mut retry_count = 0;
        let max_retries = 10;

        let client = loop {
            match async_nats::connect(url).await {
                Ok(client) => break client,
                Err(e) if retry_count < max_retries => {
                    retry_count += 1;
                    tracing::warn!(
                        "Failed to connect to NATS (attempt {}/{}): {}. Retrying in 1s...",
                        retry_count,
                        max_retries,
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to NATS after {} attempts: {}",
                        max_retries,
                        e
                    ));
                }
            }
        };

        let circuit_breaker = Config::new().build();

        Ok(Self {
            client,
            circuit_breaker,
        })
    }

    /// Publiziert ein Event an NATS. Nutzt `bytes::Bytes` für zero-copy Payload.
    pub async fn publish<T: Serialize>(
        &self,
        subject: &str,
        event: &Event<T>,
    ) -> anyhow::Result<()> {
        let mut attempts = 0;
        let max_attempts = 3;
        let payload: Bytes = Bytes::from(serde_json::to_vec(event)?);

        loop {
            match self
                .client
                .publish(subject.to_string(), payload.clone())
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) if attempts < max_attempts => {
                    attempts += 1;
                    tracing::warn!("Failed to publish to NATS, attempt {}: {}", attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempts)).await;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to publish after {} attempts: {}",
                        max_attempts,
                        e
                    ));
                }
            }
        }
    }

    pub async fn subscribe(&self, subject: &str) -> anyhow::Result<async_nats::Subscriber> {
        let subscriber = self.client.subscribe(subject.to_string()).await?;
        Ok(subscriber)
    }

    pub async fn request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        subject: &str,
        payload: &T,
    ) -> anyhow::Result<R> {
        let mut attempts = 0;
        let max_attempts = 3;
        let payload_bytes: Bytes = Bytes::from(serde_json::to_vec(payload)?);

        loop {
            if !self.circuit_breaker.is_call_permitted() {
                return Err(anyhow::anyhow!(
                    "Circuit breaker is open for subject: {}",
                    subject
                ));
            }

            match self
                .client
                .request(subject.to_string(), payload_bytes.clone())
                .await
            {
                Ok(response) => {
                    self.circuit_breaker.on_success();
                    let result = serde_json::from_slice(&response.payload)?;
                    return Ok(result);
                }
                Err(e) if attempts < max_attempts => {
                    attempts += 1;
                    tracing::warn!("Failed to request from NATS, attempt {}: {}", attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(200 * attempts)).await;
                }
                Err(e) => {
                    self.circuit_breaker.on_error();
                    return Err(anyhow::anyhow!(
                        "Request failed after {} attempts: {}",
                        max_attempts,
                        e
                    ));
                }
            }
        }
    }

    pub async fn publish_raw(&self, subject: &str, payload: Vec<u8>) -> anyhow::Result<()> {
        self.client
            .publish(subject.to_string(), payload.into())
            .await?;
        Ok(())
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
