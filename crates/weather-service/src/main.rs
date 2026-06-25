use agrocore_infrastructure::Database;

pub mod worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    agrocore_shared::telemetry::init_telemetry("agrocore_weather_service");

    let mongodb_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://192.168.1.69:27017".to_string());
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://192.168.1.44:4222".to_string());

    let db = Database::connect(&mongodb_uri, "agrocore").await?;
    
    println!("Weather-IoT Service started...");
    
    worker::start(db, nats_url).await?;

    Ok(())
}
