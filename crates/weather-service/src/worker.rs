use agrocore_infrastructure::Database;
use agrocore_messaging::{MessagingClient, Event, GlobalEvent};
use futures::StreamExt;
use tracing::{info, error};

pub async fn start(_db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("weather.>").await?;

    info!("Weather Service worker started, listening on weather.>");

    while let Some(message) = subscriber.next().await {
        let subject = message.subject.clone();
        
        if subject.as_str() == "weather.health" {
            if let Some(reply_to) = message.reply {
                let response = serde_json::json!({"status": "ok", "service": "weather"});
                let _ = messaging.publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?).await;
            }
            continue;
        }

        let event: Event<GlobalEvent> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to deserialize weather event on subject {}: {}", subject, e);
                continue;
            }
        };

        info!("Received message on subject: {}", subject);
        
        match event.payload {
            GlobalEvent::HealthCheckRequested => {
                if let Some(reply_to) = message.reply {
                    let response = serde_json::json!({"status": "ok", "service": "weather"});
                    let _ = messaging.publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?).await;
                }
            }
            _ => {
                // Handle other weather events
            }
        }
    }

    Ok(())
}
