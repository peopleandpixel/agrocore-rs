//! Notification dispatcher tests.
//!
//! Covers config validation and channel construction — the parts that decide
//! whether a notification can be delivered at all. Delivery against live
//! providers belongs in integration tests.

use agrocore_messaging::notification::{
    ChannelConfig, NotificationConfig, NtfyChannelConfig, TelegramChannelConfig,
    WebhookChannelConfig, create_channel,
};

fn telegram_config() -> TelegramChannelConfig {
    TelegramChannelConfig {
        bot_token: "123456:test-token".to_string(),
        parse_mode: Some("HTML".to_string()),
    }
}

#[test]
fn test_create_channel_returns_matching_name() {
    let channel = create_channel(&ChannelConfig::Telegram(telegram_config()))
        .expect("telegram channel should be constructible");
    assert_eq!(channel.name(), "telegram");
}

#[test]
fn test_create_channel_supports_every_variant() {
    let configs = vec![
        (ChannelConfig::Telegram(telegram_config()), "telegram"),
        (
            ChannelConfig::Ntfy(NtfyChannelConfig {
                server: "https://ntfy.sh".to_string(),
                default_topic: "agrocore".to_string(),
                username: None,
                password: None,
            }),
            "ntfy",
        ),
        (
            ChannelConfig::Webhook(WebhookChannelConfig {
                url: "https://example.com/hook".to_string(),
                default_url: String::new(),
                secret: None,
                timeout_secs: 10,
            }),
            "webhook",
        ),
    ];

    for (config, expected_name) in configs {
        let channel = create_channel(&config).expect("channel should build");
        assert_eq!(channel.name(), expected_name);
        assert_eq!(config.channel_name(), expected_name);
    }
}

#[test]
fn test_validate_rejects_empty_telegram_token() {
    let config = ChannelConfig::Telegram(TelegramChannelConfig {
        bot_token: "   ".to_string(),
        parse_mode: None,
    });
    let err = config
        .validate()
        .expect_err("blank bot token must be rejected");
    assert!(
        err.to_string().contains("bot_token"),
        "error should name the offending field, got: {err}"
    );
}

#[test]
fn test_validate_rejects_empty_ntfy_topic() {
    let config = ChannelConfig::Ntfy(NtfyChannelConfig {
        server: "https://ntfy.sh".to_string(),
        default_topic: String::new(),
        username: None,
        password: None,
    });
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_non_http_webhook_url() {
    let config = ChannelConfig::Webhook(WebhookChannelConfig {
        url: "ftp://example.com/hook".to_string(),
        default_url: String::new(),
        secret: None,
        timeout_secs: 10,
    });
    let err = config
        .validate()
        .expect_err("non-http webhook url must be rejected");
    assert!(err.to_string().contains("http"), "got: {err}");
}

#[test]
fn test_validate_accepts_well_formed_config() {
    let config = ChannelConfig::Telegram(telegram_config());
    assert!(
        config.validate().is_ok(),
        "well-formed telegram config should validate: {:?}",
        config.validate()
    );
}

#[test]
fn test_channel_config_roundtrips_through_yaml() {
    let yaml = r#"
type: telegram
bot_token: abc123
parse_mode: HTML
"#;
    let config: ChannelConfig =
        serde_yaml::from_str(yaml).expect("channel config should deserialize from YAML");
    match config {
        ChannelConfig::Telegram(cfg) => {
            assert_eq!(cfg.bot_token, "abc123");
            assert_eq!(cfg.parse_mode.as_deref(), Some("HTML"));
        }
        other => panic!("expected Telegram variant, got {other:?}"),
    }
}

#[test]
fn test_notification_config_defaults_to_disabled_when_unset() {
    let config = NotificationConfig::default();
    assert!(!config.enabled, "notifications must be off by default");
    assert!(
        config.channels.is_empty(),
        "no channel should be configured by default"
    );
}

#[test]
fn test_dead_letter_subject_is_configurable() {
    let config = NotificationConfig {
        dead_letter_subject: "notifications.dead".to_string(),
        ..Default::default()
    };
    assert_eq!(config.dead_letter_subject, "notifications.dead");
}

/// Channels without a remote probe report available without any I/O.
/// Telegram overrides `is_available` with a real Bot API probe, so it is
/// deliberately excluded here.
#[tokio::test]
async fn test_is_available_defaults_to_true_without_remote_probe() {
    let channel = create_channel(&ChannelConfig::Webhook(WebhookChannelConfig {
        url: "https://example.com/hook".to_string(),
        default_url: String::new(),
        secret: None,
        timeout_secs: 10,
    }))
    .expect("channel should build");
    assert!(
        channel.is_available().await,
        "channels without a remote probe report available"
    );
}
