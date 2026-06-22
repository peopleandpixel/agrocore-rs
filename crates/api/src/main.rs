use agrocore_infrastructure::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("agrocore_api=info,tower_http=info")
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DATABASE_NAME")
        .unwrap_or_else(|_| "agrocore".into());
    let bind_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into());

    tracing::info!("Connecting to MongoDB at {}", database_url);
    let db = Database::connect(&database_url, &db_name).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    tracing::info!("Server starting on {}", bind_addr);
    agrocore_api::run_server(db, &bind_addr).await
}
