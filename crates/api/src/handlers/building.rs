use crate::AppState;
use crate::dto::{BuildingDto, CreateBuildingDto, PaginatedBuildingResponse, UpdateBuildingDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::building::{
    Building, BuildingType, CreateBuildingDto as DomainCreateBuildingDto,
    UpdateBuildingDto as DomainUpdateBuildingDto,
};
use agrocore_domain::repositories::BuildingRepository;
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/buildings",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List buildings", body = PaginatedBuildingResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_buildings(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .building_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedBuildingResponse {
        data: result.data.into_iter().map(BuildingDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/buildings",
    request_body = CreateBuildingDto,
    responses(
        (status = 201, description = "Building created", body = BuildingDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_building(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateBuildingDto>,
) -> Result<HttpResponse, ApiError> {
    let building = state
        .db
        .building_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(BuildingDto::from(building)))
}

#[utoipa::path(
    get,
    path = "/buildings/{id}",
    responses(
        (status = 200, description = "Building details", body = BuildingDto),
        (status = 404, description = "Building not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_building(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let building = state
        .db
        .building_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Building not found".into()))?;
    Ok(HttpResponse::Ok().json(BuildingDto::from(building)))
}

#[utoipa::path(
    put,
    path = "/buildings/{id}",
    request_body = UpdateBuildingDto,
    responses(
        (status = 200, description = "Building updated", body = BuildingDto),
        (status = 404, description = "Building not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_building(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateBuildingDto>,
) -> Result<HttpResponse, ApiError> {
    let building = state
        .db
        .building_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Building not found".into()))?;
    Ok(HttpResponse::Ok().json(BuildingDto::from(building)))
}

#[utoipa::path(
    delete,
    path = "/buildings/{id}",
    responses(
        (status = 200, description = "Building deleted"),
        (status = 404, description = "Building not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_building(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .building_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Building not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/buildings/by-plot/{plot_id}",
    params(
        ("plot_id" = Uuid, Path, description = "Plot UUID"),
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List buildings by plot", body = PaginatedBuildingResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_buildings_by_plot(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .building_repo()
        .find_by_plot(agrocore_domain::TenantId(auth.0.tenant_id), *path, query.0)
        .await?;

    let response = PaginatedBuildingResponse {
        data: result.data.into_iter().map(BuildingDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/buildings")
            .route("", web::get().to(list_buildings))
            .route("", web::post().to(create_building))
            .route("/{id}", web::get().to(get_building))
            .route("/{id}", web::put().to(update_building))
            .route("/{id}", web::delete().to(delete_building))
            .route("/by-plot/{plot_id}", web::get().to(list_buildings_by_plot)),
    );
}
