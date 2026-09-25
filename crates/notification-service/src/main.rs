//! Notification Service - handles push notifications, email, SMS, etc.

use agrocore_shared::telemetry::init_telemetry;
use anyhow::Result;
use async_nats;
use futures::StreamExt;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry("agrocore-notification-service");
    info!("Starting agrocore-notification-service");

    // Connect to NATS
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let nc = async_nats::connect(&nats_url).await?;
    info!("Connected to NATS at {}", nats_url);

    // Subscribe to notification topics
    let mut sub = nc.subscribe("agrocore.notifications.>").await?;
    info!("Subscribed to agrocore.notifications.>");

    // Main loop
    while let Some(msg) = sub.next().await {
        info!(
            "Received notification: {:?}",
            String::from_utf8_lossy(&msg.payload)
        );
        // TODO: Process notification (email, push, SMS, etc.)
    }

    Ok(())
}
