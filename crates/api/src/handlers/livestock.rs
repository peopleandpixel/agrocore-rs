use crate::AppState;
use crate::dto::PaginatedAnimalResponse;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::livestock::{
    Animal, CreateAnimalDto, GrazingRecord, TreatmentRecord, UpdateAnimalDto,
};
#[allow(unused_imports)]
use agrocore_domain::repositories::AnimalRepository;
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/animals",
    responses(
        (status = 200, description = "List animals", body = PaginatedAnimalResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_animals(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .animal_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    path = "/animals",
    request_body = CreateAnimalDto,
    responses(
        (status = 201, description = "Animal created", body = Animal),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_animal(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateAnimalDto>,
) -> Result<HttpResponse, ApiError> {
    let animal = state
        .db
        .animal_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0,
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(animal))
}

#[utoipa::path(
    get,
    path = "/animals/{id}",
    responses(
        (status = 200, description = "Animal details", body = Animal),
        (status = 404, description = "Animal not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_animal(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let roles = auth.roles();
    let animal = state
        .db
        .animal_repo()
        .find_by_id_visible(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            auth.0.user_id,
            &roles,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Animal not found".into()))?;
    Ok(HttpResponse::Ok().json(animal))
}

#[utoipa::path(
    put,
    path = "/animals/{id}",
    request_body = UpdateAnimalDto,
    responses(
        (status = 200, description = "Animal updated", body = Animal),
        (status = 404, description = "Animal not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_animal(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateAnimalDto>,
) -> Result<HttpResponse, ApiError> {
    let animal = state
        .db
        .animal_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0,
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Animal not found".into()))?;
    Ok(HttpResponse::Ok().json(animal))
}

#[utoipa::path(
    delete,
    path = "/animals/{id}",
    responses(
        (status = 200, description = "Animal deleted"),
        (status = 404, description = "Animal not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_animal(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .animal_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Animal not found".into()).into())
    }
}

#[utoipa::path(
    post,
    path = "/animals/{id}/treatments",
    request_body = TreatmentRecord,
    responses(
        (status = 200, description = "Treatment added"),
        (status = 404, description = "Animal not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_treatment(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<TreatmentRecord>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .animal_repo()
        .add_treatment(agrocore_domain::TenantId(auth.0.tenant_id), *path, dto.0)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
    } else {
        Err(SharedError::NotFound("Animal not found".into()).into())
    }
}

#[utoipa::path(
    post,
    path = "/animals/{id}/grazing",
    request_body = GrazingRecord,
    responses(
        (status = 200, description = "Grazing record added"),
        (status = 404, description = "Animal not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn add_grazing(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<GrazingRecord>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .animal_repo()
        .add_grazing_record(agrocore_domain::TenantId(auth.0.tenant_id), *path, dto.0)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
    } else {
        Err(SharedError::NotFound("Animal not found".into()).into())
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/livestock")
            .route("/animals", web::get().to(list_animals))
            .route("/animals", web::post().to(create_animal))
            .route("/animals/{id}", web::get().to(get_animal))
            .route("/animals/{id}", web::put().to(update_animal))
            .route("/animals/{id}", web::delete().to(delete_animal))
            .route("/animals/{id}/treatments", web::post().to(add_treatment))
            .route("/animals/{id}/grazing", web::post().to(add_grazing)),
    );
}
