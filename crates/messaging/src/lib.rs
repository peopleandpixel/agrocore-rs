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
use uuid::Uuid;

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
    SpatialPolygonEntered(SpatialPresenceEvent),
    SpatialPolygonIn(SpatialPresenceEvent),
    SpatialPolygonLeft(SpatialPresenceEvent),
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
        let client = async_nats::connect(url).await?;

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
                    ))
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
