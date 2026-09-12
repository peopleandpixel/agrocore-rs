//! Concrete notification channel implementations.

use super::channel::{
    ChannelError, ChannelMessage, ChannelConfig,
    NotificationChannel, SmtpChannelConfig, SendgridChannelConfig,
    MailgunChannelConfig, TelegramChannelConfig, NtfyChannelConfig,
    WebhookChannelConfig, TwilioChannelConfig, WacliChannelConfig,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// ── SMTP Channel ──────────────────────────────────────────────

/// SMTP-based email notification channel.
pub struct SmtpChannel {
    config: SmtpChannelConfig,
}

impl SmtpChannel {
    pub fn new(config: SmtpChannelConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NotificationChannel for SmtpChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        // In production: use a proper SMTP library like lettre
        // For now, we construct the email and would send via SMTP
        let _email_body = format!(
            "To: {}\r\nFrom: {}\r\nSubject: {}\r\n\r\n{}",
            message.recipient, self.config.from, message.subject, message.body
        );
        // SMTP send would go here — using lettre crate in full impl
        Err(ChannelError::Unavailable)
    }

    fn name(&self) -> &'static str {
        "smtp"
    }
}

// ── SendGrid Channel ──────────────────────────────────────────

/// SendGrid API-based email notification channel.
pub struct SendgridChannel {
    config: SendgridChannelConfig,
    client: reqwest::Client,
}

impl SendgridChannel {
    pub fn new(config: SendgridChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for SendgridChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let payload = serde_json::json!({
            "personalizations": vec![serde_json::json!({
                "to": vec![serde_json::json!({"email": message.recipient})],
                "subject": message.subject
            })],
            "from": {"email": self.config.from},
            "content": vec![serde_json::json!({
                "type": if message.is_html { "text/html" } else { "text/plain" },
                "value": message.body
            })]
        });

        let resp = self
            .client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("SendGrid API error: {}", resp.status())))
        }
    }

    fn name(&self) -> &'static str {
        "sendgrid"
    }
}

// ── Mailgun Channel ───────────────────────────────────────────

/// Mailgun API-based email notification channel.
pub struct MailgunChannel {
    config: MailgunChannelConfig,
    client: reqwest::Client,
}

impl MailgunChannel {
    pub fn new(config: MailgunChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for MailgunChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let form = serde_json::json!({
            "from": self.config.from,
            "to": message.recipient,
            "subject": message.subject,
            "text": message.body,
            "html": if message.is_html { message.body } else { "".to_string() }
        });

        let resp = self
            .client
            .post(format!("https://api.mailgun.net/v3/{}/messages", self.config.domain))
            .header("Authorization", format!("Basic {}", base64_encode(&format!("api:{}", self.config.api_key))))
            .json(&form)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("Mailgun API error: {}", resp.status())))
        }
    }

    fn name(&self) -> &'static str {
        "mailgun"
    }
}

// ── Telegram Channel ──────────────────────────────────────────

/// Telegram Bot API notification channel.
pub struct TelegramChannel {
    config: TelegramChannelConfig,
    client: reqwest::Client,
}

impl TelegramChannel {
    pub fn new(config: TelegramChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Send a message to a Telegram chat by chat_id.
    pub async fn send_to_chat(
        &self,
        chat_id: &str,
        message: &ChannelMessage,
    ) -> Result<(), ChannelError> {
        let parse_mode = self.config.parse_mode.as_deref().unwrap_or("HTML");
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": message.subject, // Use subject as caption
            "parse_mode": parse_mode,
        });

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.bot_token
        );

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("Telegram API error: {}", resp.status())))
        }
    }
}

#[async_trait]
impl NotificationChannel for TelegramChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        self.send_to_chat(&message.recipient, &message).await
    }

    fn name(&self) -> &'static str {
        "telegram"
    }

    async fn is_available(&self) -> bool {
        let url = format!(
            "https://api.telegram.org/bot{}/getMe",
            self.config.bot_token
        );
        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

// ── ntfy Channel ───────────────────────────────────────────────

/// ntfy.sh notification channel (publishes to a topic).
pub struct NtfyChannel {
    config: NtfyChannelConfig,
    client: reqwest::Client,
}

impl NtfyChannel {
    pub fn new(config: NtfyChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for NtfyChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let topic = if message.recipient.is_empty() {
            self.config.default_topic.clone()
        } else {
            message.recipient
        };

        let url = format!("{}/{}", self.config.server.trim_end_matches('/'), topic);

        let mut req = self
            .client
            .publish(&url)
            .header("Title", &message.subject)
            .header("Tags", "warning")
            .header("Priority", "urgent")
            .body(message.body.clone());

        if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
            req = req.header(
                "Authorization",
                format!("Basic {}", base64_encode(&format!("{}:{}", user, pass))),
            );
        }

        let resp = req
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("ntfy API error: {}", resp.status())))
        }
    }

    fn name(&self) -> &'static str {
        "ntfy"
    }
}

