use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use validator::Validate;

use agrocore_domain::entities::user::LoginDto;

use crate::dto::*;
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/v1/health").route(web::get().to(health)))
        .service(web::resource("/api/v1/auth/login").route(web::post().to(login)))
        .service(
            web::resource("/api/v1/sites")
                .route(web::get().to(list_sites))
                .route(web::post().to(create_site))
        )
        .service(
            web::resource("/api/v1/sites/{id}")
                .route(web::get().to(get_site))
                .route(web::put().to(update_site))
                .route(web::delete().to(delete_site))
        )
        .service(
            web::resource("/api/v1/orders")
                .route(web::get().to(list_orders))
                .route(web::post().to(create_order))
        )
        .service(
            web::resource("/api/v1/orders/{id}")
                .route(web::get().to(get_order))
                .route(web::put().to(update_order))
                .route(web::delete().to(delete_order))
        )
        .service(web::resource("/api/v1/orders/my-tasks").route(web::get().to(my_tasks)))
        .service(
            web::resource("/api/v1/users")
                .route(web::get().to(list_users))
                .route(web::post().to(create_user))
        )
        .service(
            web::resource("/api/v1/users/{id}")
                .route(web::get().to(get_user))
                .route(web::put().to(update_user))
                .route(web::delete().to(delete_user))
        )
        .service(
            web::resource("/api/v1/tasks")
                .route(web::get().to(list_tasks))
                .route(web::post().to(create_task))
        )
        .service(
            web::resource("/api/v1/tasks/{id}")
                .route(web::get().to(get_task))
                .route(web::put().to(update_task))
                .route(web::delete().to(delete_task))
        );
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

async fn login(
    state: web::Data<AppState>,
    dto: web::Json<LoginRequest>,
) -> impl Responder {
    if let Err(e) = dto.0.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation".into(),
            message: e.to_string(),
        });
    }
    match state.db.user_repo().authenticate(LoginDto { email: dto.email.clone(), password: dto.password.clone() }).await {
        Ok(user) => {
            let roles: Vec<String> = user.roles.iter().map(|r| format!("{:?}", r)).collect();
            HttpResponse::Ok().json(AuthResponseDto {
                token: user.token,
                user_id: user.user_id,
                tenant_id: user.tenant_id,
                firstname: user.firstname,
                lastname: user.lastname,
                roles,
            })
        }
        Err(e) => HttpResponse::Unauthorized().json(ErrorResponse {
            error: "auth".into(),
            message: e.to_string(),
        }),
    }
}

async fn list_sites(
    state: web::Data<AppState>,
    _auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    match state.db.site_repo().find_all(uuid::Uuid::new_v4().into(), &query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(|s| SiteDto {
                id: s.id,
                tenant_id: s.tenant_id,
                label: s.label,
                site_type: format!("{:?}", s.site_type),
                crop_type: format!("{:?}", s.crop_type),
                variety: s.variety,
                area: s.area,
                bbch_stage: s.bbch_stage.map(|b| format!("{:?}", b)),
                soil_type: s.soil_type,
                slope: s.slope,
                altitude: s.altitude,
                organic: s.organic,
                is_active: s.is_active,
                created_at: s.created_at.to_rfc3339(),
                updated_at: s.updated_at.to_rfc3339(),
            }).collect(),
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

async fn get_site(
    state: web::Data<AppState>,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    match state.db.site_repo().find_by_id(uuid::Uuid::new_v4().into(), *path).await {
        Ok(Some(site)) => HttpResponse::Ok().json(SiteDto {
            id: site.id, tenant_id: site.tenant_id, label: site.label,
            site_type: format!("{:?}", site.site_type), crop_type: format!("{:?}", site.crop_type),
            variety: site.variety, area: site.area,
            bbch_stage: site.bbch_stage.map(|b| format!("{:?}", b)),
            soil_type: site.soil_type, slope: site.slope, altitude: site.altitude,
            organic: site.organic, is_active: site.is_active,
            created_at: site.created_at.to_rfc3339(), updated_at: site.updated_at.to_rfc3339(),
        }),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

async fn create_site(
    state: web::Data<AppState>,
    dto: web::Json<agrocore_domain::entities::site::CreateSiteDto>,
) -> impl Responder {
    if let Err(e) = dto.0.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.site_repo().create(uuid::Uuid::new_v4().into(), dto.0, uuid::Uuid::new_v4()).await {
        Ok(site) => HttpResponse::Created().json(SiteDto {
            id: site.id, tenant_id: site.tenant_id, label: site.label,
            site_type: format!("{:?}", site.site_type), crop_type: format!("{:?}", site.crop_type),
            variety: site.variety, area: site.area,
            bbch_stage: site.bbch_stage.map(|b| format!("{:?}", b)),
            soil_type: site.soil_type, slope: site.slope, altitude: site.altitude,
            organic: site.organic, is_active: site.is_active,
            created_at: site.created_at.to_rfc3339(), updated_at: site.updated_at.to_rfc3339(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

async fn update_site(
    state: web::Data<AppState>,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<agrocore_domain::entities::site::UpdateSiteDto>,
) -> impl Responder {
    if let Err(e) = dto.0.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.site_repo().update(uuid::Uuid::new_v4().into(), *path, dto.0, uuid::Uuid::new_v4()).await {
        Ok(Some(site)) => HttpResponse::Ok().json(SiteDto {
            id: site.id, tenant_id: site.tenant_id, label: site.label,
            site_type: format!("{:?}", site.site_type), crop_type: format!("{:?}", site.crop_type),
            variety: site.variety, area: site.area,
            bbch_stage: site.bbch_stage.map(|b| format!("{:?}", b)),
            soil_type: site.soil_type, slope: site.slope, altitude: site.altitude,
            organic: site.organic, is_active: site.is_active,
            created_at: site.created_at.to_rfc3339(), updated_at: site.updated_at.to_rfc3339(),
        }),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

async fn delete_site(
    state: web::Data<AppState>,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    match state.db.site_repo().delete(uuid::Uuid::new_v4().into(), *path).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Site not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}

async fn list_orders(state: web::Data<AppState>, _query: web::Query<agrocore_shared::Pagination>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"orders": []}))
}

async fn get_order(state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"order": {}}))
}

async fn create_order(_state: web::Data<AppState>, _dto: web::Json<agrocore_domain::entities::order::CreateOrderDto>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({"created": true}))
}

async fn update_order(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>, _dto: web::Json<agrocore_domain::entities::order::UpdateOrderDto>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"updated": true}))
}

async fn delete_order(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
}

async fn my_tasks(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"tasks": []}))
}

async fn list_users(state: web::Data<AppState>, _query: web::Query<agrocore_shared::Pagination>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"users": []}))
}

async fn get_user(state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"user": {}}))
}

async fn create_user(_state: web::Data<AppState>, _dto: web::Json<agrocore_domain::entities::user::CreateUserDto>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({"created": true}))
}

async fn update_user(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>, _dto: web::Json<agrocore_domain::entities::user::UpdateUserDto>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"updated": true}))
}

async fn delete_user(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
}

async fn list_tasks(state: web::Data<AppState>, _query: web::Query<agrocore_shared::Pagination>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"tasks": []}))
}

async fn get_task(state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"task": {}}))
}

async fn create_task(_state: web::Data<AppState>, _dto: web::Json<agrocore_domain::entities::task::CreateTaskDataDto>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({"created": true}))
}

async fn update_task(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>, _dto: web::Json<agrocore_domain::entities::task::CreateTaskDataDto>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"updated": true}))
}

async fn delete_task(_state: web::Data<AppState>, _path: web::Path<uuid::Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
}
