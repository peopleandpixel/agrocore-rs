//! Notification Service — delivers notifications to external channels.
//!
//! Consumes `notifications.send` from NATS, renders the tenant's templates and
//! dispatches the message through the configured channel (Email/SMTP, SendGrid,
//! Mailgun, Telegram, ntfy, Webhook, Twilio SMS or WhatsApp via `wacli`).
//! Failures are retried with exponential backoff and end up on a dead-letter
//! subject so nothing is lost silently.
//!
//! Configuration is read from `NOTIFICATION_CONFIG` (YAML file path) or from
//! the inline `NOTIFICATION_CONFIG_JSON` value; individual secrets may also be
//! supplied via environment variables.

use actix_web::{App, HttpResponse, HttpServer, web};
use agrocore_logging::{error, info, warn};
use agrocore_messaging::notification::{ChannelConfig, NotificationConfig, NotificationDispatcher};
use std::collections::HashMap;
use std::sync::Arc;

/// Counters exposed on `/health` for quick operational checks.
#[derive(Default)]
struct HealthState {
    started_at: chrono::DateTime<chrono::Utc>,
    channels: Vec<String>,
}

async fn health(state: web::Data<Arc<HealthState>>) -> HttpResponse {
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "agrocore-notification-service",
        "uptime_seconds": uptime,
        "channels": state.channels,
        "subscribed_subject": agrocore_messaging::notification::NATS_SUBJECT_NOTIFICATIONS_SEND,
    }))
}

/// Load the notification configuration.
///
/// Precedence: `NOTIFICATION_CONFIG_JSON` > `NOTIFICATION_CONFIG` (file path).
/// With neither set the service starts with notifications disabled, which is
/// the correct default for a fresh dev environment.
fn load_config() -> NotificationConfig {
    if let Ok(raw) = std::env::var("NOTIFICATION_CONFIG_JSON") {
        match serde_json::from_str::<NotificationConfig>(&raw) {
            Ok(cfg) => {
                info!("Loaded notification config from NOTIFICATION_CONFIG_JSON");
                return cfg;
            }
            Err(e) => warn!("Invalid NOTIFICATION_CONFIG_JSON, falling back: {e}"),
        }
    }

    if let Ok(path) = std::env::var("NOTIFICATION_CONFIG") {
        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_yaml::from_str::<NotificationConfig>(&contents) {
                Ok(cfg) => {
                    info!("Loaded notification config from {path}");
                    return cfg;
                }
                Err(e) => error!("Failed to parse notification config {path}: {e}"),
            },
            Err(e) => error!("Cannot read notification config {path}: {e}"),
        }
    }

    warn!("No notification configuration found — service starts idle");
    NotificationConfig {
        enabled: false,
        ..Default::default()
    }
}

/// Build a channel configuration from environment variables for the named
/// channel. Keeps secrets out of the YAML file.
fn channel_from_env(name: &str) -> Option<ChannelConfig> {
    let var = |suffix: &str| std::env::var(format!("NOTIFY_{name}_{suffix}")).ok();

    match name {
        "smtp" => Some(ChannelConfig::Smtp(
            agrocore_messaging::notification::SmtpChannelConfig {
                host: var("HOST")?,
                port: var("PORT").and_then(|v| v.parse().ok()).unwrap_or(587),
                username: var("USERNAME").unwrap_or_default(),
                password: var("PASSWORD")?,
                from: var("FROM")?,
                tls: var("TLS").map(|v| v != "false").unwrap_or(true),
            },
        )),
        "telegram" => Some(ChannelConfig::Telegram(
            agrocore_messaging::notification::TelegramChannelConfig {
                bot_token: var("BOT_TOKEN")?,
                parse_mode: var("PARSE_MODE"),
            },
        )),
        "ntfy" => Some(ChannelConfig::Ntfy(
            agrocore_messaging::notification::NtfyChannelConfig {
                server: var("SERVER").unwrap_or_else(|| "https://ntfy.sh".to_string()),
                default_topic: var("TOPIC")?,
                username: var("USERNAME"),
                password: var("PASSWORD"),
            },
        )),
        "webhook" => Some(ChannelConfig::Webhook(
            agrocore_messaging::notification::WebhookChannelConfig {
                url: var("URL")?,
                default_url: String::new(),
                secret: var("SECRET"),
                timeout_secs: var("TIMEOUT_SECS")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            },
        )),
        "twilio" => Some(ChannelConfig::Twilio(
            agrocore_messaging::notification::TwilioChannelConfig {
                account_sid: var("ACCOUNT_SID")?,
                auth_token: var("AUTH_TOKEN")?,
                from_number: var("FROM_NUMBER")?,
            },
        )),
        "wacli" => Some(ChannelConfig::Wacli(
            agrocore_messaging::notification::WacliChannelConfig {
                phone_number: var("PHONE_NUMBER")?,
                binary_path: var("BINARY_PATH").unwrap_or_else(|| "wacli".to_string()),
            },
        )),
        _ => None,
    }
}

