//! Notification dispatcher for external channels.
//!
//! Provides a pluggable architecture for sending notifications via
//! Email (SMTP/SendGrid), Telegram, ntfy.sh, Webhooks, SMS, and WhatsApp.

pub mod channel;
pub mod config;
pub mod dispatcher;
pub mod template;

pub use channel::{
    ChannelConfig, ChannelError, ChannelMessage, NotificationChannel, TelegramChannel, WebhookChannel,
};
pub use config::NotificationConfig;
pub use dispatcher::NotificationDispatcher;
pub use template::TemplateEngine;

// NATS subject for notification dispatch queue
pub const NATS_SUBJECT_NOTIFICATIONS_SEND: &str = "notifications.send";
pub const NATS_SUBJECT_NOTIFICATIONS_SENT: &str = "notifications.sent";
pub const NATS_SUBJECT_NOTIFICATIONS_FAILED: &str = "notifications.failed";
