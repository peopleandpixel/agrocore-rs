use actix_web::{web, HttpResponse, Responder};
use crate::dto::{ErrorResponse, PaginatedResponseDto};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/water/sources")
            .route(web::get().to(list_water_sources))
    ).service(
        web::resource("/water/usage")
            .route(web::get().to(list_water_usage))
    );
}

pub async fn list_water_sources(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state.db.water_source_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

pub async fn list_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state.db.water_usage_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}
