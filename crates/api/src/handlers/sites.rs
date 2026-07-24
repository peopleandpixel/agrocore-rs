use crate::AppState;
use crate::dto::{
    CreateSiteDto, ErrorResponse, PaginatedResponseDto, PaginatedSiteResponse, SiteDto,
    UpdateSiteDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_messaging::{Event, GlobalEvent};
use agrocore_shared::SharedError;
use validator::Validate;

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
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        "Listing sites for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .site_repo()
        .find_all_visible(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.0,
            auth.0.user_id,
            &auth.roles(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(SiteDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
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
) -> Result<HttpResponse, ApiError> {
    let site_id = *path;
    tracing::info!(
        "Getting site {} for tenant: {}",
        site_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let site = state
        .db
        .site_repo()
        .find_by_id_visible(
            agrocore_domain::TenantId(auth.0.tenant_id),
            site_id,
            auth.0.user_id,
            &auth.roles(),
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;
    Ok(HttpResponse::Ok().json(SiteDto::from(site)))
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
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    tracing::info!(
        "Creating site for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let site = state
        .db
        .site_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    let event = Event::new("api".into(), GlobalEvent::SiteCreated(site.clone()));
    let _ = state.messaging.publish("events.sites", &event).await;
    Ok(HttpResponse::Created().json(SiteDto::from(site)))
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
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let site_id = *path;
    tracing::info!(
        "Updating site {} for tenant: {}",
        site_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let site = state
        .db
        .site_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            site_id,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;
    let event = Event::new("api".into(), GlobalEvent::SiteUpdated(site.clone()));
    let _ = state.messaging.publish("events.sites", &event).await;
    Ok(HttpResponse::Ok().json(SiteDto::from(site)))
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
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let site_id = *path;
    tracing::info!(
        "Deleting site {} for tenant: {}",
        site_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    if state
        .db
        .site_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), site_id)
        .await?
    {
        let event = Event::new("api".into(), GlobalEvent::SiteDeleted(site_id));
        let _ = state.messaging.publish("events.sites", &event).await;
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Site not found".into()).into())
    }
}
