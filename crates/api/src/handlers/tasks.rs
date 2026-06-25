use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use crate::dto::{TaskDataDto, CreateTaskDataDto, ErrorResponse, PaginatedResponseDto, PaginatedTaskResponse};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

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
) -> impl Responder {
    tracing::info!("Listing tasks for tenant: {}", auth.0.tenant_id);
    match state.db.task_data_repo().find_by_worker(auth.0.tenant_id.into(), auth.0.user_id, query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(TaskDataDto::from).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal".into(),
                message: e.to_string(),
            })
        },
    }
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
) -> impl Responder {
    let task_id = *path;
    tracing::info!("Getting task {} for tenant: {}", task_id, auth.0.tenant_id);
    match state.db.task_data_repo().find_by_id(auth.0.tenant_id.into(), task_id).await {
        Ok(Some(t)) => HttpResponse::Ok().json(TaskDataDto::from(t)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Task not found".into() }),
        Err(e) => {
            tracing::error!("Failed to get task {}: {}", task_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    tracing::info!("Creating task for tenant: {}", auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Task validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.task_data_repo().create(auth.0.tenant_id.into(), auth.0.user_id, dto.0.into()).await {
        Ok(t) => HttpResponse::Created().json(TaskDataDto::from(t)),
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let task_id = *path;
    tracing::info!("Updating task {} for tenant: {}", task_id, auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Task update validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.task_data_repo().update(auth.0.tenant_id.into(), task_id, dto.0.into()).await {
        Ok(Some(t)) => HttpResponse::Ok().json(TaskDataDto::from(t)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Task not found".into() }),
        Err(e) => {
            tracing::error!("Failed to update task {}: {}", task_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let task_id = *path;
    tracing::info!("Deleting task {} for tenant: {}", task_id, auth.0.tenant_id);
    match state.db.task_data_repo().delete(auth.0.tenant_id.into(), task_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Task not found".into() }),
        Err(e) => {
            tracing::error!("Failed to delete task {}: {}", task_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}
