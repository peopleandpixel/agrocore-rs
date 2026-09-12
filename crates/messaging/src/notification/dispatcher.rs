//! Notification dispatcher: consumes from NATS and routes to configured channels.

use super::channel::{ChannelError, ChannelMessage, NotificationChannel, create_channel};
use super::config::{NotificationConfig, RetryConfig, TemplateConfig};
use super::template::TemplateEngine;
use agrocore_logging::{debug, error, info, warn};
use async_nats::Client as NatsClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A notification request published to the NATS queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    /// Event type (e.g., "order.created", "task.assigned")
    pub event_type: String,
    /// Tenant ID for routing to the right config
    pub tenant_id: Uuid,
    /// Recipient channel identifier (email, telegram_chat_id, phone, webhook_url, ntfy_topic)
    pub recipient: String,
    /// Channel type to use (email, telegram, ntfy, webhook, sms, whatsapp)
    pub channel: String,
    /// Override subject (if not set, template is used)
    pub subject: Option<String>,
    /// Override body (if not set, template is used)
    pub body: Option<String>,
    /// Additional data for template rendering
    #[serde(default)]
    pub data: serde_json::Value,
    /// Correlation ID for tracking
    pub correlation_id: Option<String>,
}

/// Result of a notification delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub request_id: Uuid,
    pub event_type: String,
    pub recipient: String,
    pub channel: String,
    pub success: bool,
    pub attempts: u32,
    pub error: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
}

/// Notification dispatcher that consumes from NATS and routes to channels.
pub struct NotificationDispatcher {
    config: NotificationConfig,
    nats: NatsClient,
    channels: Arc<RwLock<HashMap<String, Box<dyn NotificationChannel>>>>,
    template_engine: TemplateEngine,
    retry_config: RetryConfig,
}

impl NotificationDispatcher {
    pub async fn new(config: NotificationConfig, nats: NatsClient) -> Result<Self, ChannelError> {
        let retry_config = config.retry.clone();
        let mut channels: HashMap<String, Box<dyn NotificationChannel>> = HashMap::new();

        for (name, channel_config) in &config.channels {
            match create_channel(channel_config) {
                Ok(channel) => {
                    info!("Registered notification channel: {}", name);
                    channels.insert(name.clone(), channel);
                }
                Err(e) => {
                    warn!("Failed to create channel '{}': {}", name, e);
                }
            }
        }

        Ok(Self {
            config,
            nats,
            channels: Arc::new(RwLock::new(channels)),
            template_engine: TemplateEngine::new(),
            retry_config,
        })
    }

