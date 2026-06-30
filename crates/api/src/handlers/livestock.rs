use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use agrocore_domain::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use agrocore_domain::repositories::AnimalRepository;
use crate::AppState;
use crate::dto::{ErrorResponse, PaginatedAnimalResponse};
use crate::middleware::AuthExtractor as AuthUser;

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
) -> impl Responder {
    match state.db.animal_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.animal_repo().create(auth.0.tenant_id.into(), dto.0, auth.0.user_id).await {
        Ok(animal) => HttpResponse::Created().json(animal),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.animal_repo().find_by_id(auth.0.tenant_id.into(), *path).await {
        Ok(Some(animal)) => HttpResponse::Ok().json(animal),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Animal not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.animal_repo().update(auth.0.tenant_id.into(), *path, dto.0, auth.0.user_id).await {
        Ok(Some(animal)) => HttpResponse::Ok().json(animal),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Animal not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.animal_repo().delete(auth.0.tenant_id.into(), *path).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Animal not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
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
) -> impl Responder {
    match state.db.animal_repo().add_treatment(auth.0.tenant_id.into(), *path, dto.0).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Animal not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
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
) -> impl Responder {
    match state.db.animal_repo().add_grazing_record(auth.0.tenant_id.into(), *path, dto.0).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Animal not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/animals")
            .route("", web::get().to(list_animals))
            .route("", web::post().to(create_animal))
            .route("/{id}", web::get().to(get_animal))
            .route("/{id}", web::put().to(update_animal))
            .route("/{id}", web::delete().to(delete_animal))
            .route("/{id}/treatments", web::post().to(add_treatment))
            .route("/{id}/grazing", web::post().to(add_grazing))
    );
}
