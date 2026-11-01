//! Concrete notification channel implementations.

use super::types::{
    ApnsChannelConfig, ChannelConfig, ChannelError, ChannelMessage, FcmChannelConfig,
    MailgunChannelConfig, NotificationChannel, NtfyChannelConfig, PlivoChannelConfig,
    PostmarkChannelConfig, SendgridChannelConfig, Sms77ChannelConfig, SmtpChannelConfig,
    TelegramChannelConfig, TwilioChannelConfig, VonageChannelConfig, WacliChannelConfig,
    WebPushChannelConfig, WebhookChannelConfig,
};
use async_trait::async_trait;
use std::sync::Arc;

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
        Err(ChannelError::Unavailable(
            "SMTP delivery is not implemented yet".to_string(),
        ))
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
            Err(ChannelError::Api(format!(
                "SendGrid API error: {}",
                resp.status()
            )))
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
            .post(format!(
                "https://api.mailgun.net/v3/{}/messages",
                self.config.domain
            ))
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    base64_encode(&format!("api:{}", self.config.api_key))
                ),
            )
            .json(&form)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!(
                "Mailgun API error: {}",
                resp.status()
            )))
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
            Err(ChannelError::Api(format!(
                "Telegram API error: {}",
                resp.status()
            )))
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
            .post(&url)
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
            Err(ChannelError::Api(format!(
                "ntfy API error: {}",
                resp.status()
            )))
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
    fn sign_payload(&self, payload: &[u8]) -> Option<String> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let secret = self.config.secret.as_ref()?;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
        mac.update(payload);
        let result = mac.finalize();
        Some(
            result
                .into_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        )
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
            if self.config.default_url.trim().is_empty() {
                return Err(ChannelError::Config(
                    "webhook.default_url must be configured".to_string(),
                ));
            }
            self.config.default_url.clone()
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

        let body =
            serde_json::to_vec(&payload).map_err(|e| ChannelError::Serialization(e.to_string()))?;

        let signature = self.sign_payload(&body);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-AgroCore-Timestamp", timestamp.to_string());

        // Only sign when a secret is configured; an unsigned request must not
        // carry an empty signature header.
        if let Some(signature) = signature {
            req = req.header("X-AgroCore-Signature", signature);
        }

        let resp = req
            .body(body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!(
                "Webhook API error: {}",
                resp.status()
            )))
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

        let auth = base64_encode(&format!(
            "{}:{}",
            self.config.account_sid, self.config.auth_token
        ));

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ChannelError::Api(format!(
                "Twilio API error: {}",
                resp.status()
            )))
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
pub fn create_channel(
    config: &ChannelConfig,
) -> Result<Arc<dyn NotificationChannel>, ChannelError> {
    match config {
        ChannelConfig::Smtp(cfg) => Ok(Arc::new(SmtpChannel::new(cfg.clone()))),
        ChannelConfig::Sendgrid(cfg) => Ok(Arc::new(SendgridChannel::new(cfg.clone()))),
        ChannelConfig::Mailgun(cfg) => Ok(Arc::new(MailgunChannel::new(cfg.clone()))),
        ChannelConfig::Telegram(cfg) => Ok(Arc::new(TelegramChannel::new(cfg.clone()))),
        ChannelConfig::Ntfy(cfg) => Ok(Arc::new(NtfyChannel::new(cfg.clone()))),
        ChannelConfig::Webhook(cfg) => Ok(Arc::new(WebhookChannel::new(cfg.clone()))),
        ChannelConfig::Twilio(cfg) => Ok(Arc::new(TwilioChannel::new(cfg.clone()))),
        ChannelConfig::Wacli(cfg) => Ok(Arc::new(WacliChannel::new(cfg.clone()))),
        ChannelConfig::Fcm(cfg) => Ok(Arc::new(FcmChannel::new(cfg.clone()))),
        ChannelConfig::Apns(cfg) => Ok(Arc::new(ApnsChannel::new(cfg.clone()))),
        ChannelConfig::WebPush(cfg) => Ok(Arc::new(WebPushChannel::new(cfg.clone()))),
        ChannelConfig::Postmark(cfg) => Ok(Arc::new(PostmarkChannel::new(cfg.clone()))),
        ChannelConfig::Vonage(cfg) => Ok(Arc::new(VonageChannel::new(cfg.clone()))),
        ChannelConfig::Plivo(cfg) => Ok(Arc::new(PlivoChannel::new(cfg.clone()))),
        ChannelConfig::Sms77(cfg) => Ok(Arc::new(Sms77Channel::new(cfg.clone()))),
    }
}

