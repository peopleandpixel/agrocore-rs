use actix_web::{web, HttpResponse, Responder};
use crate::AppState;
use crate::middleware::AuthExtractor;
use crate::dto::ErrorResponse;
use agrocore_messaging::Event;
use crate::handlers::reporting::worker::{ReportingRequest, ReportingResponse};

pub mod worker {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ReportingRequest {
        OrdersExcel { tenant_id: Uuid },
        SitesGeoJson { tenant_id: Uuid },
        PacSipExcel { tenant_id: Uuid },
        VeterinaryExcel { tenant_id: Uuid },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ReportingResponse {
        Excel(Vec<u8>),
        GeoJson(geojson::FeatureCollection),
        Error(String),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/reporting")
            .service(web::resource("/export/orders/excel").route(web::get().to(export_orders_excel)))
            .service(web::resource("/export/sites/geojson").route(web::get().to(export_sites_geojson)))
            .service(web::resource("/export/pac/sip").route(web::get().to(export_pac_sip)))
            .service(web::resource("/export/veterinary").route(web::get().to(export_veterinary)))
    );
}

#[utoipa::path(
    get,
    path = "/api/v1/reporting/export/orders/excel",
    responses(
        (status = 200, description = "Excel file of orders", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_orders_excel(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> impl Responder {
    let request = ReportingRequest::OrdersExcel { tenant_id: auth.0.tenant_id };
    let event = Event::new("api".into(), request);
    
    match state.messaging.request::<_, ReportingResponse>("reporting.request", &event).await {
        Ok(response) => {
            match response {
                ReportingResponse::Excel(buffer) => HttpResponse::Ok()
                    .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                    .insert_header(("Content-Disposition", "attachment; filename=\"orders.xlsx\""))
                    .body(buffer),
                ReportingResponse::Error(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Reporting Service error".into(), message: e }),
                _ => HttpResponse::InternalServerError().json(ErrorResponse { error: "Unexpected response".into(), message: "Wrong response type".into() }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Messaging error".into(), message: e.to_string() }),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/reporting/export/sites/geojson",
    responses(
        (status = 200, description = "GeoJSON of sites", body = String),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_sites_geojson(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> impl Responder {
    let request = ReportingRequest::SitesGeoJson { tenant_id: auth.0.tenant_id };
    let event = Event::new("api".into(), request);
    
    match state.messaging.request::<_, ReportingResponse>("reporting.request", &event).await {
        Ok(response) => {
            match response {
                ReportingResponse::GeoJson(feature_collection) => HttpResponse::Ok().json(feature_collection),
                ReportingResponse::Error(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Reporting Service error".into(), message: e }),
                _ => HttpResponse::InternalServerError().json(ErrorResponse { error: "Unexpected response".into(), message: "Wrong response type".into() }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Messaging error".into(), message: e.to_string() }),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/reporting/export/pac/sip",
    responses(
        (status = 200, description = "SIP (PAC) report in Excel format", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_pac_sip(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> impl Responder {
    let request = ReportingRequest::PacSipExcel { tenant_id: auth.0.tenant_id };
    let event = Event::new("api".into(), request);
    
    match state.messaging.request::<_, ReportingResponse>("reporting.request", &event).await {
        Ok(response) => {
            match response {
                ReportingResponse::Excel(buffer) => HttpResponse::Ok()
                    .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                    .insert_header(("Content-Disposition", "attachment; filename=\"pac_sip_report.xlsx\""))
                    .body(buffer),
                ReportingResponse::Error(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Reporting Service error".into(), message: e }),
                _ => HttpResponse::InternalServerError().json(ErrorResponse { error: "Unexpected response".into(), message: "Wrong response type".into() }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Messaging error".into(), message: e.to_string() }),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/reporting/export/veterinary",
    responses(
        (status = 200, description = "Veterinary report in Excel format", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_veterinary(
    state: web::Data<AppState>,
    auth: AuthExtractor,
) -> impl Responder {
    let request = ReportingRequest::VeterinaryExcel { tenant_id: auth.0.tenant_id };
    let event = Event::new("api".into(), request);
    
    match state.messaging.request::<_, ReportingResponse>("reporting.request", &event).await {
        Ok(response) => {
            match response {
                ReportingResponse::Excel(buffer) => HttpResponse::Ok()
                    .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                    .insert_header(("Content-Disposition", "attachment; filename=\"veterinary_report.xlsx\""))
                    .body(buffer),
                ReportingResponse::Error(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Reporting Service error".into(), message: e }),
                _ => HttpResponse::InternalServerError().json(ErrorResponse { error: "Unexpected response".into(), message: "Wrong response type".into() }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Messaging error".into(), message: e.to_string() }),
    }
}
