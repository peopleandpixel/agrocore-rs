//! MQTT Bridge - Bidirectional NATS ↔ MQTT Message Forwarding
//!
//! This module provides a bridge between NATS and MQTT for IoT device communication.
//! Note: MQTT functionality requires optional MQTT client support.

use crate::{
    MessagingClient, MqttConfig, NATS_SUBJECT_COMMANDS_BRIDGE,
    NATS_SUBJECT_COMMANDS_BRIDGE_BROADCAST, NATS_SUBJECT_DEVICE_STATUS, NATS_SUBJECT_EVENTS,
    NATS_SUBJECT_TELEMETRY,
};
use agrocore_logging::{error, info};
use async_nats::Subject;
use futures_util::StreamExt;
use rumqttc::{Event as MqttEvent, MqttOptions, Packet, QoS};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};

// Custom serde implementation for QoS
mod qos_serde {
    use super::*;

    pub fn serialize<S>(qos: &QoS, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = match qos {
            QoS::AtMostOnce => 0u8,
            QoS::AtLeastOnce => 1u8,
            QoS::ExactlyOnce => 2u8,
        };
        serializer.serialize_u8(val)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<QoS, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u8::deserialize(deserializer)?;
        match val {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _ => Err(serde::de::Error::custom("Invalid QoS level")),
        }
    }
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// NATS subject patterns to forward to MQTT
    pub nats_to_mqtt: Vec<BridgeRoute>,
    /// MQTT topic patterns to forward to NATS
    pub mqtt_to_nats: Vec<BridgeRoute>,
    /// Reconnect delay
    pub reconnect_delay_ms: u64,
    /// Max reconnect attempts
    pub max_reconnect_attempts: u32,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            nats_to_mqtt: vec![
                BridgeRoute {
                    nats_subject: NATS_SUBJECT_EVENTS.to_string(),
                    mqtt_topic: "agrocore/events".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: NATS_SUBJECT_TELEMETRY.to_string(),
                    mqtt_topic: "agrocore/telemetry".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: NATS_SUBJECT_DEVICE_STATUS.to_string(),
                    mqtt_topic: "agrocore/status".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: true,
                },
            ],
            mqtt_to_nats: vec![
                BridgeRoute {
                    nats_subject: NATS_SUBJECT_COMMANDS_BRIDGE.to_string(),
                    mqtt_topic: "agrocore/commands/+/+".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: NATS_SUBJECT_COMMANDS_BRIDGE_BROADCAST.to_string(),
                    mqtt_topic: "agrocore/commands/+/broadcast".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
            ],
            reconnect_delay_ms: 5000,
            max_reconnect_attempts: 10,
        }
    }
}

/// Bridge route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeRoute {
    pub nats_subject: String,
    pub mqtt_topic: String,
    #[serde(with = "qos_serde")]
    pub qos: QoS,
    pub retain: bool,
}

/// Bridge statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    pub nats_to_mqtt_messages: u64,
    pub mqtt_to_nats_messages: u64,
    pub nats_errors: u64,
    pub mqtt_errors: u64,
    pub last_nats_to_mqtt: Option<chrono::DateTime<chrono::Utc>>,
    pub last_mqtt_to_nats: Option<chrono::DateTime<chrono::Utc>>,
    pub connected: bool,
    pub reconnect_count: u32,
}

/// MQTT Bridge for bidirectional NATS ↔ MQTT message forwarding
pub struct MqttBridge {
    nats_client: MessagingClient,
    config: BridgeConfig,
    stats: Arc<RwLock<BridgeStats>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    // MQTT client is optional - bridge can work NATS-only
    mqtt_client: Option<rumqttc::AsyncClient>,
    mqtt_event_loop: Option<rumqttc::EventLoop>,
}

