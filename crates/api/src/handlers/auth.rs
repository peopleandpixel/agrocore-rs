use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use validator::Validate;
use utoipa::ToSchema;
use agrocore_domain::entities::user::LoginDto;
use crate::dto::{AuthResponseDto, ErrorResponse};
use crate::AppState;

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponseDto),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    state: web::Data<AppState>,
    dto: web::Json<LoginRequest>,
) -> impl Responder {
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Login validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation".into(),
            message: e.to_string(),
        });
    }
    match state.db.user_repo().authenticate(LoginDto { email: dto.email.clone(), password: dto.password.clone() }).await {
        Ok(user) => {
            tracing::info!("User {} logged in successfully", user.user_id);
            HttpResponse::Ok().json(AuthResponseDto {
                token: user.token,
                user_id: user.user_id,
                tenant_id: user.tenant_id,
                firstname: user.firstname,
                lastname: user.lastname,
                roles: user.roles,
            })
        }
        Err(e) => {
            tracing::error!("Authentication failed: {}", e);
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "auth".into(),
                message: e.to_string(),
            })
        }
    }
}
