use agrocore_backup::config::load_config;
use agrocore_backup::nats_client::NatsClient as BackupNatsClient;
use agrocore_backup::service::BackupService;
use agrocore_infrastructure::Database;
use std::sync::Arc;

use agrocore_logging::{error, info};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    agrocore_shared::telemetry::init_telemetry_with_logs("agrocore_api");
    let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();

    let _config = agrocore_shared::config::AgroCoreConfig::init_global(
        agrocore_shared::config::AgroCoreConfig::from_env(),
    );
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://agrocore:agrocore@localhost:5432/agrocore".into());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());

    info!("Connecting to PostgreSQL at {}", database_url);
    let db = agrocore_infrastructure::PostgresDb::connect(&database_url)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Messaging is a hard dependency of the notification dispatcher, but not of
    // serving data. Previously an unreachable broker aborted startup, so one
    // missing dependency took the whole API offline. It is now required only
    // when explicitly enabled.
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let messaging_required = std::env::var("MESSAGING_REQUIRED")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    let messaging = match agrocore_messaging::MessagingClient::connect(&nats_url).await {
        Ok(m) => Some(m),
        Err(e) if messaging_required => {
            return Err(std::io::Error::other(format!(
                "NATS required but unreachable at {nats_url}: {e}"
            )));
        }
        Err(e) => {
            error!(
                "NATS unreachable at {}, continuing without messaging: {}",
                nats_url, e
            );
            None
        }
    };

    // Initialize backup service
    let backup_config = load_config().unwrap_or_default();
    // Backups are enabled by default, but an unreachable broker must not stop
    // the API from serving data. The `?` here used to abort startup, which meant
    // one missing dependency took the whole system offline.
    let backup_service = if backup_config.enabled {
        match BackupNatsClient::connect(&nats_url).await {
            Ok(backup_nats) => {
                match BackupService::new(backup_config, database_url.clone(), backup_nats).await {
                    Ok(svc) => Some(Arc::new(svc)),
                    Err(e) => {
                        error!("Failed to initialize backup service: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!(
                    "NATS unreachable at {}, backup service disabled: {}",
                    nats_url, e
                );
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
