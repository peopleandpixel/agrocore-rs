use crate::AppState;
use crate::dto::{BreedDto, CreateBreedDto, PaginatedBreedResponse, UpdateBreedDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/breeds",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List breeds", body = PaginatedBreedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_breeds(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .breed_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedBreedResponse {
        data: result.data.into_iter().map(BreedDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/breeds",
    request_body = CreateBreedDto,
    responses(
        (status = 201, description = "Breed created", body = BreedDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_breed(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateBreedDto>,
) -> Result<HttpResponse, ApiError> {
    let breed = state
        .db
        .breed_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(BreedDto::from(breed)))
}

#[utoipa::path(
    get,
    path = "/breeds/{id}",
    responses(
        (status = 200, description = "Breed details", body = BreedDto),
        (status = 404, description = "Breed not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_breed(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let breed = state
        .db
        .breed_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Breed not found".into()))?;
    Ok(HttpResponse::Ok().json(BreedDto::from(breed)))
}

#[utoipa::path(
    put,
    path = "/breeds/{id}",
    request_body = UpdateBreedDto,
    responses(
        (status = 200, description = "Breed updated", body = BreedDto),
        (status = 404, description = "Breed not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_breed(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateBreedDto>,
) -> Result<HttpResponse, ApiError> {
    let breed = state
        .db
        .breed_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Breed not found".into()))?;
    Ok(HttpResponse::Ok().json(BreedDto::from(breed)))
}

#[utoipa::path(
    delete,
    path = "/breeds/{id}",
    responses(
        (status = 200, description = "Breed deleted"),
        (status = 404, description = "Breed not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_breed(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .breed_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Breed not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/breeds/by-species/{species}",
    responses(
        (status = 200, description = "List breeds by species", body = Vec<BreedDto>),
        (status = 404, description = "Species not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_breeds_by_species(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<agrocore_domain::entities::breed::Species>,
) -> Result<HttpResponse, ApiError> {
    let species = path.into_inner();
    let breeds = state
        .db
        .breed_repo()
        .find_by_species(agrocore_domain::TenantId(auth.0.tenant_id), species)
        .await?;
    Ok(HttpResponse::Ok().json(breeds.into_iter().map(BreedDto::from).collect::<Vec<_>>()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/breeds")
            .route("", web::get().to(list_breeds))
            .route("", web::post().to(create_breed))
            .route("/{id}", web::get().to(get_breed))
            .route("/{id}", web::put().to(update_breed))
            .route("/{id}", web::delete().to(delete_breed))
            .route(
                "/by-species/{species}",
                web::get().to(list_breeds_by_species),
            ),
    );
}
