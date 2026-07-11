use crate::AppState;
use crate::dto::{AuthResponseDto, ErrorResponse};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::user::{LoginDto, RefreshRequest};
use agrocore_infrastructure::generate_jwt;
use agrocore_shared::SharedError;
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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
) -> Result<HttpResponse, ApiError> {
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let user = state
        .db
        .user_repo()
        .authenticate(LoginDto {
            email: dto.email.clone(),
            password: dto.password.clone(),
        })
        .await?;
    tracing::info!("User {} logged in successfully", user.user_id);
    // Generate refresh token
    let refresh_token = Uuid::new_v4().to_string();
    let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

    // Update user with refresh token
    let _ = state
        .db
        .user_repo()
        .update_refresh_token(user.user_id, &refresh_token, refresh_expires_at)
        .await;

    Ok(HttpResponse::Ok().json(AuthResponseDto {
        token: user.token,
        refresh_token: Some(refresh_token),
        user_id: user.user_id,
        tenant_id: user.tenant_id,
        firstname: user.firstname,
        lastname: user.lastname,
        roles: user.roles,
        token_expires_in: 3600,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponseDto),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse)
    ),
    security(
        (), // No auth header needed - refresh_token in body
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    state: web::Data<AppState>,
    dto: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let refresh_token = dto.0.refresh_token;

    // Find user by refresh token
    let user = state
        .db
        .user_repo()
        .find_by_refresh_token(&refresh_token)
        .await?
        .ok_or_else(|| SharedError::Unauthorized("Invalid refresh token".into()))?;

    // Check if refresh token is expired
    if let Some(expires_at) = user.refresh_token_expires_at
        && expires_at < Utc::now()
    {
        // Invalidate expired token
        let _ = state.db.user_repo().invalidate_refresh_token(user.id).await;
        return Err(SharedError::Unauthorized("Refresh token expired".into()).into());
    }

    // Generate new tokens
    let new_token = generate_jwt(&user)
        .map_err(|e| SharedError::Internal(format!("Token generation failed: {}", e)))?;

    let new_refresh_token = Uuid::new_v4().to_string();
    let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

    // Update refresh token in database
    let _ = state
        .db
        .user_repo()
        .update_refresh_token(user.id, &new_refresh_token, refresh_expires_at)
        .await;

    Ok(HttpResponse::Ok().json(AuthResponseDto {
        token: new_token,
        refresh_token: Some(new_refresh_token),
        token_expires_in: 3600,
        user_id: user.id,
        tenant_id: user.tenant_id,
        firstname: user.firstname,
        lastname: user.lastname,
        roles: user.roles,
    }))
}
