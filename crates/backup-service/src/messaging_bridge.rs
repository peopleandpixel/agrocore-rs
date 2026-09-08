//! AgroCore Messaging Bridge Binary
//!
//! This binary runs the MQTT Bridge with scheduled stats reporting via agrocore-scheduler.

use agrocore_backup::config::load_config;
use agrocore_backup::error::{BackupError, BackupResult};
use agrocore_backup::nats_client::NatsClient;
use agrocore_logging::{EnvironmentType, LoggingConfig, info, init_logging};
use agrocore_messaging::{BridgeConfig, MqttBridge, MqttBridgeBuilder};
use agrocore_scheduler::{JobDefinition, JobType, SchedulerConfig, SchedulerService};
use agrocore_shared::config::AgroCoreConfig;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> BackupResult<()> {
    // Initialize logging via agrocore-logging
    let logging_config = LoggingConfig {
        level: "info".to_string(),
        console_enabled: true,
        console_pretty: true,
        console_thread_ids: true,
        console_thread_names: true,
        environment: EnvironmentType::Development,
        ..Default::default()
    };
    init_logging(logging_config)
        .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Logging init failed: {}", e)))?;

    info!("Starting agrocore-messaging-bridge service v{}", env!("CARGO_PKG_VERSION"));

    // Load AgroCore config first (for DB connection, NATS URL, etc.)
    let agro_config = AgroCoreConfig::global();

    // Load backup-specific config (for MQTT bridge config if needed)
    let _backup_config = load_config()?;

    // Connect to NATS
    let nats_url = &agro_config.nats_url;
    info!("Connecting to NATS at {}", nats_url);
    let nats_client = NatsClient::connect(nats_url).await?;
    info!("NATS connected successfully");

    // Create MQTT Bridge (NATS-only mode for now)
    let bridge_config = BridgeConfig::default();
    let bridge = MqttBridgeBuilder::new()
        .nats_url(nats_url)
        .bridge_config(bridge_config)
        .build_nats_only()
        .await
        .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Failed to create MQTT Bridge: {}", e)))?;

    // Create Scheduler
    let scheduler_config = SchedulerConfig {
        enabled: true,
        timezone: "UTC".to_string(),
        default_job_timeout_seconds: 3600,
        max_concurrent_jobs: 10,
        retry_failed_jobs: true,
        max_retries: 3,
        retry_delay_seconds: 60,
    };

    let scheduler = Arc::new(
        SchedulerService::new(scheduler_config, Some(nats_client.inner().clone()))
            .await
            .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Failed to create scheduler: {}", e)))?,
    );

    // Register bridge scheduler jobs (stats reporter) - done externally to avoid cyclic dependency
    let stats = bridge.stats.clone();
    
    // Register handler for stats reporting
    scheduler
        .register_handler("bridge_stats_reporter", move |_job_def| {
            let stats = stats.clone();
            Box::pin(async move {
                let stats = stats.read().await;
                agrocore_logging::info!(
                    "Bridge Stats - NATS→MQTT: {}, MQTT→NATS: {}, NATS Errors: {}, MQTT Errors: {}, Connected: {}",
                    stats.nats_to_mqtt_messages,
                    stats.mqtt_to_nats_messages,
                    stats.nats_errors,
                    stats.mqtt_errors,
                    stats.connected
                );
                Ok(())
            })
        })
        .await;

    // Add the recurring stats reporter job (every 60 seconds)
    let job_def = JobDefinition {
        id: "bridge_stats_reporter".to_string(),
        name: "MQTT Bridge Stats Reporter".to_string(),
        description: "Periodic stats reporting for MQTT Bridge".to_string(),
        schedule: "* * * * * *".to_string(), // Every 60 seconds (every minute)
        timezone: Some("UTC".to_string()),
        job_type: JobType::Builtin {
            handler: "bridge_stats_reporter".to_string(),
        },
        payload: serde_json::json!({}),
        timeout_seconds: Some(30),
        max_retries: Some(3),
        retry_delay_seconds: Some(10),
        enabled: true,
        tags: vec!["messaging".to_string(), "bridge".to_string(), "stats".to_string()],
    };
    
    scheduler.add_job(job_def).await
        .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Failed to add bridge stats job: {}", e)))?;

    // Start the scheduler
    scheduler
        .start()
        .await
        .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Failed to start scheduler: {}", e)))?;

    info!("Scheduler started with bridge stats reporter job");

    // Start the bridge
    tokio::spawn(async move {
        let mut bridge = bridge;
        if let Err(e) = bridge.start().await {
            agrocore_logging::error!("MQTT Bridge error: {}", e);
        }
    });

    info!("MQTT Bridge started");

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping gracefully...");
        }
        Err(e) => {
            agrocore_logging::error!("Failed to listen for shutdown signal: {}", e);
        }
    }

    // Stop scheduler
    if let Err(e) = scheduler.stop().await {
        agrocore_logging::error!("Failed to stop scheduler: {}", e);
    }
    info!("Scheduler stopped");

    // Close NATS connection
    nats_client.close().await;
    info!("NATS connection closed");

    info!("Messaging bridge service stopped gracefully");
    Ok(())
}