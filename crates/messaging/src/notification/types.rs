//! Core notification types: channel trait, message envelope, error type and
//! the per-channel configuration structs referenced by `channel.rs`.
//!
//! These live in their own module because `channel.rs` needs them and
//! `config.rs` (which holds the top-level `NotificationConfig`) needs
//! `ChannelConfig` — keeping them here avoids a cycle between the two.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Errors raised while delivering a notification.
#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("channel unavailable: {0}")]
    Unavailable(String),
    #[error("channel configuration error: {0}")]
    Config(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("provider API error: {0}")]
    Api(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("delivery failed: {0}")]
    Delivery(String),
    #[error("no channel configured for recipient")]
    NoChannel,
}

/// A single outbound notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    /// Channel name this message is routed to (e.g. "email", "telegram").
    pub channel: String,
    /// Destination: email address, phone number, chat id, topic, URL.
    pub recipient: String,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub is_html: bool,
    /// Template/event key used to render subject and body.
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Attempt counter, incremented by the dispatcher on retry.
    #[serde(default)]
    pub attempt: u32,
    /// Free-form payload used for template rendering and channel extras.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ChannelMessage {
    pub fn new(
        channel: impl Into<String>,
        recipient: impl Into<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            channel: channel.into(),
            recipient: recipient.into(),
            subject: subject.into(),
            body: body.into(),
            is_html: false,
            event_type: String::new(),
            correlation_id: None,
            timestamp: Utc::now(),
            attempt: 0,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Delivery result for a processed notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReport {
    pub channel: String,
    pub recipient: String,
    pub delivered: bool,
    pub attempts: u32,
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl DeliveryReport {
    pub fn success(message: &ChannelMessage, attempts: u32) -> Self {
        Self {
            channel: message.channel.clone(),
            recipient: message.recipient.clone(),
            delivered: true,
            attempts,
            correlation_id: message.correlation_id.clone(),
            error: None,
            completed_at: Utc::now(),
        }
    }

    pub fn failure(message: &ChannelMessage, attempts: u32, error: impl Into<String>) -> Self {
        Self {
            channel: message.channel.clone(),
            recipient: message.recipient.clone(),
            delivered: false,
            attempts,
            correlation_id: message.correlation_id.clone(),
            error: Some(error.into()),
            completed_at: Utc::now(),
        }
    }
}

/// A pluggable delivery backend.
///
/// Implementations are shared as `Arc<dyn NotificationChannel>` so no `Clone`
/// bound is needed (which would break dyn compatibility).
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Deliver the message. Implementations must not panic on failure.
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError>;

    /// Stable channel identifier used for routing and metrics.
    fn name(&self) -> &'static str;

    /// Cheap readiness probe used during dispatcher startup.
    fn health_check(&self) -> Result<(), ChannelError> {
        Ok(())
    }

    /// Whether the channel can currently accept traffic. Channels that do not
    /// implement a remote probe report `true`.
    async fn is_available(&self) -> bool {
        true
    }
}

// ── Per-channel configuration ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpChannelConfig {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    #[serde(default = "default_true")]
    pub tls: bool,
}

