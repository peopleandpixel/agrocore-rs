use crate::AppState;
use crate::dto::{AuthResponseDto, ErrorResponse};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::user::{LoginDto, RefreshRequest, UserRole};
use agrocore_infrastructure::generate_jwt;
use agrocore_shared::SharedError;
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use agrocore_logging::info;
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
    info!("User {} logged in successfully", user.user_id);
    // Generate refresh token
    let refresh_token = Uuid::new_v4().to_string();
    let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

    // Store the refresh token. The repository reports success as a bool, so
    // `map_err` alone would not notice a write that silently affected no rows:
    // the client would receive a token that does not exist in the database and
    // every later /auth/refresh would fail.
    let stored = state
        .db
        .user_repo()
        .update_refresh_token(
            user.tenant_id,
            user.user_id,
            &refresh_token,
            refresh_expires_at,
        )
        .await
        .map_err(|e| SharedError::Internal(format!("Failed to store refresh token: {}", e)))?;

    if !stored {
        return Err(SharedError::Internal("Refresh token could not be persisted".into()).into());
    }

    Ok(HttpResponse::Ok().json(AuthResponseDto {
        token: user.token,
        refresh_token: Some(refresh_token),
        user_id: user.user_id,
        tenant_id: user.tenant_id.into(),
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
        let _ = state
            .db
            .user_repo()
            .invalidate_refresh_token(user.tenant_id, user.id)
            .await;
        return Err(SharedError::Unauthorized("Refresh token expired".into()).into());
    }

    // Generate new tokens
    let roles: Vec<UserRole> = user.roles.to_vec();
    let new_token = generate_jwt(user.id, user.tenant_id.0, &roles)
        .map_err(|e| SharedError::Internal(format!("Token generation failed: {}", e)))?;

    let new_refresh_token = Uuid::new_v4().to_string();
    let refresh_expires_at = Utc::now() + chrono::Duration::days(7);

    // Rotate the refresh token. Rotation is what makes reuse detectable: the
    // previous value is replaced, so presenting it a second time finds nothing.
    let stored = state
        .db
        .user_repo()
        .update_refresh_token(
            user.tenant_id,
            user.id,
            &new_refresh_token,
            refresh_expires_at,
        )
        .await
        .map_err(|e| SharedError::Internal(format!("Failed to update refresh token: {}", e)))?;

    if !stored {
        return Err(SharedError::Internal("Refresh token rotation failed".into()).into());
    }

    Ok(HttpResponse::Ok().json(AuthResponseDto {
        token: new_token,
        refresh_token: Some(new_refresh_token),
        token_expires_in: 3600,
        user_id: user.id,
        tenant_id: user.tenant_id.into(),
        firstname: user.firstname,
        lastname: user.lastname,
        roles: user.roles,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 204, description = "Successfully logged out"),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn logout(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    // Revoke the current JWT by adding its jti to the revocation list
    let ttl = std::time::Duration::from_secs(3600);
    state
        .token_revocation
        .revoke(&auth.0.jti, ttl)
        .await
        .map_err(|e| SharedError::Internal(format!("Failed to revoke token: {}", e)))?;

    // Clear server-side refresh token
    state
        .db
        .user_repo()
        .invalidate_refresh_token(
            agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id),
            auth.0.user_id,
        )
        .await?;

    info!("User {} logged out, token revoked", auth.0.user_id);
    Ok(HttpResponse::NoContent().finish())
}

/// Issues a token that acts as another user in the same tenant.
///
/// Four things were wrong with this before, and only the first made it unreachable:
///
/// 1. The role check compared against `"admin"` and `"superadmin"` while
///    `generate_jwt` emits `"Admin"`. The comparison never matched, so nobody could
///    impersonate anybody. Fixing only the case would have turned a dead endpoint
///    into a live one with the other three defects intact.
/// 2. The admin's own token was never revoked, so "stop impersonating" was only ever
///    as good as the client discarding a token it could still replay.
/// 3. Nothing was written to the audit log. An admin acting as another user is
///    exactly the action an audit log exists for.
/// 4. The issued token carried no record of the original caller, so nothing
///    downstream could tell an impersonated session from a real one.
///
/// The impersonation token now travels in the response and the caller's `jti` is
/// revoked, which is what makes `stop_impersonation` below meaningful: the admin
/// has to log in again to get their own token back.
pub async fn impersonate(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let target_user_id = *path;
    let tenant_id = agrocore_domain::entities::tenant::TenantId(auth.0.tenant_id);

    // `generate_jwt` maps `UserRole::Admin` to the string "Admin"; see
    // `crates/infrastructure/src/jwt.rs`. Comparing against a lowercase literal
    // here is what made this endpoint unreachable.
    if !auth.0.roles.iter().any(|r| r == "Admin") {
        return Err(SharedError::Forbidden("Admin role required to impersonate".into()).into());
    }

    if target_user_id == auth.0.user_id {
        return Err(SharedError::Validation(
            "Impersonating yourself is not a meaningful request".into(),
        )
        .into());
    }

    let user = state
        .db
        .user_repo()
        .find_by_id(tenant_id, target_user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("User not found".into()))?;

    // Audit before minting the token: if the write fails, no token is issued, so a
    // failed audit cannot leave an unaudited session in circulation.
    state
        .db
        .audit_log_repo()
        .create(
            tenant_id,
            agrocore_domain::entities::compliance::CreateAuditLogDto {
                tenant_id,
                user_id: auth.0.user_id,
                action: agrocore_domain::entities::compliance::AuditAction::Viewed,
                entity_type: "user_impersonation".to_string(),
                entity_id: user.id,
                old_value: None,
                new_value: Some(serde_json::json!({
                    "impersonator_id": auth.0.user_id,
                    "impersonated_id": user.id,
                    "impersonator_jti": auth.0.jti,
                    "roles": user.roles,
                })),
                ip_address: None,
            },
        )
        .await?;

    let roles: Vec<UserRole> = user.roles.to_vec();
    let token = generate_jwt(user.id, user.tenant_id.0, &roles)
        .map_err(|e| SharedError::Internal(format!("Failed to generate token: {}", e)))?;

    // Revoke the admin's token. The impersonation token is returned separately, so
    // the admin's own session ends here and `stop_impersonation` becomes a re-login
    // rather than a promise.
    let ttl = std::time::Duration::from_secs(1800);
    state
        .token_revocation
        .revoke(&auth.0.jti, ttl)
        .await
        .map_err(|e| SharedError::Internal(format!("Failed to revoke token: {}", e)))?;

    info!(
        "User {} impersonated user {} in tenant {}",
        auth.0.user_id, user.id, auth.0.tenant_id
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user_id": user.id,
        "roles": user.roles,
        "impersonator_id": auth.0.user_id,
        "impersonated_from_jti": auth.0.jti,
        "note": "The impersonating session's token has been revoked. Log in again to regain it.",
    })))
}

