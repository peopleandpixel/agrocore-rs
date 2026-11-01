//! Tenant user notification preferences.
//!
//! Each tenant user can configure which channels they receive notifications on,
//! their quiet hours (do-not-disturb), and escalation rules.

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};

/// User notification preferences stored per tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    /// User ID
    pub user_id: String,
    /// Tenant ID
    pub tenant_id: String,

    /// Enabled notification channels for this user.
    /// If empty, all channels are enabled.
    #[serde(default)]
    pub enabled_channels: Vec<NotificationChannelType>,

    /// Disabled notification channels (takes precedence over enabled).
    #[serde(default)]
    pub disabled_channels: Vec<NotificationChannelType>,

    /// Quiet hours configuration (do-not-disturb).
    #[serde(default)]
    pub quiet_hours: Option<QuietHours>,

    /// Escalation rules for critical notifications.
    #[serde(default)]
    pub escalation: Vec<EscalationRule>,

    /// Minimum severity level to receive notifications.
    #[serde(default = "default_min_severity")]
    pub min_severity: NotificationSeverity,

    /// Whether to receive digest summaries instead of individual notifications.
    #[serde(default)]
    pub digest_mode: bool,

    /// Digest frequency if digest_mode is enabled.
    #[serde(default = "default_digest_frequency")]
    pub digest_frequency: DigestFrequency,
}

fn default_min_severity() -> NotificationSeverity {
    NotificationSeverity::Info
}

fn default_digest_frequency() -> DigestFrequency {
    DigestFrequency::Daily
}

/// Notification channel types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannelType {
    Email,
    Sms,
    Push,
    WhatsApp,
    Telegram,
    Ntfy,
    Webhook,
}

/// Notification severity levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Quiet hours (do-not-disturb) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// Start time in HH:MM format (24h).
    pub start: String,
    /// End time in HH:MM format (24h).
    pub end: String,
    /// Timezone (IANA format, e.g. "Europe/Berlin").
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Days of the week when quiet hours apply.
    /// Empty means all days.
    #[serde(default)]
    pub days: Vec<Weekday>,
    /// Whether to allow critical notifications during quiet hours.
    #[serde(default = "default_allow_critical")]
    pub allow_critical: bool,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_allow_critical() -> bool {
    true
}

/// Days of the week.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Escalation rule for critical notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// Description of the escalation rule.
    pub description: String,
    /// Severity level that triggers this escalation.
    pub severity: NotificationSeverity,
    /// Delay in minutes before escalating.
    pub delay_minutes: u32,
    /// Channel to escalate to.
    pub escalate_to: NotificationChannelType,
    /// Optional: phone number or email for the escalation target.
    pub target: Option<String>,
}

/// Digest frequency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DigestFrequency {
    /// No digest, send immediately.
    Immediate,
    /// Hourly digest.
    Hourly,
    /// Daily digest.
    Daily,
    /// Weekly digest.
    Weekly,
}

impl UserNotificationPreferences {
    /// Create default preferences for a new user.
    pub fn new(user_id: String, tenant_id: String) -> Self {
        Self {
            user_id,
            tenant_id,
            enabled_channels: Vec::new(),
            disabled_channels: Vec::new(),
            quiet_hours: None,
            escalation: Vec::new(),
            min_severity: NotificationSeverity::Info,
            digest_mode: false,
            digest_frequency: DigestFrequency::Daily,
        }
    }

    /// Check if a channel is enabled for this user.
    pub fn is_channel_enabled(&self, channel: &NotificationChannelType) -> bool {
        // Disabled takes precedence
        if !self.disabled_channels.is_empty() {
            return !self.disabled_channels.contains(channel);
        }
        // If enabled list is empty, all channels are enabled
        if self.enabled_channels.is_empty() {
            return true;
        }
        // Otherwise check the enabled list
        self.enabled_channels.contains(channel)
    }

    /// Check if a notification should be sent based on severity.
    pub fn should_notify(&self, severity: &NotificationSeverity) -> bool {
        severity >= &self.min_severity
    }

    /// Check if it's currently quiet hours.
    /// Returns true if notifications should be suppressed.
    pub fn is_quiet_hours(&self, now: &chrono::DateTime<chrono::Utc>) -> bool {
        let quiet = match &self.quiet_hours {
            Some(q) => q,
            None => return false,
        };

        // Parse times
        let start = match parse_time(&quiet.start) {
            Some(t) => t,
            None => return false,
        };
        let end = match parse_time(&quiet.end) {
            Some(t) => t,
            None => return false,
        };

        // Use UTC if timezone parsing fails (simple approach)
        let local = *now;

        // Check day of week
        if !quiet.days.is_empty() {
            let weekday = match local.weekday() {
                chrono::Weekday::Mon => Weekday::Monday,
                chrono::Weekday::Tue => Weekday::Tuesday,
                chrono::Weekday::Wed => Weekday::Wednesday,
                chrono::Weekday::Thu => Weekday::Thursday,
                chrono::Weekday::Fri => Weekday::Friday,
                chrono::Weekday::Sat => Weekday::Saturday,
                chrono::Weekday::Sun => Weekday::Sunday,
            };
            if !quiet.days.contains(&weekday) {
                return false;
            }
        }

        let current_minutes = local.hour() * 60 + local.minute();
        let start_minutes = start.0 * 60 + start.1;
        let end_minutes = end.0 * 60 + end.1;

        if start_minutes < end_minutes {
            // Normal case: start < end (e.g., 09:00 - 17:00)
            current_minutes >= start_minutes && current_minutes < end_minutes
        } else {
            // Overnight case: start > end (e.g., 22:00 - 07:00)
            current_minutes >= start_minutes || current_minutes < end_minutes
        }
    }

    /// Get escalation rules for a given severity.
    pub fn get_escalation_rules(&self, severity: &NotificationSeverity) -> Vec<&EscalationRule> {
        self.escalation
            .iter()
            .filter(|r| r.severity == *severity)
            .collect()
    }
}

fn parse_time(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some((hour, minute))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_user_has_all_channels_enabled() {
        let prefs = UserNotificationPreferences::new("user1".to_string(), "tenant1".to_string());
        assert!(prefs.is_channel_enabled(&NotificationChannelType::Email));
        assert!(prefs.is_channel_enabled(&NotificationChannelType::Sms));
    }

    #[test]
    fn disabled_channel_overrides_enabled() {
        let mut prefs =
            UserNotificationPreferences::new("user1".to_string(), "tenant1".to_string());
        prefs.disabled_channels = vec![NotificationChannelType::Sms];
        assert!(prefs.is_channel_enabled(&NotificationChannelType::Email));
        assert!(!prefs.is_channel_enabled(&NotificationChannelType::Sms));
    }

    #[test]
    fn severity_filtering() {
        let mut prefs =
            UserNotificationPreferences::new("user1".to_string(), "tenant1".to_string());
        prefs.min_severity = NotificationSeverity::Warning;
        assert!(!prefs.should_notify(&NotificationSeverity::Info));
        assert!(prefs.should_notify(&NotificationSeverity::Warning));
        assert!(prefs.should_notify(&NotificationSeverity::Critical));
    }
}
