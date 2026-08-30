use crate::AppState;
use crate::dto::{
    CreateLivestockDto, LivestockDto, PaginatedLivestockResponse, UpdateLivestockDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::livestock::{
    CreateLivestockDto as DomainCreateLivestockDto, Livestock, LivestockType,
    UpdateLivestockDto as DomainUpdateLivestockDto,
};
use agrocore_domain::repositories::LivestockRepository;
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/livestock",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List livestock", body = PaginatedLivestockResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_livestock(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .livestock_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedLivestockResponse {
        data: result.data.into_iter().map(LivestockDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/livestock",
    request_body = CreateLivestockDto,
    responses(
        (status = 201, description = "Livestock created", body = LivestockDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_livestock(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateLivestockDto>,
) -> Result<HttpResponse, ApiError> {
    let livestock = state
        .db
        .livestock_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(LivestockDto::from(livestock)))
}

#[utoipa::path(
    get,
    path = "/livestock/{id}",
    responses(
        (status = 200, description = "Livestock details", body = LivestockDto),
        (status = 404, description = "Livestock not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_livestock(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let livestock = state
        .db
        .livestock_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Livestock not found".into()))?;
    Ok(HttpResponse::Ok().json(LivestockDto::from(livestock)))
}

#[utoipa::path(
    put,
    path = "/livestock/{id}",
    request_body = UpdateLivestockDto,
    responses(
        (status = 200, description = "Livestock updated", body = LivestockDto),
        (status = 404, description = "Livestock not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_livestock(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateLivestockDto>,
) -> Result<HttpResponse, ApiError> {
    let livestock = state
        .db
        .livestock_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Livestock not found".into()))?;
    Ok(HttpResponse::Ok().json(LivestockDto::from(livestock)))
}

#[utoipa::path(
    delete,
    path = "/livestock/{id}",
    responses(
        (status = 200, description = "Livestock deleted"),
        (status = 404, description = "Livestock not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_livestock(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .livestock_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Livestock not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/livestock/by-plot/{plot_id}",
    params(
        ("plot_id" = Uuid, Path, description = "Plot UUID"),
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List livestock by plot", body = PaginatedLivestockResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_livestock_by_plot(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .livestock_repo()
        .find_by_plot(agrocore_domain::TenantId(auth.0.tenant_id), *path, query.0)
        .await?;

    let response = PaginatedLivestockResponse {
        data: result.data.into_iter().map(LivestockDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/livestock/by-herd/{herd_id}",
    responses(
        (status = 200, description = "List livestock by herd", body = Vec<LivestockDto>),
        (status = 404, description = "Herd not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_livestock_by_herd(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let livestock = state
        .db
        .livestock_repo()
        .find_by_herd(
            agrocore_domain::TenantId(auth.0.tenant_id),
            path.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(
        livestock
            .into_iter()
            .map(LivestockDto::from)
            .collect::<Vec<_>>(),
    ))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/livestock")
            .route("", web::get().to(list_livestock))
            .route("", web::post().to(create_livestock))
            .route("/{id}", web::get().to(get_livestock))
            .route("/{id}", web::put().to(update_livestock))
            .route("/{id}", web::delete().to(delete_livestock))
            .route("/by-plot/{plot_id}", web::get().to(list_livestock_by_plot))
            .route("/by-herd/{herd_id}", web::get().to(list_livestock_by_herd)),
    );
}