impl MqttBridge {
    /// Create a new MQTT Bridge (NATS-only mode)
    pub async fn new(
        nats_url: &str,
        mqtt_config: Option<MqttConfig>,
        bridge_config: BridgeConfig,
    ) -> anyhow::Result<Self> {
        let nats_client = MessagingClient::connect(nats_url).await?;

        // Try to connect MQTT if config provided
        let (mqtt_client, mqtt_event_loop) = if let Some(config) = mqtt_config {
            let mut mqtt_options = MqttOptions::new(
                &format!("agrocore-{}", uuid::Uuid::new_v4()),
                &config.broker_host,
                config.broker_port,
            );

            if let (Some(username), Some(password)) =
                (config.username.clone(), config.password.clone())
            {
                mqtt_options.set_credentials(username, password);
            }

            if config.use_tls {
                // TLS config would go here
            }

            let (client, event_loop) = rumqttc::AsyncClient::new(mqtt_options, 100);
            (Some(client), Some(event_loop))
        } else {
            (None, None)
        };

        Ok(Self {
            nats_client,
            config: bridge_config,
            stats: Arc::new(RwLock::new(BridgeStats::default())),
            shutdown_tx: None,
            mqtt_client,
            mqtt_event_loop,
        })
    }

    /// Create a new MQTT Bridge (NATS-only mode, no MQTT)
    pub async fn new_nats_only(
        nats_url: &str,
        bridge_config: BridgeConfig,
    ) -> anyhow::Result<Self> {
        Self::new(nats_url, None, bridge_config).await
    }

    /// Start the bridge
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start NATS to MQTT forwarding (if MQTT available)
        if self.mqtt_client.is_some() {
            self.start_nats_to_mqtt().await?;
            self.start_mqtt_to_nats().await?;
        }

        // Start stats reporter
        self.start_stats_reporter().await;

        info!(
            "MQTT Bridge started (NATS-only mode: {})",
            self.mqtt_client.is_none()
        );

        // Wait for shutdown signal
        shutdown_rx.recv().await;