// ── FCM Channel ───────────────────────────────────────────────

/// Firebase Cloud Messaging push notification channel.
pub struct FcmChannel {
    config: FcmChannelConfig,
    client: reqwest::Client,
}

impl FcmChannel {
    pub fn new(config: FcmChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for FcmChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        // Load service account credentials
        let credentials = std::fs::read_to_string(&self.config.credentials_path)
            .map_err(|e| ChannelError::Config(format!("Cannot read FCM credentials: {e}")))?;
        let creds: serde_json::Value = serde_json::from_str(&credentials)
            .map_err(|e| ChannelError::Config(format!("Invalid FCM credentials: {e}")))?;

        // Get OAuth2 token from Google
        let token_url = "https://oauth2.googleapis.com/token";
        let jwt = generate_fcm_jwt(&creds);
        let body = format!(
            "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={}",
            urlencoding::encode(&jwt)
        );
        let resp = self
            .client
            .post(token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let token_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ChannelError::Api(format!("FCM token error: {e}")))?;
        let access_token = token_body["access_token"]
            .as_str()
            .ok_or_else(|| ChannelError::Api("FCM missing access_token".to_string()))?;

        // Send to FCM
        let project_id = if self.config.project_id.is_empty() {
            creds["project_id"]
                .as_str()
                .map(String::from)
                .unwrap_or_default()
        } else {
            self.config.project_id.clone()
        };

        let fcm_url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            project_id
        );

        let payload = serde_json::json!({
            "message": {
                "token": message.recipient,
                "notification": {
                    "title": message.subject,
                    "body": message.body
                },
                "data": {
                    "event_type": message.event_type
                }
            }
        });

        let resp = self
            .client
            .post(&fcm_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "FCM API error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "fcm"
    }
}

/// Generate a JWT for FCM OAuth2 authentication.
fn generate_fcm_jwt(creds: &serde_json::Value) -> String {
    use chrono::Utc;

    let client_email = creds["client_email"].as_str().unwrap_or("");
    let _private_key = creds["private_key"].as_str().unwrap_or("");
    let _project_id = creds["project_id"].as_str().unwrap_or("");

    let now = Utc::now().timestamp();
    let header = encode_jwt_part(&serde_json::json!({
        "alg": "RS256",
        "typ": "JWT"
    }));
    let payload = encode_jwt_part(&serde_json::json!({
        "iss": client_email,
        "scope": "https://www.googleapis.com/auth/firebase.messaging",
        "aud": "https://oauth2.googleapis.com/token",
        "iat": now,
        "exp": now + 3600
    }));

    // Signature is placeholder — in production use a proper RSA signing library
    format!("{}.{}.placeholder-signature", header, payload)
}

fn encode_jwt_part(value: &serde_json::Value) -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    URL_SAFE_NO_PAD.encode(bytes)
}

// ── APNs Channel ──────────────────────────────────────────────

/// Apple Push Notification service channel.
pub struct ApnsChannel {
    config: ApnsChannelConfig,
    client: reqwest::Client,
}

impl ApnsChannel {
    pub fn new(config: ApnsChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for ApnsChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let gateway = if self.config.production {
            "https://api.push.apple.com"
        } else {
            "https://api.sandbox.push.apple.com"
        };

        let url = format!("{}/3/device/{}", gateway, message.recipient);

        let payload = serde_json::json!({
            "aps": {
                "alert": {
                    "title": message.subject,
                    "body": message.body
                },
                "sound": "default"
            }
        });

        let resp = self
            .client
            .post(&url)
            .header("apns-topic", &self.config.bundle_id)
            .header("Content-Type", "application/json")
            // In production, generate a proper JWT with the .p8 key
            .header("authorization", "bearer placeholder-apns-jwt")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "APNs error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "apns"
    }
}

// ── WebPush Channel ───────────────────────────────────────────

/// WebPush notification channel.
pub struct WebPushChannel {
    config: WebPushChannelConfig,
    client: reqwest::Client,
}

