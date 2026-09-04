use agrocore_backup::config::load_config;
use agrocore_backup::nats_client::NatsClient as BackupNatsClient;
use agrocore_backup::service::BackupService;
use agrocore_infrastructure::Database;
use std::sync::Arc;

use agrocore_logging::{debug, error, info, warn};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    agrocore_shared::telemetry::init_telemetry_with_logs("agrocore_api");
    let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();

    let _config = agrocore_shared::config::AgroCoreConfig::init_global(
        agrocore_shared::config::AgroCoreConfig::from_env(),
    );
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://agrocore:***@localhost:5432/agrocore".into());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());

    info!("Connecting to PostgreSQL at {}", database_url);
    let db = agrocore_infrastructure::PostgresDb::connect(&database_url)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let messaging = agrocore_messaging::MessagingClient::connect(&nats_url)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Initialize backup service
    let backup_config = load_config().unwrap_or_default();
    let backup_service = if backup_config.enabled {
        let backup_nats = BackupNatsClient::connect(&nats_url)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        match BackupService::new(backup_config, database_url.clone(), backup_nats).await {
            Ok(svc) => Some(Arc::new(svc)),
            Err(e) => {
                error!("Failed to initialize backup service: {}", e);
                None
            }
        }
    } else {
        None
    };

    info!("Server starting on {}", bind_addr);
    agrocore_api::run_server(
        Database::Postgres(db),
        messaging,
        &bind_addr,
        backup_service,
    )
    .await
}
