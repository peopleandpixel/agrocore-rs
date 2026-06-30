use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use crate::dto::{SiteDto, CreateSiteDto, UpdateSiteDto, ErrorResponse, PaginatedResponseDto, PaginatedSiteResponse};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;
use agrocore_messaging::{Event, GlobalEvent};

#[utoipa::path(
    get,
    path = "/api/v1/sites",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List sites", body = PaginatedSiteResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn list_sites(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    tracing::info!("Listing sites for tenant: {}", auth.0.tenant_id);
    match state.db.site_repo().find_all_visible(auth.0.tenant_id.into(), query.0, auth.0.user_id, &auth.roles()).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(SiteDto::from).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => {
            tracing::error!("Failed to list sites: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal".into(),
                message: e.to_string(),
            })
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/sites/{id}",
    responses(
        (status = 200, description = "Site details", body = SiteDto),
        (status = 404, description = "Site not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn get_site(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let site_id = *path;
    tracing::info!("Getting site {} for tenant: {}", site_id, auth.0.tenant_id);
    match state.db.site_repo().find_by_id_visible(auth.0.tenant_id.into(), site_id, auth.0.user_id, &auth.roles()).await {
        Ok(Some(site)) => HttpResponse::Ok().json(SiteDto::from(site)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => {
            tracing::error!("Failed to get site {}: {}", site_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/sites",
    request_body = CreateSiteDto,
    responses(
        (status = 201, description = "Site created", body = SiteDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn create_site(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateSiteDto>,
) -> impl Responder {
    if let Err(e) = auth.require_manager() {
        return HttpResponse::Forbidden().json(ErrorResponse { error: "forbidden".into(), message: e.to_string() });
    }
    tracing::info!("Creating site for tenant: {}", auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Site validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.site_repo().create(auth.0.tenant_id.into(), dto.0.into(), auth.0.user_id).await {
        Ok(site) => {
            let event = Event::new("api".into(), GlobalEvent::SiteCreated(site.clone()));
            let _ = state.messaging.publish("events.sites", &event).await;
            HttpResponse::Created().json(SiteDto::from(site))
        },
        Err(e) => {
            tracing::error!("Failed to create site: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/sites/{id}",
    request_body = UpdateSiteDto,
    responses(
        (status = 200, description = "Site updated", body = SiteDto),
        (status = 404, description = "Site not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn update_site(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateSiteDto>,
) -> impl Responder {
    if let Err(e) = auth.require_manager() {
        return HttpResponse::Forbidden().json(ErrorResponse { error: "forbidden".into(), message: e.to_string() });
    }
    let site_id = *path;
    tracing::info!("Updating site {} for tenant: {}", site_id, auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Site update validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.site_repo().update(auth.0.tenant_id.into(), site_id, dto.0.into(), auth.0.user_id).await {
        Ok(Some(site)) => {
            let event = Event::new("api".into(), GlobalEvent::SiteUpdated(site.clone()));
            let _ = state.messaging.publish("events.sites", &event).await;
            HttpResponse::Ok().json(SiteDto::from(site))
        },
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => {
            tracing::error!("Failed to update site {}: {}", site_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/sites/{id}",
    responses(
        (status = 200, description = "Site deleted"),
        (status = 404, description = "Site not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn delete_site(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    if let Err(e) = auth.require_manager() {
        return HttpResponse::Forbidden().json(ErrorResponse { error: "forbidden".into(), message: e.to_string() });
    }
    let site_id = *path;
    tracing::info!("Deleting site {} for tenant: {}", site_id, auth.0.tenant_id);
    match state.db.site_repo().delete(auth.0.tenant_id.into(), site_id).await {
        Ok(true) => {
            let event = Event::new("api".into(), GlobalEvent::SiteDeleted(site_id));
            let _ = state.messaging.publish("events.sites", &event).await;
            HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
        },
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => {
            tracing::error!("Failed to delete site {}: {}", site_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}