impl WebPushChannel {
    pub fn new(config: WebPushChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for WebPushChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        // The recipient should be the WebPush subscription endpoint URL
        let payload = serde_json::json!({
            "title": message.subject,
            "body": message.body
        });

        let resp = self
            .client
            .post(&message.recipient)
            .header("Content-Type", "application/json")
            .header("Content-Encoding", "aes128gcm")
            .header("TTL", "60")
            // VAPID authorization header would be generated here
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    generate_webpush_vapid_token(
                        &self.config.vapid_private_key,
                        &self.config.vapid_subject,
                    )
                ),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 201 {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "WebPush error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "webpush"
    }
}

/// Generate a VAPID JWT for WebPush authentication.
fn generate_webpush_vapid_token(_private_key: &str, _subject: &str) -> String {
    // In production, use a proper ECDH/ES256 implementation
    "placeholder-vapid-token".to_string()
}

// ── Postmark Channel ──────────────────────────────────────────

/// Postmark email notification channel.
pub struct PostmarkChannel {
    config: PostmarkChannelConfig,
    client: reqwest::Client,
}

impl PostmarkChannel {
    pub fn new(config: PostmarkChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for PostmarkChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let payload = serde_json::json!({
            "From": self.config.from,
            "To": message.recipient,
            "Subject": message.subject,
            "HtmlBody": if message.is_html { &message.body } else { "" },
            "TextBody": if !message.is_html { &message.body } else { "" },
        });

        let resp = self
            .client
            .post("https://api.postmarkapp.com/email/withTemplate")
            .header("X-Postmark-Server-Token", &self.config.api_token)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "Postmark error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "postmark"
    }
}

// ── Vonage Channel ────────────────────────────────────────────

/// Vonage (Nexmo) SMS notification channel.
pub struct VonageChannel {
    config: VonageChannelConfig,
    client: reqwest::Client,
}

impl VonageChannel {
    pub fn new(config: VonageChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for VonageChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let body_str = format!("{}\n\n{}", message.subject, message.body);

        let form_body = format!(
            "api_key={}&api_secret={}&from={}&to={}&text={}",
            urlencoding::encode(&self.config.api_key),
            urlencoding::encode(&self.config.api_secret),
            urlencoding::encode(&self.config.from),
            urlencoding::encode(&message.recipient),
            urlencoding::encode(&body_str)
        );

        let resp = self
            .client
            .post("https://rest.nexmo.com/sms/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "Vonage error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "vonage"
    }
}

// ── Plivo Channel ─────────────────────────────────────────────

/// Plivo SMS notification channel.
pub struct PlivoChannel {
    config: PlivoChannelConfig,
    client: reqwest::Client,
}

impl PlivoChannel {
    pub fn new(config: PlivoChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for PlivoChannel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let body_str = format!("{}\n\n{}", message.subject, message.body);

        let url = format!(
            "https://api.plivo.com/v1/Account/{}/Message/",
            self.config.auth_id
        );

        let form_body = format!(
            "src={}&dst={}&text={}",
            urlencoding::encode(&self.config.from),
            urlencoding::encode(&message.recipient),
            urlencoding::encode(&body_str)
        );

        let resp = self
            .client
            .post(&url)
            .basic_auth(&self.config.auth_id, Some(&self.config.auth_token))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 202 {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(ChannelError::Api(format!(
                "Plivo error: {} - {}",
                status, text
            )))
        }
    }

    fn name(&self) -> &'static str {
        "plivo"
    }
}

// ── SMS77 Channel ─────────────────────────────────────────────

/// SMS77 SMS notification channel.
pub struct Sms77Channel {
    config: Sms77ChannelConfig,
    client: reqwest::Client,
}

impl Sms77Channel {
    pub fn new(config: Sms77ChannelConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for Sms77Channel {
    async fn send(&self, message: ChannelMessage) -> Result<(), ChannelError> {
        let body_str = format!("{}\n\n{}", message.subject, message.body);

        let form_body = format!(
            "user={}&from={}&to={}&text={}",
            urlencoding::encode(&self.config.api_key),
            urlencoding::encode(&self.config.from),
            urlencoding::encode(&message.recipient),
            urlencoding::encode(&body_str)
        );

        let resp = self
            .client
            .post("https://gap.sms77.io")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
            .map_err(|e| ChannelError::Network(e.to_string()))?;

        let text = resp.text().await.unwrap_or_default();
        // SMS77 returns "100 DELIV" on success
        if text.starts_with("100") {
            Ok(())
        } else {
            Err(ChannelError::Api(format!("SMS77 error: {}", text)))
        }
    }

    fn name(&self) -> &'static str {
        "sms77"
    }
}
