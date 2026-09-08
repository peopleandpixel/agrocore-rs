use crate::config::load_config;
use crate::error::{BackupError, BackupResult};
use crate::nats_client::NatsClient;
use crate::service::BackupService;
use agrocore_logging::{
    EnvironmentType, LoggingConfig, error as logging_error, info, init_logging, warn,
};
use agrocore_messaging::{BridgeConfig, MqttBridgeBuilder};
use agrocore_scheduler::{JobDefinition, JobType, SchedulerConfig, SchedulerService};
use agrocore_shared::config::AgroCoreConfig;
use std::sync::Arc;
use tokio::signal;

mod config;
mod encryption;
mod error;
mod manifest;
mod nats_client;
mod pg_dump;
mod retention;
mod service;
mod storage;
mod verification;

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

    agrocore_logging::info!(
        "Starting agrocore-backup service v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load AgroCore config first (for DB connection, NATS URL, etc.)
    let agro_config = AgroCoreConfig::global();

    // Load backup-specific config
    let backup_config = load_config()?;

    if !backup_config.enabled {
        warn!("Backup service is disabled via config. Exiting.");
        return Ok(());
    }

    // Validate config early
    backup_config.validate()?;

    // Connect to NATS
    let nats_url = &agro_config.nats_url;
    info!("Connecting to NATS at {}", nats_url);
    let mut nats_client = NatsClient::connect(nats_url).await?;
    info!("NATS connected successfully");

    // Create MQTT Bridge (NATS-only mode for now)
    let bridge_config = BridgeConfig::default();
    let mut bridge = MqttBridgeBuilder::new()
        .nats_url(nats_url)
        .bridge_config(bridge_config)
        .build_nats_only()
        .await
        .map_err(|e| BackupError::Anyhow(anyhow::anyhow!("Failed to create MQTT Bridge: {}", e)))?;

    // Create backup service
    let mut backup_service = BackupService::new(
        backup_config,
        agro_config.database_url.clone(),
        nats_client.clone(),
    )
    .await?;

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
            .map_err(|e| {
                BackupError::Anyhow(anyhow::anyhow!("Failed to create scheduler: {}", e))
            })?,
    );

    // Register builtin handlers for backup jobs
    let service = backup_service.clone();
    scheduler
        .register_handler("backup_database", move |job_def| {
            let service = service.clone();
            let backup_type = job_def
                .payload
                .get("backup_type")
                .and_then(|v| v.as_str())
                .unwrap_or("database");
            let bt = match backup_type {
                "config" => crate::service::BackupType::Config,
                _ => crate::service::BackupType::Database,
            };
            Box::pin(async move {
                if let Err(e) = service.run_backup(bt).await {
                    logging_error!("Scheduled backup failed: {}", e);
                }
                Ok(())
            })
        })
        .await;

    // Register handler for MQTT Bridge stats reporting
    let stats = bridge.stats.clone();
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

    // Start the scheduler
    scheduler.start().await?;

    // Add DB backup job
    let db_job = JobDefinition {
        id: "backup-database".to_string(),
        name: "Database Backup".to_string(),
        description: "Scheduled database backup".to_string(),
        schedule: backup_service.config().schedule_db.clone(),
        timezone: Some(backup_service.config().timezone.clone()),
        job_type: JobType::Builtin {
            handler: "backup_database".to_string(),
        },
        payload: serde_json::json!({ "backup_type": "database" }),
        timeout_seconds: Some(3600),
        max_retries: Some(3),
        retry_delay_seconds: Some(60),
        enabled: backup_service.config().enabled,
        tags: vec!["backup".to_string(), "database".to_string()],
    };
    scheduler.add_job(db_job).await?;

    // Add Config backup job
    let config_job = JobDefinition {
        id: "backup-config".to_string(),
        name: "Config Backup".to_string(),
        description: "Scheduled config backup".to_string(),
        schedule: backup_service.config().schedule_config.clone(),
        timezone: Some(backup_service.config().timezone.clone()),
        job_type: JobType::Builtin {
            handler: "backup_database".to_string(),
        },
        payload: serde_json::json!({ "backup_type": "config" }),
        timeout_seconds: Some(3600),
        max_retries: Some(3),
        retry_delay_seconds: Some(60),
        enabled: backup_service.config().enabled,
        tags: vec!["backup".to_string(), "config".to_string()],
    };
    scheduler.add_job(config_job).await?;

    // Add MQTT Bridge stats reporter job (every 60 seconds)
    let bridge_stats_job = JobDefinition {
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
        tags: vec![
            "messaging".to_string(),
            "bridge".to_string(),
            "stats".to_string(),
        ],
    };
    scheduler.add_job(bridge_stats_job).await?;

    // Register infrastructure jobs (DB pool health & monthly cleanup)
    // These use the same database pool as the backup service
    let db_pool = backup_service.db_pool().clone();

    // Register handler for DB pool health check (every 5 seconds)
    let pool = db_pool.clone();
    scheduler
        .register_handler("db_pool_health", move |_job_def| {
            let pool = pool.clone();
            Box::pin(async move {
                // Pool metrics would be updated here via sqlx PoolStats
                agrocore_logging::debug!("Pool health check tick");
                Ok(())
            })
        })
        .await;

    // Add pool health check job (every 5 seconds)
    let pool_health_job = JobDefinition {
        id: "db_pool_health".to_string(),
        name: "Database Pool Health Check".to_string(),
        description: "Periodic database pool health check".to_string(),
        schedule: "* * * * * *".to_string(), // Every 5 seconds
        timezone: Some("UTC".to_string()),
        job_type: JobType::Builtin {
            handler: "db_pool_health".to_string(),
        },
        payload: serde_json::json!({}),
        timeout_seconds: Some(30),
        max_retries: Some(3),
        retry_delay_seconds: Some(10),
        enabled: true,
        tags: vec![
            "infrastructure".to_string(),
            "database".to_string(),
            "health".to_string(),
        ],
    };
    scheduler.add_job(pool_health_job).await?;

    // Register handler for monthly depreciation/cleanup (monthly)
    scheduler
        .register_handler("db_monthly_cleanup", move |_job_def| {
            Box::pin(async move {
                agrocore_logging::info!("Monthly depreciation automation tick");
                // Here: call depreciation calculation for all active equipment
                Ok(())
            })
        })
        .await;

    // Add monthly cleanup job (monthly, first day of month at midnight)
    let monthly_cleanup_job = JobDefinition {
        id: "db_monthly_cleanup".to_string(),
        name: "Monthly Database Cleanup & Depreciation".to_string(),
        description: "Monthly depreciation calculation and cleanup".to_string(),
        schedule: "0 0 1 * *".to_string(), // Monthly, first day at midnight
        timezone: Some("UTC".to_string()),
        job_type: JobType::Builtin {
            handler: "db_monthly_cleanup".to_string(),
        },
        payload: serde_json::json!({}),
        timeout_seconds: Some(3600),
        max_retries: Some(3),
        retry_delay_seconds: Some(60),
        enabled: true,
        tags: vec![
            "infrastructure".to_string(),
            "database".to_string(),
            "cleanup".to_string(),
            "depreciation".to_string(),
        ],
    };
    scheduler.add_job(monthly_cleanup_job).await?;

    // Set scheduler on backup service
    backup_service.scheduler = Some(scheduler.clone());

    info!("Backup service started successfully");
    info!("  DB Schedule: {}", backup_service.config().schedule_db);
    info!(
        "  Config Schedule: {}",
        backup_service.config().schedule_config
    );
    info!("  Targets: {}", backup_service.config().targets.len());
    info!(
        "  Retention: daily={}, weekly={}, monthly={}, yearly={}",
        backup_service.config().retention.daily,
        backup_service.config().retention.weekly,
        backup_service.config().retention.monthly,
        backup_service.config().retention.yearly
    );

    // Run scheduler in background task (scheduler is Send)
    let scheduler_handle = tokio::spawn(async move {
        // Keep scheduler running until shutdown
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    // Run bridge on main thread (blocks until shutdown signal)
    info!("Starting MQTT Bridge on main thread...");
    if let Err(e) = bridge.start().await {
        agrocore_logging::error!("MQTT Bridge error: {}", e);
    }

    // Wait for shutdown signal (bridge.start() waits for shutdown internally via shutdown_tx)
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping gracefully...");
        }
        Err(e) => {
            agrocore_logging::error!("Failed to listen for shutdown signal: {}", e);
        }
    }

    // Stop scheduler
    if let Some(scheduler) = &backup_service.scheduler {
        if let Err(e) = scheduler.stop().await {
            agrocore_logging::error!("Failed to stop scheduler: {}", e);
        }
    }
    info!("Scheduler stopped");

    // Wait for scheduler task to finish
    let _ = scheduler_handle.await;

    // Close NATS connection
    nats_client.close().await;
    info!("NATS connection closed");

    info!("Backup service stopped gracefully");
    Ok(())
}
