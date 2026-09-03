use crate::config::load_config;
use crate::error::{BackupError, BackupResult};
use crate::nats_client::NatsClient;
use crate::service::BackupService;
use agrocore_logging::{info, warn};
use agrocore_shared::config::AgroCoreConfig;
use std::sync::Arc;
use tokio::signal;
use tracing::error;

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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,agrocore_backup=debug".into()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!(
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

    // Create backup service
    let mut backup_service = BackupService::new(
        backup_config,
        agro_config.database_url.clone(),
        nats_client.clone(),
    )
    .await?;

    // Start scheduled jobs
    backup_service.start_scheduler().await?;

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

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping gracefully...");
        }
        Err(e) => {
            error!("Failed to listen for shutdown signal: {}", e);
        }
    }

    // Stop scheduler
    if let Some(scheduler) = &backup_service.scheduler {
        if let Err(e) = scheduler.stop().await {
            error!("Failed to stop scheduler: {}", e);
        }
    }
    info!("Scheduler stopped");

    // Close NATS connection
    nats_client.close().await;
    info!("NATS connection closed");

    info!("Backup service stopped gracefully");
    Ok(())
}
