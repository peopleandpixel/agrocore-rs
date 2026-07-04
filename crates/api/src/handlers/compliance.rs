use crate::dto::{ErrorResponse, PaginatedResponseDto};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use agrocore_domain::entities::compliance::CreateComplianceChecklistDto;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/compliance/checklists")
            .route(web::get().to(list_checklists))
            .route(web::post().to(create_checklist)),
    )
    .service(web::resource("/compliance/fertilizer").route(web::get().to(list_fertilizer_records)))
    .service(
        web::resource("/compliance/plant-protection")
            .route(web::get().to(list_plant_protection_records)),
    );
}

pub async fn list_checklists(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state
        .db
        .compliance_checklist_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await
    {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "internal".into(),
            message: e.to_string(),
        }),
    }
}

pub async fn list_fertilizer_records(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state
        .db
        .fertilizer_record_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await
    {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "internal".into(),
            message: e.to_string(),
        }),
    }
}

pub async fn list_plant_protection_records(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state
        .db
        .plant_protection_record_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await
    {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "internal".into(),
            message: e.to_string(),
        }),
    }
}

pub async fn create_checklist(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateComplianceChecklistDto>,
) -> impl Responder {
    match state
        .db
        .compliance_checklist_repo()
        .create(auth.0.tenant_id, dto.into_inner())
        .await
    {
        Ok(checklist) => HttpResponse::Created().json(checklist),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "internal".into(),
            message: e.to_string(),
        }),
    }
}
