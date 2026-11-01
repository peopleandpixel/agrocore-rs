//! Inbound webhook handlers for receiving messages from external services.
//!
//! Supports:
//! - WhatsApp Business API webhooks
//! - Telegram Bot API webhooks
//! - E-Mail reply parsing (via webhook from mail service)

use serde::{Deserialize, Serialize};

// ── WhatsApp Business API ────────────────────────────────────────

/// WhatsApp webhook verification request.
#[derive(Debug, Deserialize)]
pub struct WhatsAppVerification {
    pub hub_mode: String,
    pub hub_verify_token: String,
    pub hub_challenge: String,
}

/// WhatsApp webhook response.
#[derive(Debug, Serialize)]
pub struct WhatsAppVerificationResponse {
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: String,
}

/// WhatsApp inbound message webhook payload.
#[derive(Debug, Deserialize)]
pub struct WhatsAppWebhook {
    pub object: String,
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppEntry {
    pub id: String,
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppChange {
    pub value: WhatsAppValue,
    pub field: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    pub metadata: WhatsAppMetadata,
    #[serde(default)]
    pub contacts: Vec<WhatsAppContact>,
    #[serde(default)]
    pub messages: Vec<WhatsAppMessage>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppContact {
    pub input: WhatsAppInput,
    pub profile: WhatsAppProfile,
    pub wa_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppInput {
    pub phone_number: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    pub text: Option<WhatsAppText>,
    pub image: Option<WhatsAppMedia>,
    pub audio: Option<WhatsAppMedia>,
    pub document: Option<WhatsAppMedia>,
    pub video: Option<WhatsAppMedia>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMedia {
    pub id: String,
    pub mime_type: String,
    pub sha256: String,
    pub filename: Option<String>,
}

// ── Telegram Bot API ─────────────────────────────────────────────

/// Telegram webhook update payload.
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub edited_message: Option<TelegramMessage>,
    pub channel_post: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub date: i64,
    pub text: Option<String>,
    pub entities: Option<Vec<TelegramEntity>>,
    pub photo: Option<Vec<TelegramPhotoSize>>,
    pub document: Option<TelegramDocument>,
    pub reply_to_message: Option<Box<TelegramMessage>>,
    pub forward_from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramChat {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub r#type: String, // "private", "group", "supergroup", "channel"
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramEntity {
    pub offset: i64,
    pub length: i64,
    pub r#type: String,
    pub url: Option<String>,
    pub user: Option<TelegramUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramPhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i64,
    pub height: i64,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramDocument {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

// ── E-Mail Reply Webhook ─────────────────────────────────────────

/// Generic email reply webhook payload.
/// Supports Mailgun, SendGrid, and custom formats.
#[derive(Debug, Deserialize, Serialize)]
pub struct EmailReplyWebhook {
    /// From address
    #[serde(default)]
    pub from: String,
    /// To address
    #[serde(default)]
    pub to: String,
    /// Subject line
    #[serde(default)]
    pub subject: String,
    /// Plain text body
    #[serde(default)]
    pub text: String,
    /// HTML body
    #[serde(default)]
    pub html: String,
    /// Message ID
    #[serde(default)]
    pub message_id: String,
    /// In-reply-to reference
    #[serde(default)]
    pub in_reply_to: String,
    /// References chain
    #[serde(default)]
    pub references: String,
    /// Timestamp
    #[serde(default)]
    pub timestamp: String,
    /// Attachments
    #[serde(default)]
    pub attachments: Vec<EmailAttachment>,
    /// Provider-specific raw data
    #[serde(default, flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmailAttachment {
    pub content_id: String,
    pub name: String,
    pub content_type: String,
    pub content: Option<String>, // base64 encoded
    pub content_length: i64,
}

// ── Unified Inbound Message ──────────────────────────────────────

/// Normalized inbound message from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    /// Source channel type
    pub source: InboundSource,
    /// Sender identifier (phone number, user ID, email)
    pub sender: String,
    /// Sender display name
    pub sender_name: String,
    /// Message text content
    pub text: String,
    /// Optional media URL or ID
    pub media: Option<String>,
    /// Optional media MIME type
    pub media_type: Option<String>,
    /// Original message ID from the source
    pub message_id: String,
    /// Timestamp (Unix epoch seconds)
    pub timestamp: i64,
    /// Raw payload for processing
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InboundSource {
    WhatsApp,
    Telegram,
    Email,
}

impl InboundMessage {
    /// Create from WhatsApp webhook.
    pub fn from_whatsapp(msg: &WhatsAppMessage, contact: &WhatsAppContact) -> Self {
        let text = msg
            .text
            .as_ref()
            .map(|t| t.body.clone())
            .unwrap_or_default();

        let media = msg
            .image
            .as_ref()
            .map(|m| m.id.clone())
            .or_else(|| msg.audio.as_ref().map(|m| m.id.clone()))
            .or_else(|| msg.document.as_ref().map(|m| m.id.clone()))
            .or_else(|| msg.video.as_ref().map(|m| m.id.clone()));

        let media_type = msg
            .image
            .as_ref()
            .map(|m| m.mime_type.clone())
            .or_else(|| msg.audio.as_ref().map(|m| m.mime_type.clone()))
            .or_else(|| msg.document.as_ref().map(|m| m.mime_type.clone()))
            .or_else(|| msg.video.as_ref().map(|m| m.mime_type.clone()));

        Self {
            source: InboundSource::WhatsApp,
            sender: msg.from.clone(),
            sender_name: contact.profile.name.clone(),
            text,
            media,
            media_type,
            message_id: msg.id.clone(),
            timestamp: timestamp_from_str(&msg.timestamp),
            raw: serde_json::to_value(msg).unwrap_or_default(),
        }
    }

    /// Create from Telegram update.
    pub fn from_telegram(update: &TelegramUpdate) -> Option<Self> {
        let msg = update.message.as_ref()?;
        let sender = msg.from.as_ref()?.id.to_string();
        let sender_name = msg
            .from
            .as_ref()
            .map(|u| u.username.clone().unwrap_or_else(|| u.first_name.clone()))
            .unwrap_or_default();

        let text = msg.text.clone().unwrap_or_default();
        let media = msg
            .photo
            .as_ref()
            .and_then(|p| p.first().map(|ps| ps.file_id.clone()))
            .or_else(|| msg.document.as_ref().map(|d| d.file_id.clone()));
        let media_type = msg.document.as_ref().and_then(|d| d.mime_type.clone());

        Some(Self {
            source: InboundSource::Telegram,
            sender,
            sender_name,
            text,
            media,
            media_type,
            message_id: msg.message_id.to_string(),
            timestamp: msg.date,
            raw: serde_json::to_value(update).unwrap_or_default(),
        })
    }

    /// Create from email reply webhook.
    pub fn from_email(email: &EmailReplyWebhook) -> Self {
        let text = if email.text.is_empty() {
            email.html.clone()
        } else {
            email.text.clone()
        };

        Self {
            source: InboundSource::Email,
            sender: email.from.clone(),
            sender_name: email.from.clone(),
            text,
            media: None,
            media_type: None,
            message_id: email.message_id.clone(),
            timestamp: timestamp_from_str(&email.timestamp),
            raw: serde_json::to_value(email).unwrap_or_default(),
        }
    }
}

fn timestamp_from_str(ts: &str) -> i64 {
    // Try parsing as Unix timestamp
    if let Ok(unix) = ts.parse::<i64>() {
        return unix;
    }
    // Try parsing as ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        return dt.timestamp();
    }
    0
}
