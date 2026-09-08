use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use agrocore_infrastructure::Database;
use agrocore_logging::{EnvironmentType, LoggingConfig, error, info, init_logging};
use std::time::Duration;

pub mod worker;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "service": "asset-registry"}))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

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
    init_logging(logging_config)?;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:***@localhost:5432/agrocore".to_string());
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3004".to_string());

    let db = Database::connect(&database_url).await?;

    println!("Asset Registry starting on {}...", bind_addr);

    let db_clone = db.clone();
    let nats_url_clone = nats_url.clone();

    // Start NATS worker in background
    tokio::spawn(async move {
        if let Err(e) = worker::start(db_clone, nats_url_clone).await {
            eprintln!("Asset worker error: {}", e);
        }
    });

    // Start HTTP server for health checks
    HttpServer::new(|| App::new().route("/health", web::get().to(health)))
        .bind(bind_addr)?
        .run()
        .await?;

    Ok(())
}
