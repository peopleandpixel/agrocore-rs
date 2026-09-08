use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use agrocore_infrastructure::Database;
use agrocore_logging::{error, info};
use agrocore_scheduler::{JobDefinition, JobType, SchedulerConfig, SchedulerService};
use agrocore_shared::config::AgroCoreConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

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
        .unwrap_or_else(|_| "postgres://postgres:***@localhost:5432/agrocore".to_string());
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3010".to_string());

    let db = Database::connect(&database_url).await?;

    println!("Weather-IoT Service starting on {}...", bind_addr);

    // Load AgroCore config
    let _config = AgroCoreConfig::init_global(AgroCoreConfig::from_env());

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

    let nats_client = agrocore_messaging::MessagingClient::connect(&nats_url).await?;
    let scheduler =
        Arc::new(SchedulerService::new(scheduler_config, Some(nats_client.nats().clone())).await?);

    // Register handler for weather updates
    let db_for_handler = db.clone();
    let nats_url_for_handler = nats_url.clone();
    scheduler
        .register_handler("weather_update", move |_job_def| {
            let db = db_for_handler.clone();
            let nats_url = nats_url_for_handler.clone();
            Box::pin(async move {
                if let Err(e) = worker::run_weather_update(db, nats_url).await {
                    agrocore_logging::error!("Weather update failed: {}", e);
                }
                Ok(())
            })
        })
        .await;

    // Start the scheduler
    scheduler.start().await?;

    // Add weather update job (every 30 minutes)
    let weather_job = JobDefinition {
        id: "weather_update".to_string(),
        name: "Weather Update".to_string(),
        description: "Fetch weather data for all tenants every 30 minutes".to_string(),
        schedule: "0 */30 * * * *".to_string(), // Every 30 minutes
        timezone: Some("UTC".to_string()),
        job_type: JobType::Builtin {
            handler: "weather_update".to_string(),
        },
        payload: serde_json::json!({}),
        timeout_seconds: Some(3600),
        max_retries: Some(3),
        retry_delay_seconds: Some(60),
        enabled: true,
        tags: vec!["weather".to_string(), "update".to_string()],
    };
    scheduler.add_job(weather_job).await?;

    info!("Scheduler started with weather update job");

    let db_clone = db.clone();
    let nats_url_clone = nats_url.clone();

    // Start worker in background (for NATS message handling)
    let timeout_duration = std::time::Duration::from_secs(30);
    tokio::spawn(async move {
        match tokio::time::timeout(
            timeout_duration,
            worker::start_nats_listener(db_clone, nats_url_clone),
        )
        .await
        {
            Ok(Ok(())) => info!("Weather worker completed"),
            Ok(Err(e)) => error!("Worker error: {}", e),
            Err(_) => error!("Timeout: Weather worker exceeded 30s"),
        }
    });

    // Start HTTP server for health checks
    let server = HttpServer::new(|| App::new().route("/health", web::get().to(health)))
        .bind(bind_addr)?
        .run();

    // Wait for shutdown signal
    let ctrl_c = signal::ctrl_c();
    let server_handle = tokio::spawn(server);

    tokio::select! {
        _ = ctrl_c => {
            info!("Shutdown signal received, stopping gracefully...");
        }
        _ = server_handle => {
            error!("HTTP server stopped unexpectedly");
        }
    }

    // Stop scheduler
    if let Err(e) = scheduler.stop().await {
        error!("Failed to stop scheduler: {}", e);
    }
    info!("Scheduler stopped");

    Ok(())
}
