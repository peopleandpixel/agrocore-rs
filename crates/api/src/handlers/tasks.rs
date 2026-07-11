use crate::AppState;
use crate::dto::{
    CreateTaskDataDto, ErrorResponse, PaginatedResponseDto, PaginatedTaskResponse, TaskDataDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List tasks", body = PaginatedTaskResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn list_tasks(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    tracing::info!("Listing tasks for tenant: {}", auth.0.tenant_id);
    let result = state
        .db
        .task_data_repo()
        .find_by_worker(auth.0.tenant_id, auth.0.user_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(TaskDataDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}",
    responses(
        (status = 200, description = "Task details", body = TaskDataDto),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let task_id = *path;
    tracing::info!("Getting task {} for tenant: {}", task_id, auth.0.tenant_id);
    let roles = auth.roles();
    let task = state
        .db
        .task_data_repo()
        .find_by_id_visible(auth.0.tenant_id, task_id, auth.0.user_id, &roles)
        .await?
        .ok_or_else(|| SharedError::NotFound("Task not found".into()))?;
    Ok(HttpResponse::Ok().json(TaskDataDto::from(task)))
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    request_body = CreateTaskDataDto,
    responses(
        (status = 201, description = "Task created", body = TaskDataDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn create_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateTaskDataDto>,
) -> Result<HttpResponse, ApiError> {
    // Workers should be able to create tasks (log their own work),
    // but managers are definitely allowed.
    tracing::info!("Creating task for tenant: {}", auth.0.tenant_id);
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let task = state
        .db
        .task_data_repo()
        .create(auth.0.tenant_id, auth.0.user_id, dto.0.into())
        .await?;
    Ok(HttpResponse::Created().json(TaskDataDto::from(task)))
}

#[utoipa::path(
    put,
    path = "/api/v1/tasks/{id}",
    request_body = CreateTaskDataDto,
    responses(
        (status = 200, description = "Task updated", body = TaskDataDto),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn update_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<CreateTaskDataDto>,
) -> Result<HttpResponse, ApiError> {
    let task_id = *path;
    tracing::info!("Updating task {} for tenant: {}", task_id, auth.0.tenant_id);
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let task = state
        .db
        .task_data_repo()
        .update(auth.0.tenant_id, task_id, dto.0.into())
        .await?
        .ok_or_else(|| SharedError::NotFound("Task not found".into()))?;
    Ok(HttpResponse::Ok().json(TaskDataDto::from(task)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    responses(
        (status = 200, description = "Task deleted"),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn delete_task(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let task_id = *path;
    tracing::info!("Deleting task {} for tenant: {}", task_id, auth.0.tenant_id);
    if state
        .db
        .task_data_repo()
        .delete(auth.0.tenant_id, task_id)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Task not found".into()).into())
    }
}
