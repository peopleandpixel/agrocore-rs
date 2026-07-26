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
}
