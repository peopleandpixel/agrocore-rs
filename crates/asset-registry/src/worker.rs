use agrocore_infrastructure::Database;
use agrocore_messaging::{MessagingClient, Event, GlobalEvent};
use futures::StreamExt;
use tracing::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetRequest {
    GetEquipment { id: Uuid },
    GetLivestock { id: Uuid },
    ListAssets { tenant_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetResponse {
    Equipment(agrocore_domain::entities::equipment::Equipment),
    Livestock(agrocore_domain::entities::livestock::Animal),
    List(Vec<AssetSummary>),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: Uuid,
    pub label: String,
    pub asset_type: String,
}

pub async fn start(_db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("assets.request").await?;

    info!("Asset Registry worker listening on assets.request");

    while let Some(message) = subscriber.next().await {
        let event: Event<AssetRequest> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(_) => {
                // Check for GlobalEvent (HealthCheck)
                if let Ok(global_event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload) {
                    if matches!(global_event.payload, GlobalEvent::HealthCheckRequested) {
                        if let Some(reply_to) = message.reply {
                            let response = serde_json::json!({"status": "ok", "service": "asset-registry"});
                            let _ = messaging.publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?).await;
                        }
                    }
                }
                continue;
            }
        };

        info!("Received asset request: {:?}", event.payload);
        // Implement logic using db and repos...
        // For now returning errors or placeholders as we focus on structure
        let response = AssetResponse::Error("Not fully implemented yet".into());

        if let Some(reply_to) = message.reply {
            let response_payload = serde_json::to_vec(&response)?;
            messaging.publish_raw(reply_to.as_str(), response_payload).await?;
        }
    }

    Ok(())
}
