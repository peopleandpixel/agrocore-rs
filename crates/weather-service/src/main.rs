use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use agrocore_infrastructure::Database;
use agrocore_logging::{error, info};
use std::time::Duration;
/// Timeout-Struktur für Service-Operationen (OPT-010)
pub const SERVICE_TIMEOUT_SECS: u64 = 30;
pub const WORKER_TIMEOUT: Duration = Duration::from_secs(SERVICE_TIMEOUT_SECS);

pub mod providers;
pub mod worker;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "service": "weather"}))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    agrocore_shared::telemetry::init_telemetry("agrocore_weather_service");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/agrocore".to_string());
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3010".to_string());

    let db = Database::connect(&database_url).await?;

    println!("Weather-IoT Service starting on {}...", bind_addr);

    let db_clone = db.clone();
    let nats_url_clone = nats_url.clone();

    // Start worker in background with 30s timeout (OPT-010)
    let timeout_duration = std::time::Duration::from_secs(30);
    tokio::spawn(async move {
        match tokio::time::timeout(timeout_duration, worker::start(db_clone, nats_url_clone)).await
        {
            Ok(Ok(())) => info!("Weather worker completed"),
            Ok(Err(e)) => error!("Worker error: {}", e),
            Err(_) => error!("Timeout: Weather worker exceeded 30s"),
        }
    });

    // Start HTTP server for health checks
    HttpServer::new(|| App::new().route("/health", web::get().to(health)))
        .bind(bind_addr)?
        .run()
        .await?;

    Ok(())
}
