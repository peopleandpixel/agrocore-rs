use agrocore_infrastructure::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    agrocore_shared::telemetry::init_telemetry("agrocore_api");
    let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();

    let _config = agrocore_shared::config::AgroCoreConfig::init_global(
        agrocore_shared::config::AgroCoreConfig::from_env(),
    );
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://agrocore:***@localhost:5432/agrocore".into());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());

    tracing::info!("Connecting to PostgreSQL at {}", database_url);
    let db = agrocore_infrastructure::PostgresDb::connect(&database_url)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let messaging = agrocore_messaging::MessagingClient::connect(&nats_url)
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    tracing::info!("Server starting on {}", bind_addr);
    agrocore_api::run_server(Database::Postgres(db), messaging, &bind_addr).await
}
