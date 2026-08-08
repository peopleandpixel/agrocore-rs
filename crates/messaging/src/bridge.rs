//! MQTT Bridge - Bidirectional NATS ↔ MQTT Message Forwarding

use crate::{MqttClient, MqttConfig, MessagingClient, UnifiedMessagingClient, Event, IoTTelemetryEvent, IoTDeviceStatusEvent};
use async_nats::Client as NatsClient;
use rumqttc::{AsyncClient, Event as MqttEvent, EventLoop, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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
                    nats_subject: "events.>".to_string(),
                    mqtt_topic: "agrocore/events".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: "telemetry.>".to_string(),
                    mqtt_topic: "agrocore/telemetry".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: "device.status.>".to_string(),
                    mqtt_topic: "agrocore/status".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: true,
                },
            ],
            mqtt_to_nats: vec![
                BridgeRoute {
                    nats_subject: "commands.bridge".to_string(),
                    mqtt_topic: "agrocore/commands/+/+".to_string(),
                    qos: QoS::AtLeastOnce,
                    retain: false,
                },
                BridgeRoute {
                    nats_subject: "commands.bridge.broadcast".to_string(),
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
    unified_client: UnifiedMessagingClient,
    config: BridgeConfig,
    stats: Arc<RwLock<BridgeStats>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl MqttBridge {
    /// Create a new MQTT Bridge
    pub async fn new(
        nats_url: &str,
        mqtt_config: MqttConfig,
        bridge_config: BridgeConfig,
    ) -> anyhow::Result<Self> {
        let unified_client = UnifiedMessagingClient::new(Some(nats_url), Some(mqtt_config)).await?;
        
        Ok(Self {
            unified_client,
            config: bridge_config,
            stats: Arc::new(RwLock::new(BridgeStats::default())),
            shutdown_tx: None,
        })
    }

    /// Start the bridge
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start NATS to MQTT forwarding
        self.start_nats_to_mqtt().await?;
        
        // Start MQTT to NATS forwarding
        self.start_mqtt_to_nats().await?;

        // Start stats reporter
        self.start_stats_reporter().await;

        info!("MQTT Bridge started");

        // Wait for shutdown signal
        shutdown_rx.recv().await;
        
        info!("MQTT Bridge shutting down");
        Ok(())
    }

    /// Start NATS to MQTT forwarding
    async fn start_nats_to_mqtt(&self) -> anyhow::Result<()> {
        let nats = self.unified_client.nats()
            .ok_or_else(|| anyhow::anyhow!("NATS not configured"))?;
        
        let mqtt = self.unified_client.mqtt()
            .ok_or_else(|| anyhow::anyhow!("MQTT not configured"))?;

        for route in &self.config.nats_to_mqtt {
            let nats_subject = route.nats_subject.clone();
            let mqtt_topic = route.mqtt_topic.clone();
            let qos = route.qos;
            let retain = route.retain;
            let stats = self.stats.clone();
            let mqtt = mqtt.clone();

            tokio::spawn(async move {
                let subscriber = match nats.subscribe(nats_subject.clone()).await {
                    Ok(sub) => sub,
                    Err(e) => {
                        error!("Failed to subscribe to NATS subject {}: {}", nats_subject, e);
                        return;
                    }
                };

                info!("Subscribed to NATS subject: {} -> MQTT topic: {}", nats_subject, mqtt_topic);

                while let Some(msg) = subscriber.next().await {
                    let payload = msg.payload.to_vec();
                    
                    // Forward to MQTT
                    match mqtt.client.publish(mqtt_topic.clone(), qos, false, payload).await {
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
    async fn start_mqtt_to_nats(&self) -> anyhow::Result<()> {
        let nats = self.unified_client.nats()
            .ok_or_else(|| anyhow::anyhow!("NATS not configured"))?;
        
        let mqtt = self.unified_client.mqtt()
            .ok_or_else(|| anyhow::anyhow!("MQTT not configured"))?;

        // Subscribe to MQTT topics
        for route in &self.config.mqtt_to_nats {
            mqtt.subscribe_commands(route.mqtt_topic.clone(), route.qos).await?;
        }

        // Process MQTT events
        let mut event_loop = mqtt.event_loop.clone();
        let stats = self.stats.clone();
        let nats = nats.clone();
        let routes = self.config.mqtt_to_nats.clone();

        tokio::spawn(async move {
            info!("Starting MQTT to NATS event processor");

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
                                    
                                    match nats.publish(subject.clone(), payload.clone().into()).await {
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

    pub fn add_nats_to_mqtt_route(mut self, nats_subject: &str, mqtt_topic: &str, qos: QoS, retain: bool) -> Self {
        self.bridge_config.nats_to_mqtt.push(BridgeRoute {
            nats_subject: nats_subject.to_string(),
            mqtt_topic: mqtt_topic.to_string(),
            qos,
            retain,
        });
        self
    }

    pub fn add_mqtt_to_nats_route(mut self, mqtt_topic: &str, nats_subject: &str, qos: QoS, retain: bool) -> Self {
        self.bridge_config.mqtt_to_nats.push(BridgeRoute {
            nats_subject: nats_subject.to_string(),
            mqtt_topic: mqtt_topic.to_string(),
            qos,
            retain,
        });
        self
    }

    pub async fn build(self) -> anyhow::Result<MqttBridge> {
        let nats_url = self.nats_url.ok_or_else(|| anyhow::anyhow!("NATS URL required"))?;
        let mqtt_config = self.mqtt_config.ok_or_else(|| anyhow::anyhow!("MQTT config required"))?;
        
        MqttBridge::new(&nats_url, mqtt_config, self.bridge_config).await
    }
}

impl Default for MqttBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}