fn default_smtp_port() -> u16 {
    587
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendgridChannelConfig {
    pub api_key: String,
    pub from: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailgunChannelConfig {
    pub api_key: String,
    pub domain: String,
    pub from: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChannelConfig {
    pub bot_token: String,
    /// `HTML` or `MarkdownV2`; forwarded to the Bot API as `parse_mode`.
    #[serde(default)]
    pub parse_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtfyChannelConfig {
    #[serde(default = "default_ntfy_server")]
    pub server: String,
    pub default_topic: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

fn default_ntfy_server() -> String {
    "https://ntfy.sh".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookChannelConfig {
    /// Fallback endpoint used when a message carries no explicit recipient.
    #[serde(default)]
    pub default_url: String,
    pub url: String,
    /// Optional HMAC-SHA256 secret; the signature is sent as `X-Signature`.
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default = "default_webhook_timeout")]
    pub timeout_secs: u64,
}

fn default_webhook_timeout() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwilioChannelConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WacliChannelConfig {
    /// WhatsApp recipient in international format, e.g. `+4915112345678`.
    pub phone_number: String,
    /// Path to the `wacli` binary.
    #[serde(default = "default_wacli_binary")]
    pub binary_path: String,
}

fn default_wacli_binary() -> String {
    "wacli".to_string()
}

/// Channel configuration as stored in `NotificationConfig::channels`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChannelConfig {
    Smtp(SmtpChannelConfig),
    Sendgrid(SendgridChannelConfig),
    Mailgun(MailgunChannelConfig),
    Telegram(TelegramChannelConfig),
    Ntfy(NtfyChannelConfig),
    Webhook(WebhookChannelConfig),
    Twilio(TwilioChannelConfig),
    Wacli(WacliChannelConfig),
}

impl ChannelConfig {
    /// Canonical channel name, matching what `NotificationChannel::name`
    /// returns so routing and metrics stay consistent.
    pub fn channel_name(&self) -> &'static str {
        match self {
            ChannelConfig::Smtp(_) => "smtp",
            ChannelConfig::Sendgrid(_) => "sendgrid",
            ChannelConfig::Mailgun(_) => "mailgun",
            ChannelConfig::Telegram(_) => "telegram",
            ChannelConfig::Ntfy(_) => "ntfy",
            ChannelConfig::Webhook(_) => "webhook",
            ChannelConfig::Twilio(_) => "twilio",
            ChannelConfig::Wacli(_) => "wacli",
        }
    }

    /// Configuration errors caught before any network call is attempted.
    pub fn validate(&self) -> Result<(), ChannelError> {
        let missing = |field: &str| ChannelError::Config(format!("{field} must not be empty"));
        match self {
            ChannelConfig::Smtp(c) => {
                if c.host.trim().is_empty() {
                    return Err(missing("smtp.host"));
                }
                if c.from.trim().is_empty() {
                    return Err(missing("smtp.from"));
                }
                if c.password.is_empty() {
                    return Err(missing("smtp.password"));
                }
            }
            ChannelConfig::Sendgrid(c) => {
                if c.api_key.trim().is_empty() {
                    return Err(missing("sendgrid.api_key"));
                }
                if c.from.trim().is_empty() {
                    return Err(missing("sendgrid.from"));
                }
            }
            ChannelConfig::Mailgun(c) => {
                if c.api_key.trim().is_empty() {
                    return Err(missing("mailgun.api_key"));
                }
                if c.domain.trim().is_empty() {
                    return Err(missing("mailgun.domain"));
                }
            }
            ChannelConfig::Telegram(c) => {
                if c.bot_token.trim().is_empty() {
                    return Err(missing("telegram.bot_token"));
                }
            }
            ChannelConfig::Ntfy(c) => {
                if c.server.trim().is_empty() {
                    return Err(missing("ntfy.server"));
                }
                if c.default_topic.trim().is_empty() {
                    return Err(missing("ntfy.default_topic"));
                }
            }
            ChannelConfig::Webhook(c) => {
                if c.url.trim().is_empty() {
                    return Err(missing("webhook.url"));
                }
                if !(c.url.starts_with("http://") || c.url.starts_with("https://")) {
                    return Err(ChannelError::Config(
                        "webhook.url must be an http(s) URL".to_string(),
                    ));
                }
            }
            ChannelConfig::Twilio(c) => {
                if c.account_sid.trim().is_empty() {
                    return Err(missing("twilio.account_sid"));
                }
                if c.auth_token.trim().is_empty() {
                    return Err(missing("twilio.auth_token"));
                }
                if !c.from_number.trim().is_empty() && !c.from_number.starts_with('+') {
                    return Err(ChannelError::Config(
                        "twilio.from_number must use international format (+...)".to_string(),
                    ));
                }
            }
            ChannelConfig::Wacli(c) => {
                if c.phone_number.trim().is_empty() {
                    return Err(missing("wacli.phone_number"));
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.channel_name())
    }
}

/// Helper for building a stable idempotency/correlation key.
pub fn new_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_true() -> bool {
    true
}