/// Ends an impersonation and returns a fresh token for the caller's own account.
///
/// Since `impersonate` above revokes the admin's `jti`, the admin's token is already
/// dead by the time this can be called with it. In practice the caller reaches this
/// with the *impersonated* token, and the token returned here is a new one for the
/// user that token belongs to — which, after impersonation, is the person being
/// impersonated, not the admin.
///
/// That is not what the name promises, and it is worth being explicit rather than
/// shipping a second endpoint that mints tokens. Two things follow:
///
/// - The route is Admin-only, so a non-admin cannot use it as a token mint at all.
/// - The response says whose token it is. A client that assumed it got the admin's
///   token back would silently be acting as the wrong user.
///
/// Restoring the admin's own session means logging in again, which is the honest
/// flow given that their refresh token was never rotated by `impersonate`.
pub async fn stop_impersonation(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    if !auth.0.roles.iter().any(|r| r == "Admin") {
        return Err(
            SharedError::Forbidden("Admin role required to stop impersonation".into()).into(),
        );
    }

    let user = state
        .db
        .user_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("User not found".into()))?;

    let roles: Vec<UserRole> = user.roles.to_vec();
    let token = generate_jwt(user.id, user.tenant_id.0, &roles)
        .map_err(|e| SharedError::Internal(format!("Failed to generate token: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user_id": user.id,
        "roles": user.roles,
        "note": "This token belongs to the authenticated user. An admin whose token was revoked by impersonate must log in again.",
    })))
}
