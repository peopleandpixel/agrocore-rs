//! Notification dispatcher for external channels.
//!
//! Provides a pluggable architecture for sending notifications via
//! Email (SMTP/SendGrid), Telegram, ntfy.sh, Webhooks, SMS, and WhatsApp.

pub mod channel;
pub mod config;
pub mod dispatcher;
pub mod template;
pub mod types;

pub use channel::create_channel;
pub use config::NotificationConfig;
pub use dispatcher::NotificationDispatcher;
pub use template::TemplateEngine;
pub use types::{
    ChannelConfig, ChannelError, ChannelMessage, DeliveryReport, MailgunChannelConfig,
    NotificationChannel, NtfyChannelConfig, SendgridChannelConfig, SmtpChannelConfig,
    TelegramChannelConfig, TwilioChannelConfig, WacliChannelConfig, WebhookChannelConfig,
    new_correlation_id,
};

// Re-export the concrete channels so callers can construct them directly.
pub use channel::{NtfyChannel, TelegramChannel, TwilioChannel, WacliChannel, WebhookChannel};

// NATS subjects for the notification pipeline
pub const NATS_SUBJECT_NOTIFICATIONS_SEND: &str = "notifications.send";
pub const NATS_SUBJECT_NOTIFICATIONS_SENT: &str = "notifications.sent";
pub const NATS_SUBJECT_NOTIFICATIONS_FAILED: &str = "notifications.failed";
