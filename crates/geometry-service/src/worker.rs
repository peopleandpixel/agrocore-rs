use agrocore_infrastructure::Database;
use agrocore_logging::info;
use agrocore_messaging::{Event, GlobalEvent, MessagingClient};
use agrocore_shared::with_retry;
use futures::StreamExt;
use geo::prelude::*;
use geo::{Point, Polygon};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryRequest {
    CalculateArea {
        points: Vec<(f64, f64)>,
    },
    CheckPointInPolygon {
        point: (f64, f64),
        polygon: Vec<(f64, f64)>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryResponse {
    Area(f64),
    Inside(bool),
    Error(String),
}

pub async fn start(_db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = with_retry("geometry-nats-connect", 3, 1, || {
        let url = nats_url.clone();
        Box::pin(async move {
            MessagingClient::connect(&url)
                .await
                .map_err(|e| anyhow::anyhow!("NATS connection failed: {}", e))
        })
    })
    .await?;
    let mut subscriber = messaging.subscribe("geometry.request".to_string()).await?;

    info!("Geometry worker listening on geometry.request");

    while let Some(message) = subscriber.next().await {
        let event: Event<GeometryRequest> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(_) => {
                // Check for GlobalEvent (HealthCheck)
                if let Ok(global_event) =
                    serde_json::from_slice::<Event<GlobalEvent>>(&message.payload)
                    && matches!(global_event.payload, GlobalEvent::HealthCheckRequested)
                    && let Some(reply_to) = message.reply
                {
                    let response = serde_json::json!({"status": "ok", "service": "geometry"});
                    let _ = messaging
                        .publish_raw(reply_to.to_string(), serde_json::to_vec(&response)?)
                        .await;
                }
                continue;
            }
        };

        let response = match event.payload {
            GeometryRequest::CalculateArea { points } => {
                let polygon = Polygon::new(
                    geo::LineString::from(
                        points
                            .into_iter()
                            .map(|(lng, lat)| geo::Coord { x: lng, y: lat })
                            .collect::<Vec<_>>(),
                    ),
                    vec![],
                );
                // Simple area calculation (Euclidean). In real world use geodesic for ha.
                // For this demo we use the crate's signed_area or similar.
                GeometryResponse::Area(polygon.unsigned_area())
            }
            GeometryRequest::CheckPointInPolygon { point, polygon } => {
                let p = Point::new(point.0, point.1);
                let poly = Polygon::new(
                    geo::LineString::from(
                        polygon
                            .into_iter()
                            .map(|(lng, lat)| geo::Coord { x: lng, y: lat })
                            .collect::<Vec<_>>(),
                    ),
                    vec![],
                );
                GeometryResponse::Inside(poly.contains(&p))
            }
        };

        if let Some(reply_to) = message.reply {
            let response_payload = serde_json::to_vec(&response)?;
            messaging
                .publish_raw(reply_to.to_string(), response_payload)
                .await?;
        }
    }

    Ok(())
}
