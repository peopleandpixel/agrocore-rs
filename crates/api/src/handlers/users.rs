use crate::AppState;
use crate::dto::{
    ErrorResponse, PaginatedResponseDto, PaginatedUserResponse, UserDto,
    user::{CreateUserDto, UpdateUserDto},
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_messaging::{Event, GlobalEvent};
use agrocore_shared::SharedError;
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List users", body = PaginatedUserResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn list_users(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    tracing::info!(
        "Listing users for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .user_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(UserDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    responses(
        (status = 200, description = "User details", body = UserDto),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user_id = *path;

    // SECURITY: Worker darf nur eigenes Profil sehen
    if !auth.is_manager() && auth.0.user_id != user_id {
        return Err(
            SharedError::Forbidden("Workers can only view their own profile".into()).into(),
        );
    }

    tracing::info!(
        "Getting user {} for tenant: {}",
        user_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let u = state
        .db
        .user_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("User not found".into()))?;
    Ok(HttpResponse::Ok().json(UserDto::from(u)))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created", body = UserDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn create_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateUserDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    tracing::info!(
        "Creating user for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let u = state
        .db
        .user_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    let event = Event::new("api".into(), GlobalEvent::UserCreated(u.clone()));
    let _ = state.messaging.publish("events.users", &event).await;
    Ok(HttpResponse::Created().json(UserDto::from(u)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated", body = UserDto),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn update_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateUserDto>,
) -> Result<HttpResponse, ApiError> {
    let user_id = *path;
    if let Err(e) = auth.require_admin()
        && auth.0.user_id != user_id
    {
        return Err(e.into());
    }
    tracing::info!(
        "Updating user {} for tenant: {}",
        user_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let u = state
        .db
        .user_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            user_id,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("User not found".into()))?;
    let event = Event::new("api".into(), GlobalEvent::UserUpdated(u.clone()));
    let _ = state.messaging.publish("events.users", &event).await;
    Ok(HttpResponse::Ok().json(UserDto::from(u)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    auth.require_admin()?;
    let user_id = *path;
    tracing::info!(
        "Deleting user {} for tenant: {}",
        user_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    if state
        .db
        .user_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), user_id)
        .await?
    {
        let event = Event::new("api".into(), GlobalEvent::UserDeleted(user_id));
        let _ = state.messaging.publish("events.users", &event).await;
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("User not found".into()).into())
    }
}