// ── Webhook Channel ───────────────────────────────────────────

/// Generic HTTP webhook notification channel with HMAC signing.
pub struct WebhookChannel {
    config: WebhookChannelConfig,
    client: reqwest::Client,
}

impl WebhookChannel {
    pub fn new(config: WebhookChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Compute HMAC-SHA256 signature for webhook payload.
    fn sign_payload(&self, payload: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.config.secret.as_bytes())
            .expect("HMAC accepted empty key");
        mac.update(payload);
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let target_url = message
            .metadata
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
            .trim()
            .to_string();

        let url = if target_url.is_empty() {
            self.config
                .default_url
                .clone()
                .ok_or_else(|| ChannelError::Config("No webhook URL provided".to_string()))?
        } else {
            target_url
        };

        let timestamp = message.timestamp.timestamp();
        let payload = serde_json::json!({
            "subject": message.subject,
            "body": message.body,
            "recipient": message.recipient,
            "is_html": message.is_html,
            "timestamp": timestamp,
            "correlation_id": message.correlation_id,
        });

        let body = serde_json::to_vec(&payload).map_err(|e| ChannelError::Serialization(e.to_string()))?;

        let signature = self.sign_payload(&body);

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-AgroCore-Signature", signature)
            .header("X-AgroCore-Timestamp", timestamp.to_string())
            .body(body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("Webhook API error: {}", resp.status())))
        }
    }

    fn name(&self) -> &'static str {
        "webhook"
    }
}

// ── Twilio Channel ────────────────────────────────────────────

/// Twilio SMS notification channel.
pub struct TwilioChannel {
    config: TwilioChannelConfig,
    client: reqwest::Client,
}

impl TwilioChannel {
    pub fn new(config: TwilioChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for TwilioChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let payload = serde_json::json!({
            "From": self.config.from_number,
            "To": message.recipient,
            "Body": format!("{}\n\n{}", message.subject, message.body),
        });

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.account_sid
        );

        let auth = base64_encode(&format!("{}:{}", self.config.account_sid, self.config.auth_token));

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .form(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("Twilio API error: {}", resp.status())))
        }
    }

    fn name(&self) -> &'static str {
        "twilio"
    }
}

// ── WACLI Channel ──────────────────────────────────────────────

/// WhatsApp notification channel via wacli CLI.
pub struct WacliChannel {
    config: WacliChannelConfig,
}

impl WacliChannel {
    pub fn new(config: WacliChannelConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NotificationChannel for WacliChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let body = format!("{}: {}", message.subject, message.body);

        let output = tokio::process::Command::new("wacli")
            .arg("--from")
            .arg(&self.config.phone_number)
            .arg("--to")
            .arg(&message.recipient)
            .arg("--text")
            .arg(&body)
            .output()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ChannelError::Api(format!("wacli error: {}", stderr)))
        }
    }

    fn name(&self) -> &'static str {
        "wacli"
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
}

// ── Channel factory ───────────────────────────────────────────

/// Create a channel instance from its configuration enum variant.
pub fn create_channel(config: &ChannelConfig) -> Result<Box<dyn NotificationChannel>, ChannelError> {
    match config {
        ChannelConfig::Smtp(cfg) => Ok(Box::new(SmtpChannel::new(cfg.clone()))),
        ChannelConfig::Sendgrid(cfg) => Ok(Box::new(SendgridChannel::new(cfg.clone()))),
        ChannelConfig::Mailgun(cfg) => Ok(Box::new(MailgunChannel::new(cfg.clone()))),
        ChannelConfig::Telegram(cfg) => Ok(Box::new(TelegramChannel::new(cfg.clone()))),
        ChannelConfig::Ntfy(cfg) => Ok(Box::new(NtfyChannel::new(cfg.clone()))),
        ChannelConfig::Webhook(cfg) => Ok(Box::new(WebhookChannel::new(cfg.clone()))),
        ChannelConfig::Twilio(cfg) => Ok(Box::new(TwilioChannel::new(cfg.clone()))),
        ChannelConfig::Wacli(cfg) => Ok(Box::new(WacliChannel::new(cfg.clone()))),
    }
}
