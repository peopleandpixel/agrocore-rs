use crate::AppState;
use crate::dto::{CreateVarietyDto, PaginatedVarietyResponse, UpdateVarietyDto, VarietyDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/varieties",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List varieties", body = PaginatedVarietyResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_varieties(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .variety_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedVarietyResponse {
        data: result.data.into_iter().map(VarietyDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/varieties",
    request_body = CreateVarietyDto,
    responses(
        (status = 201, description = "Variety created", body = VarietyDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_variety(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateVarietyDto>,
) -> Result<HttpResponse, ApiError> {
    let variety = state
        .db
        .variety_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(VarietyDto::from(variety)))
}

#[utoipa::path(
    get,
    path = "/varieties/{id}",
    responses(
        (status = 200, description = "Variety details", body = VarietyDto),
        (status = 404, description = "Variety not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_variety(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let variety = state
        .db
        .variety_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Variety not found".into()))?;
    Ok(HttpResponse::Ok().json(VarietyDto::from(variety)))
}

#[utoipa::path(
    put,
    path = "/varieties/{id}",
    request_body = UpdateVarietyDto,
    responses(
        (status = 200, description = "Variety updated", body = VarietyDto),
        (status = 404, description = "Variety not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_variety(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateVarietyDto>,
) -> Result<HttpResponse, ApiError> {
    let variety = state
        .db
        .variety_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Variety not found".into()))?;
    Ok(HttpResponse::Ok().json(VarietyDto::from(variety)))
}

#[utoipa::path(
    delete,
    path = "/varieties/{id}",
    responses(
        (status = 200, description = "Variety deleted"),
        (status = 404, description = "Variety not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_variety(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .variety_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Variety not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/varieties/by-category/{category}",
    responses(
        (status = 200, description = "List varieties by category", body = Vec<VarietyDto>),
        (status = 404, description = "Category not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_varieties_by_category(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<agrocore_domain::entities::variety::VarietyCategory>,
) -> Result<HttpResponse, ApiError> {
    let category = path.into_inner();
    let varieties = state
        .db
        .variety_repo()
        .find_by_category(agrocore_domain::TenantId(auth.0.tenant_id), category)
        .await?;
    Ok(HttpResponse::Ok().json(
        varieties
            .into_iter()
            .map(VarietyDto::from)
            .collect::<Vec<_>>(),
    ))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/varieties")
            .route("", web::get().to(list_varieties))
            .route("", web::post().to(create_variety))
            .route("/{id}", web::get().to(get_variety))
            .route("/{id}", web::put().to(update_variety))
            .route("/{id}", web::delete().to(delete_variety))
            .route(
                "/by-category/{category}",
                web::get().to(list_varieties_by_category),
            ),
    );
}