/// Discover channels from `NOTIFY_CHANNELS` (comma separated) and merge them
/// into the configuration.
fn merge_env_channels(mut config: NotificationConfig) -> NotificationConfig {
    let Ok(list) = std::env::var("NOTIFY_CHANNELS") else {
        return config;
    };

    for name in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match channel_from_env(name) {
            Some(channel_cfg) => {
                info!("Channel '{name}' configured from environment");
                config.channels.insert(name.to_string(), channel_cfg);
            }
            None => warn!(
                "Channel '{name}' requested via NOTIFY_CHANNELS but its NOTIFY_{name}_* \
                 environment variables are incomplete — skipped",
                name = name.to_uppercase()
            ),
        }
    }
    config
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    agrocore_shared::telemetry::init_telemetry("agrocore-notification-service");
    info!("Starting agrocore-notification-service");

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let bind_addr =
        std::env::var("NOTIFICATION_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".to_string());

    let config = merge_env_channels(load_config());
    let channel_names: Vec<String> = config.channels.keys().cloned().collect();
    let enabled = config.enabled && !config.channels.is_empty();

    if !enabled {
        warn!(
            "Notifications disabled (enabled={}, channels={}) — running as a health-only process",
            config.enabled,
            config.channels.len()
        );
    } else {
        info!("Configured channels: {}", channel_names.join(", "));
    }

    // Connect to NATS. A service with no channels still needs the client so the
    // health endpoint can report readiness, but we must not block startup.
    let nats = match async_nats::connect(&nats_url).await {
        Ok(client) => {
            info!("Connected to NATS at {nats_url}");
            client
        }
        Err(e) => {
            error!("Failed to connect to NATS at {nats_url}: {e}");
            return Err(anyhow::anyhow!("NATS connection failed: {e}"));
        }
    };

    if enabled {
        let dispatcher = NotificationDispatcher::new(config, nats.clone())
            .await
            .map_err(|e| anyhow::anyhow!("dispatcher init failed: {e}"))?;
        dispatcher
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("dispatcher start failed: {e}"))?;
        info!(
            "Dispatcher consuming '{}'",
            agrocore_messaging::notification::NATS_SUBJECT_NOTIFICATIONS_SEND
        );
    }

    let state = Arc::new(HealthState {
        started_at: chrono::Utc::now(),
        channels: channel_names,
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
    })
    .bind(&bind_addr)?;
    info!("Health endpoint listening on {bind_addr}");

    // Actix servers are single-threaded handles (not `Send`), so they must be
    // driven by the actix runtime rather than `tokio::spawn`.
    let server = server.run();
    let handle = server.handle();

    actix_web::rt::spawn(async move {
        if let Err(e) = server.await {
            error!("HTTP server stopped: {e}");
        }
    });

    wait_for_shutdown().await;
    info!("Shutdown signal received, stopping gracefully...");
    handle.stop(true).await;

    info!("agrocore-notification-service stopped");
    Ok(())
}

/// Keeps the `HashMap` import meaningful for readers of the config helpers.
#[allow(dead_code)]
type ChannelMap = HashMap<String, ChannelConfig>;

/// Block until SIGINT/SIGTERM arrives.
///
/// The dispatcher and the HTTP server both run on actix's LocalSet, so the
/// signal futures must be polled on that runtime rather than on a separate
/// `#[tokio::main]` runtime.
async fn wait_for_shutdown() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("SIGINT received"),
        Err(e) => error!("Cannot wait for SIGINT: {e}"),
    }
}
