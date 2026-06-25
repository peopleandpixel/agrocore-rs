use agrocore_infrastructure::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    agrocore_shared::telemetry::init_telemetry("agrocore_api");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DATABASE_NAME")
        .unwrap_or_else(|_| "agrocore".into());
    let bind_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into());

    tracing::info!("Connecting to MongoDB at {}", database_url);
    let db = Database::connect(&database_url, &db_name).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://192.168.1.44:4222".to_string());
    let messaging = agrocore_messaging::MessagingClient::connect(&nats_url).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    tracing::info!("Server starting on {}", bind_addr);
    agrocore_api::run_server(db, messaging, &bind_addr).await
}
