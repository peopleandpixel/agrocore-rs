use crate::AppState;
use crate::dto::ErrorResponse;
use crate::error::ApiError;
use crate::handlers::reporting::worker::{ReportingRequest, ReportingResponse};
use crate::middleware::AuthExtractor;
use actix_web::{HttpResponse, web};
use agrocore_messaging::Event;
use agrocore_shared::SharedError;

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
            .service(
                web::resource("/export/orders/excel").route(web::get().to(export_orders_excel)),
            )
            .service(
                web::resource("/export/sites/geojson").route(web::get().to(export_sites_geojson)),
            )
            .service(web::resource("/export/pac/sip").route(web::get().to(export_pac_sip)))
            .service(web::resource("/export/veterinary").route(web::get().to(export_veterinary))),
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
) -> Result<HttpResponse, ApiError> {
    let request = ReportingRequest::OrdersExcel {
        tenant_id: auth.0.tenant_id,
    };
    let event = Event::new("api".into(), request);

    let response = state
        .messaging
        .request::<_, ReportingResponse>("reporting.request", &event)
        .await
        .map_err(|e| SharedError::Internal(e.to_string()))?;

    match response {
        ReportingResponse::Excel(buffer) => Ok(HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"orders.xlsx\"",
            ))
            .body(buffer)),
        ReportingResponse::Error(e) => Err(SharedError::Internal(e).into()),
        _ => Err(SharedError::Internal("Wrong response type".into()).into()),
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
) -> Result<HttpResponse, ApiError> {
    let request = ReportingRequest::SitesGeoJson {
        tenant_id: auth.0.tenant_id,
    };
    let event = Event::new("api".into(), request);

    let response = state
        .messaging
        .request::<_, ReportingResponse>("reporting.request", &event)
        .await
        .map_err(|e| SharedError::Internal(e.to_string()))?;

    match response {
        ReportingResponse::GeoJson(feature_collection) => {
            Ok(HttpResponse::Ok().json(feature_collection))
        }
        ReportingResponse::Error(e) => Err(SharedError::Internal(e).into()),
        _ => Err(SharedError::Internal("Wrong response type".into()).into()),
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
) -> Result<HttpResponse, ApiError> {
    let request = ReportingRequest::PacSipExcel {
        tenant_id: auth.0.tenant_id,
    };
    let event = Event::new("api".into(), request);

    let response = state
        .messaging
        .request::<_, ReportingResponse>("reporting.request", &event)
        .await
        .map_err(|e| SharedError::Internal(e.to_string()))?;

    match response {
        ReportingResponse::Excel(buffer) => Ok(HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"pac_sip_report.xlsx\"",
            ))
            .body(buffer)),
        ReportingResponse::Error(e) => Err(SharedError::Internal(e).into()),
        _ => Err(SharedError::Internal("Wrong response type".into()).into()),
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
) -> Result<HttpResponse, ApiError> {
    let request = ReportingRequest::VeterinaryExcel {
        tenant_id: auth.0.tenant_id,
    };
    let event = Event::new("api".into(), request);

    let response = state
        .messaging
        .request::<_, ReportingResponse>("reporting.request", &event)
        .await
        .map_err(|e| SharedError::Internal(e.to_string()))?;

    match response {
        ReportingResponse::Excel(buffer) => Ok(HttpResponse::Ok()
            .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"veterinary_report.xlsx\"",
            ))
            .body(buffer)),
        ReportingResponse::Error(e) => Err(SharedError::Internal(e).into()),
        _ => Err(SharedError::Internal("Wrong response type".into()).into()),
    }
}
