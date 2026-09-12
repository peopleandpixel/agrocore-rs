//! Notification configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level notification configuration.
/// Configures which channels are available and per-channel settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationConfig {
    /// Whether notifications are enabled overall.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Channel configurations, keyed by channel name (e.g. "email", "telegram").
    #[serde(default)]
    pub channels: HashMap<String, super::ChannelConfig>,
    /// Notification templates, keyed by event type.
    #[serde(default)]
    pub templates: HashMap<String, TemplateConfig>,
    /// Global retry policy for failed notifications.
    #[serde(default)]
    pub retry: RetryConfig,
    /// Dead-letter subject for permanently failed notifications.
    #[serde(default = "default_dlq")]
    pub dead_letter_subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    /// Template for the subject/title.
    pub subject: String,
    /// Template for the message body.
    pub body: String,
    /// Channel-specific overrides keyed by channel name.
    #[serde(default)]
    pub overrides: HashMap<String, ChannelTemplateOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelTemplateOverride {
    pub subject: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 300000, // 5 minutes
            backoff_multiplier: 2.0,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_dlq() -> String {
    "notifications.failed".to_string()
}
