use agrocore_infrastructure::Database;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};

pub mod worker;

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok", "service": "asset-registry"}))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    agrocore_shared::telemetry::init_telemetry("agrocore_asset_registry");

    let mongodb_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://192.168.1.69:27017".to_string());
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://192.168.1.44:4222".to_string());
    let bind_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3004".to_string());

    let db = Database::connect(&mongodb_uri, "agrocore").await?;
    
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
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
    })
    .bind(bind_addr)?
    .run()
    .await?;

    Ok(())
}
