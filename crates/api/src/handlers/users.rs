use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use crate::dto::{UserDto, CreateUserDto, UpdateUserDto, ErrorResponse, PaginatedResponseDto, PaginatedUserResponse};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

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
) -> impl Responder {
    tracing::info!("Listing users for tenant: {}", auth.0.tenant_id);
    match state.db.user_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(UserDto::from).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal".into(),
                message: e.to_string(),
            })
        },
    }
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
) -> impl Responder {
    let user_id = *path;
    tracing::info!("Getting user {} for tenant: {}", user_id, auth.0.tenant_id);
    match state.db.user_repo().find_by_id(auth.0.tenant_id.into(), user_id).await {
        Ok(Some(u)) => HttpResponse::Ok().json(UserDto::from(u)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "User not found".into() }),
        Err(e) => {
            tracing::error!("Failed to get user {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    tracing::info!("Creating user for tenant: {}", auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("User validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.user_repo().create(auth.0.tenant_id.into(), dto.0.into()).await {
        Ok(u) => HttpResponse::Created().json(UserDto::from(u)),
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let user_id = *path;
    tracing::info!("Updating user {} for tenant: {}", user_id, auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("User update validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.user_repo().update(auth.0.tenant_id.into(), user_id, dto.0.into()).await {
        Ok(Some(u)) => HttpResponse::Ok().json(UserDto::from(u)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "User not found".into() }),
        Err(e) => {
            tracing::error!("Failed to update user {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let user_id = *path;
    tracing::info!("Deleting user {} for tenant: {}", user_id, auth.0.tenant_id);
    match state.db.user_repo().delete(auth.0.tenant_id.into(), user_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "User not found".into() }),
        Err(e) => {
            tracing::error!("Failed to delete user {}: {}", user_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}
