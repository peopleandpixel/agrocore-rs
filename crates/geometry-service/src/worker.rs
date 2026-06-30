use agrocore_infrastructure::Database;
use agrocore_messaging::{MessagingClient, Event, GlobalEvent};
use futures::StreamExt;
use tracing::info;
use serde::{Deserialize, Serialize};
use geo::prelude::*;
use geo::{Polygon, Point};

#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryRequest {
    CalculateArea { points: Vec<(f64, f64)> },
    CheckPointInPolygon { point: (f64, f64), polygon: Vec<(f64, f64)> },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GeometryResponse {
    Area(f64),
    Inside(bool),
    Error(String),
}

pub async fn start(_db: Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("geometry.request").await?;

    info!("Geometry worker listening on geometry.request");

    while let Some(message) = subscriber.next().await {
        let event: Event<GeometryRequest> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(_) => {
                // Check for GlobalEvent (HealthCheck)
                if let Ok(global_event) = serde_json::from_slice::<Event<GlobalEvent>>(&message.payload) {
                    if matches!(global_event.payload, GlobalEvent::HealthCheckRequested) {
                        if let Some(reply_to) = message.reply {
                            let response = serde_json::json!({"status": "ok", "service": "geometry"});
                            let _ = messaging.publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?).await;
                        }
                    }
                }
                continue;
            }
        };

        let response = match event.payload {
            GeometryRequest::CalculateArea { points } => {
                let polygon = Polygon::new(
                    geo::LineString::from(points.into_iter().map(|(lng, lat)| geo::Coord { x: lng, y: lat }).collect::<Vec<_>>()),
                    vec![],
                );
                // Simple area calculation (Euclidean). In real world use geodesic for ha.
                // For this demo we use the crate's signed_area or similar.
                GeometryResponse::Area(polygon.unsigned_area())
            }
            GeometryRequest::CheckPointInPolygon { point, polygon } => {
                let p = Point::new(point.0, point.1);
                let poly = Polygon::new(
                    geo::LineString::from(polygon.into_iter().map(|(lng, lat)| geo::Coord { x: lng, y: lat }).collect::<Vec<_>>()),
                    vec![],
                );
                GeometryResponse::Inside(poly.contains(&p))
            }
        };

        if let Some(reply_to) = message.reply {
            let response_payload = serde_json::to_vec(&response)?;
            messaging.publish_raw(reply_to.as_str(), response_payload).await?;
        }
    }

    Ok(())
}
