use crate::ReportingService;
use agrocore_messaging::{MessagingClient, Event, GlobalEvent};
use futures::StreamExt;
use tracing::{info, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum ReportingRequest {
    OrdersExcel { tenant_id: Uuid },
    SitesGeoJson { tenant_id: Uuid },
    PacSipExcel { tenant_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReportingResponse {
    Excel(Vec<u8>),
    GeoJson(geojson::FeatureCollection),
    Error(String),
}

pub async fn start(service: ReportingService, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("reporting.request").await?;

    info!("Reporting worker listening on reporting.request");

    while let Some(message) = subscriber.next().await {
        let event: Event<ReportingRequest> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to deserialize reporting request: {}", e);
                continue;
            }
        };

        info!("Received reporting request: {:?}", event.payload);

        let response = match event.payload {
            ReportingRequest::OrdersExcel { tenant_id } => {
                match service.generate_orders_excel(tenant_id).await {
                    Ok(data) => ReportingResponse::Excel(data),
                    Err(e) => ReportingResponse::Error(e.to_string()),
                }
            }
            ReportingRequest::SitesGeoJson { tenant_id } => {
                match service.generate_sites_geojson(tenant_id).await {
                    Ok(data) => ReportingResponse::GeoJson(data),
                    Err(e) => ReportingResponse::Error(e.to_string()),
                }
            }
            ReportingRequest::PacSipExcel { tenant_id } => {
                match service.generate_pac_sip_excel(tenant_id).await {
                    Ok(data) => ReportingResponse::Excel(data),
                    Err(e) => ReportingResponse::Error(e.to_string()),
                }
            }
        };

        if let Some(reply_to) = message.reply {
            let response_payload = serde_json::to_vec(&response)?;
            messaging.publish_raw(reply_to.as_str(), response_payload).await?;
        }
    }

    Ok(())
}

pub async fn start_audit_worker(db: agrocore_infrastructure::Database, nats_url: String) -> anyhow::Result<()> {
    let messaging = MessagingClient::connect(&nats_url).await?;
    let mut subscriber = messaging.subscribe("events.audit").await?;

    info!("Audit worker listening on events.audit");

    while let Some(message) = subscriber.next().await {
        let event: Event<GlobalEvent> = match serde_json::from_slice(&message.payload) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to deserialize audit event: {}", e);
                continue;
            }
        };

        match event.payload {
            GlobalEvent::AuditLogCreated(audit_log) => {
                info!("Received audit log event: {:?}", audit_log.id);
                if let Err(e) = db.audit_log_repo().create_log(audit_log).await {
                    error!("Failed to save audit log to DB: {}", e);
                }
            }
            GlobalEvent::HealthCheckRequested => {
                if let Some(reply_to) = message.reply {
                    let response = serde_json::json!({"status": "ok", "service": "reporting-audit"});
                    let _ = messaging.publish_raw(reply_to.as_str(), serde_json::to_vec(&response)?).await;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
