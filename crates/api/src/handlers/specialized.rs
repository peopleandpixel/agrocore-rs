use actix_web::{web, HttpResponse, Responder};
use crate::dto::{ErrorResponse, PaginatedResponseDto};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/vineyards")
            .route(web::get().to(list_vineyards))
    ).service(
        web::resource("/olive-groves")
            .route(web::get().to(list_olive_groves))
    );
}

pub async fn list_vineyards(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state.db.vineyard_repo().find_all(auth.0.tenant_id.into(), query.0).await {
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

pub async fn list_olive_groves(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state.db.olive_grove_repo().find_all(auth.0.tenant_id.into(), query.0).await {
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