    /// Start consuming from the NATS notifications.send subject.
    pub async fn start(&self) -> Result<(), ChannelError> {
        info!("NotificationDispatcher started — consuming from notifications.send");

        let subscriber = self
            .nats
            .subscribe("notifications.send".to_string())
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let dispatcher = self.clone_inner();
        let dlq_subject = self.config.dead_letter_subject.clone();

        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                let request: Option<NotificationRequest> = serde_json::from_slice(&msg.payload).ok();

                if let Some(request) = request {
                    dispatcher.process_notification(request, &dlq_subject).await;
                } else {
                    error!("Failed to deserialize NotificationRequest from NATS message");
                }
            }
        });

        Ok(())
    }

    fn clone_inner(&self) -> Arc<Self> {
        // We need Arc<Self> for the spawned task, but NotificationDispatcher
        // contains Box<dyn NotificationChannel> which is not Clone.
        // We'll wrap the channels in Arc<RwLock<...>> instead of cloning.
        // For the async consumer, we use a different approach:
        Arc::new(DispatcherRef {
            config: self.config.clone(),
            nats: self.nats.clone(),
            channels: self.channels.clone(),
            template_engine: self.template_engine.clone(),
            retry_config: self.retry_config.clone(),
        })
    }

    /// Process a single notification request with retry logic.
    async fn process_notification(
        &self,
        request: NotificationRequest,
        dlq_subject: &str,
    ) {
        let request_id = Uuid::new_v4();
        let started_at = Utc::now();

        info!(
            "Processing notification for event '{}' to '{}' via '{}' channel",
            request.event_type, request.recipient, request.channel
        );

        // Find the channel
        let channel = {
            let channels = self.channels.read().await;
            channels.get(&request.channel).cloned()
        };

        let Some(channel) = channel else {
            error!(
                "No channel configured for '{}' — event type: {}",
                request.channel, request.event_type
            );
            self.publish_result(NotificationResult {
                request_id,
                event_type: request.event_type.clone(),
                recipient: request.recipient.clone(),
                channel: request.channel.clone(),
                success: false,
                attempts: 0,
                error: Some(format!("Channel '{}' not configured", request.channel)),
                sent_at: None,
            })
            .await;
            return;
        };

        // Render templates if subject/body not provided
        let (subject, body) = self.render_message(&request).await;

        // Retry loop
        let mut attempts = 0u32;
        let mut last_error: Option<String> = None;

        loop {
            attempts += 1;

            if attempts > self.retry_config.max_attempts {
                error!(
                    "Notification to {} failed after {} attempts: {:?}",
                    request.recipient, attempts, last_error
                );

                // Move to dead-letter queue
                let dlq_msg = serde_json::json!({
                    "request": request,
                    "attempts": attempts,
                    "last_error": last_error,
                    "failed_at": Utc::now(),
                });
                let _ = self
                    .nats
                    .publish_raw(dlq_subject.to_string(), serde_json::to_vec(&dlq_msg).unwrap_or_default())
                    .await;

                self.publish_result(NotificationResult {
                    request_id,
                    event_type: request.event_type.clone(),
                    recipient: request.recipient.clone(),
                    channel: request.channel.clone(),
                    success: false,
                    attempts,
                    error: last_error.clone(),
                    sent_at: None,
                })
                .await;
                return;
            }

            let message = ChannelMessage {
                recipient: request.recipient.clone(),
                subject: subject.clone(),
                body: body.clone(),
                is_html: request.metadata.get("is_html").and_then(|v| v.as_bool()).unwrap_or(false),
                correlation_id: request.correlation_id.clone(),
                timestamp: started_at,
                metadata: request.data.clone(),
            };

            match channel.send(message).await {
                Ok(_) => {
                    info!(
                        "Notification sent to {} via {} (attempt {})",
                        request.recipient, request.channel, attempts
                    );
                    self.publish_result(NotificationResult {
                        request_id,
                        event_type: request.event_type.clone(),
                        recipient: request.recipient.clone(),
                        channel: request.channel.clone(),
                        success: true,
                        attempts,
                        error: None,
                        sent_at: Some(Utc::now()),
                    })
                    .await;
                    return;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    if attempts < self.retry_config.max_attempts {
                        let delay_ms = self
                            .retry_config
                            .initial_delay_ms
                            * (self.retry_config.backoff_multiplier as u64).pow(attempts - 1);
                        let delay_ms = delay_ms.min(self.retry_config.max_delay_ms);
                        debug!(
                            "Notification attempt {}/{} failed, retrying in {}ms: {}",
                            attempts, self.retry_config.max_attempts, delay_ms, e
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
    }

    /// Render subject and body from templates or use overrides.
    async fn render_message(&self, request: &NotificationRequest) -> (String, String) {
        // If subject and body are provided, use them directly
        if let (Some(subject), Some(body)) = (&request.subject, &request.body) {
            return (subject.clone(), body.clone());
        }

        // Look up template for event type
        let template: Option<&TemplateConfig> = self.config.templates.get(&request.event_type);

        if let Some(template) = template {
            let data_map: HashMap<String, String> = if let Value::Object(map) = &request.data {
                map.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            } else {
                HashMap::new()
            };

            let subject = if let Some(s) = &request.subject {
                s.clone()
            } else {
                self.template_engine
                    .render(&template.subject, &data_map)
                    .unwrap_or_else(|_| template.subject.clone())
            };

            let body = if let Some(b) = &request.body {
                b.clone()
            } else {
                self.template_engine
                    .render(&template.body, &data_map)
                    .unwrap_or_else(|_| template.body.clone())
            };

            (subject, body)
        } else {
            // Fallback: use event type as subject, data as body
            let subject = request
                .subject
                .clone()
                .unwrap_or_else(|| format!("Notification: {}", request.event_type));
            let body = request
                .body
                .clone()
                .unwrap_or_else(|| serde_json::to_string_pretty(&request.data).unwrap_or_default());
            (subject, body)
        }
    }

    /// Publish a delivery result event to NATS.
    async fn publish_result(&self, result: NotificationResult) {
        let subject = if result.success {
            "notifications.sent"
        } else {
            "notifications.failed"
        };

        let payload = serde_json::to_vec(&result).unwrap_or_default();
        if let Err(e) = self
            .nats
            .publish_raw(subject.to_string(), payload)
            .await
        {
            warn!("Failed to publish notification result: {}", e);
        }
    }
}

/// Lightweight reference for async consumption (avoids Clone on Box<dyn>).
struct DispatcherRef {
    config: NotificationConfig,
    nats: NatsClient,
    channels: Arc<RwLock<HashMap<String, Box<dyn NotificationChannel>>>>,
    template_engine: TemplateEngine,
    retry_config: RetryConfig,
}

impl DispatcherRef {
    async fn process_notification(
        &self,
        request: NotificationRequest,
        dlq_subject: &str,
    ) {
        // Delegate to Dispatcher logic (shared implementation)
        let dispatcher = NotificationDispatcher {
            config: self.config.clone(),
            nats: self.nats.clone(),
            channels: self.channels.clone(),
            template_engine: self.template_engine.clone(),
            retry_config: self.retry_config.clone(),
        };
        dispatcher.process_notification(request, dlq_subject).await;
    }
}
