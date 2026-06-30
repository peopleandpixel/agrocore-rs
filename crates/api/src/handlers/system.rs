use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use crate::AppState;
use crate::dto::{ErrorResponse, CreateUserDto};
use agrocore_domain::entities::tenant::CreateTenantDto;
use agrocore_domain::entities::user::UserRole;

#[derive(Serialize, ToSchema)]
pub struct SystemStatusResponse {
    pub initialized: bool,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct InitialSetupRequest {
    pub admin: CreateUserDto,
    pub tenant: CreateTenantDto,
}

#[utoipa::path(
    get,
    path = "/api/v1/system/status",
    responses(
        (status = 200, description = "System status", body = SystemStatusResponse)
    ),
    tag = "system"
)]
pub async fn get_status(state: web::Data<AppState>) -> impl Responder {
    match state.db.user_repo().count_all().await {
        Ok(count) => HttpResponse::Ok().json(SystemStatusResponse {
            initialized: count > 0,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "database".into(),
            message: e.to_string(),
        }),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/system/setup",
    request_body = InitialSetupRequest,
    responses(
        (status = 201, description = "Initial setup completed"),
        (status = 400, description = "Invalid request or system already initialized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn initial_setup(
    state: web::Data<AppState>,
    dto: web::Json<InitialSetupRequest>,
) -> impl Responder {
    // 1. Check if already initialized
    match state.db.user_repo().count_all().await {
        Ok(count) if count > 0 => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "already_initialized".into(),
                message: "System is already initialized".into(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "database".into(),
                message: e.to_string(),
            });
        }
        _ => {}
    }

    if let Err(e) = dto.admin.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    if let Err(e) = dto.tenant.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }

    // 2. Create Tenant
    let tenant = match state.db.tenant_repo().create(dto.tenant.clone()).await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(ErrorResponse { error: "database".into(), message: e.to_string() }),
    };

    // 3. Create Admin User
    let mut admin_dto = dto.admin.clone();
    admin_dto.roles = Some(vec![UserRole::Admin]);
    
    match state.db.user_repo().create(tenant.id, admin_dto.into()).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "database".into(), message: e.to_string() }),
    }
}