        info!("MQTT Bridge shutting down");
        Ok(())
    }

    /// Start NATS to MQTT forwarding
    async fn start_nats_to_mqtt(&self) -> anyhow::Result<()> {
        let Some(mqtt_client) = &self.mqtt_client else {
            info!("NATS to MQTT forwarding disabled (no MQTT client)");
            return Ok(());
        };

        let nats = self.nats_client.nats();
        let mqtt = mqtt_client.clone();
        let stats = self.stats.clone();

        for route in &self.config.nats_to_mqtt {
            let nats_subject = route.nats_subject.clone();
            let mqtt_topic = route.mqtt_topic.clone();
            let qos = route.qos;
            let retain = route.retain;
            let stats = stats.clone();
            let mqtt = mqtt.clone();
            let nats_client = nats.clone();

            tokio::spawn(async move {
                let subject = nats_subject.clone();
                let subscriber = match nats_client.subscribe(subject).await {
                    Ok(sub) => sub,
                    Err(e) => {
                        error!(
                            "Failed to subscribe to NATS subject {}: {}",
                            nats_subject, e
                        );
                        return;
                    }
                };

                info!(
                    "Subscribed to NATS subject: {} -> MQTT topic: {}",
                    nats_subject, mqtt_topic
                );

                let mut subscriber = subscriber;
                while let Some(msg) = subscriber.next().await {
                    let payload = msg.payload.to_vec();

                    // Forward to MQTT
                    match mqtt.publish(mqtt_topic.clone(), qos, retain, payload).await {
                        Ok(_) => {
                            let mut stats = stats.write().await;
                            stats.nats_to_mqtt_messages += 1;
                            stats.last_nats_to_mqtt = Some(chrono::Utc::now());
                        }
                        Err(e) => {
                            error!("Failed to forward NATS message to MQTT: {}", e);
                            let mut stats = stats.write().await;
                            stats.nats_errors += 1;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Start MQTT to NATS forwarding
    async fn start_mqtt_to_nats(&mut self) -> anyhow::Result<()> {
        let Some(mqtt_event_loop) = self.mqtt_event_loop.take() else {
            info!("MQTT to NATS forwarding disabled (no MQTT event loop)");
            return Ok(());
        };

        let Some(mqtt_client) = &self.mqtt_client else {
            return Ok(());
        };

        let nats = self.nats_client.nats().clone();
        let stats = self.stats.clone();
        let routes = self.config.mqtt_to_nats.clone();

        // Subscribe to MQTT topics
        for route in &self.config.mqtt_to_nats {
            mqtt_client
                .subscribe(route.mqtt_topic.as_str(), route.qos)
                .await?;
        }

        tokio::spawn(async move {
            info!("Starting MQTT to NATS event processor");

            let mut event_loop = mqtt_event_loop;
            loop {
                match event_loop.poll().await {
                    Ok(event) => {
                        if let MqttEvent::Incoming(Packet::Publish(publish)) = event {
                            let topic = publish.topic;
                            let payload = publish.payload.to_vec();

                            // Find matching route
                            for route in &routes {
                                if Self::topic_matches(&topic, &route.mqtt_topic) {
                                    let subject = route.nats_subject.clone();

                                    match nats.publish(subject, payload.clone().into()).await {
                                        Ok(_) => {
                                            let mut stats = stats.write().await;
                                            stats.mqtt_to_nats_messages += 1;
                                            stats.last_mqtt_to_nats = Some(chrono::Utc::now());
                                        }
                                        Err(e) => {
                                            error!("Failed to forward MQTT message to NATS: {}", e);
                                            let mut stats = stats.write().await;
                                            stats.mqtt_errors += 1;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {}", e);
                        // Reconnect logic would go here
                        return;
                    }
                }
            }
        });

        Ok(())
    }

    /// Check if MQTT topic matches route pattern (supports wildcards)
    fn topic_matches(topic: &str, pattern: &str) -> bool {
        // Simple wildcard matching for + and #
        if pattern == "#" {
            return true;
        }

        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let topic_parts: Vec<&str> = topic.split('/').collect();

        if pattern_parts.len() != topic_parts.len() {
            return false;
        }

        for (p, t) in pattern_parts.iter().zip(topic_parts.iter()) {
            if *p == "+" || *p == "#" {
                continue;
            }
            if *p != *t {
                return false;
            }
        }

        true
    }

    /// Start periodic stats reporting
    async fn start_stats_reporter(&self) {
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let stats = stats.read().await;
                info!(
                    "Bridge Stats - NATS→MQTT: {}, MQTT→NATS: {}, NATS Errors: {}, MQTT Errors: {}, Connected: {}",
                    stats.nats_to_mqtt_messages,
                    stats.mqtt_to_nats_messages,
                    stats.nats_errors,
                    stats.mqtt_errors,
                    stats.connected
                );
            }
        });
    }

    /// Get bridge statistics
    pub async fn get_stats(&self) -> BridgeStats {
        self.stats.read().await.clone()
    }

    /// Shutdown the bridge
    pub async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

/// Builder for MqttBridge
pub struct MqttBridgeBuilder {
    nats_url: Option<String>,
    mqtt_config: Option<MqttConfig>,
    bridge_config: BridgeConfig,
}

impl MqttBridgeBuilder {
    pub fn new() -> Self {
        Self {
            nats_url: None,
            mqtt_config: None,
            bridge_config: BridgeConfig::default(),
        }
    }

    pub fn nats_url(mut self, url: &str) -> Self {
        self.nats_url = Some(url.to_string());
        self
    }

    pub fn mqtt_config(mut self, config: MqttConfig) -> Self {
        self.mqtt_config = Some(config);
        self
    }

    pub fn bridge_config(mut self, config: BridgeConfig) -> Self {
        self.bridge_config = config;
        self
    }

    pub fn add_nats_to_mqtt_route(
        mut self,
        nats_subject: &str,
        mqtt_topic: &str,
        qos: QoS,
        retain: bool,
    ) -> Self {
        self.bridge_config.nats_to_mqtt.push(BridgeRoute {
            nats_subject: nats_subject.to_string(),
            mqtt_topic: mqtt_topic.to_string(),
            qos,
            retain,
        });
        self
    }

    pub fn add_mqtt_to_nats_route(
        mut self,
        mqtt_topic: &str,
        nats_subject: &str,
        qos: QoS,
        retain: bool,
    ) -> Self {
        self.bridge_config.mqtt_to_nats.push(BridgeRoute {
            nats_subject: nats_subject.to_string(),
            mqtt_topic: mqtt_topic.to_string(),
            qos,
            retain,
        });
        self
    }

    pub async fn build(self) -> anyhow::Result<MqttBridge> {
        let nats_url = self
            .nats_url
            .ok_or_else(|| anyhow::anyhow!("NATS URL required"))?;

        MqttBridge::new(&nats_url, self.mqtt_config, self.bridge_config).await
    }

    pub async fn build_nats_only(self) -> anyhow::Result<MqttBridge> {
        let nats_url = self
            .nats_url
            .ok_or_else(|| anyhow::anyhow!("NATS URL required"))?;

        MqttBridge::new_nats_only(&nats_url, self.bridge_config).await
    }
}

impl Default for MqttBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
