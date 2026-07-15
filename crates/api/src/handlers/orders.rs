use crate::AppState;
use crate::dto::{
    CreateOrderDto, ErrorResponse, OrderDto, PaginatedOrderResponse, PaginatedResponseDto,
    UpdateOrderDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::order::MyTask;
use agrocore_domain::entities::workforce::CreateWorkLogDto;
#[allow(unused_imports)]
use agrocore_domain::repositories::WorkerTaskStatusRepository;
use agrocore_domain::services::workflow::WorkflowService;
use agrocore_messaging::{Event, GlobalEvent};
use agrocore_shared::SharedError;
use chrono::Utc;
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/v1/orders",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List orders", body = PaginatedOrderResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn list_orders(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listing orders for tenant: {}", auth.0.tenant_id);
    let result = state
        .db
        .order_repo()
        .find_all_visible(auth.0.tenant_id, query.0, auth.0.user_id, &auth.roles())
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(OrderDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders/{id}",
    responses(
        (status = 200, description = "Order details", body = OrderDto),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn get_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let order_id = *path;
    tracing::info!(
        "Getting order {} for tenant: {}",
        order_id,
        auth.0.tenant_id
    );
    let order = state
        .db
        .order_repo()
        .find_by_id_visible(auth.0.tenant_id, order_id, auth.0.user_id, &auth.roles())
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;
    Ok(HttpResponse::Ok().json(OrderDto::from(order)))
}

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    request_body = CreateOrderDto,
    responses(
        (status = 201, description = "Order created", body = OrderDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn create_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateOrderDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    tracing::info!("Creating order for tenant: {}", auth.0.tenant_id);
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let o = state
        .db
        .order_repo()
        .create(auth.0.tenant_id, dto.0.into(), auth.0.user_id)
        .await?;
    let event = Event::new("api".into(), GlobalEvent::OrderCreated(o.clone()));
    let _ = state.messaging.publish("events.orders", &event).await;
    Ok(HttpResponse::Created().json(OrderDto::from(o)))
}

#[utoipa::path(
    put,
    path = "/api/v1/orders/{id}",
    request_body = UpdateOrderDto,
    responses(
        (status = 200, description = "Order updated", body = OrderDto),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn update_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateOrderDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let order_id = *path;
    tracing::info!(
        "Updating order {} for tenant: {}",
        order_id,
        auth.0.tenant_id
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let o = state
        .db
        .order_repo()
        .update(auth.0.tenant_id, order_id, dto.0.into(), auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;
    let event = Event::new("api".into(), GlobalEvent::OrderUpdated(o.clone()));
    let _ = state.messaging.publish("events.orders", &event).await;
    Ok(HttpResponse::Ok().json(OrderDto::from(o)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/orders/{id}",
    responses(
        (status = 200, description = "Order deleted"),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn delete_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let order_id = *path;
    tracing::info!(
        "Deleting order {} for tenant: {}",
        order_id,
        auth.0.tenant_id
    );
    if state
        .db
        .order_repo()
        .delete(auth.0.tenant_id, order_id)
        .await?
    {
        let event = Event::new("api".into(), GlobalEvent::OrderDeleted(order_id));
        let _ = state.messaging.publish("events.orders", &event).await;
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Order not found".into()).into())
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/orders/{id}/complete",
    responses(
        (status = 200, description = "Order completed", body = OrderDto),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 400, description = "Order cannot be completed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn complete_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let order_id = *path;
    let tenant_id = auth.0.tenant_id;

    // 1. Fetch current order with visibility check (prevents unauthorized access)
    let mut order = state
        .db
        .order_repo()
        .find_by_id_visible(tenant_id, order_id, auth.0.user_id, &auth.roles())
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;

    let now = Utc::now();
    let duration_minutes = order.complete_at(now);

    if duration_minutes.is_none() {
        return Err(SharedError::Validation(format!(
            "Order {} cannot be completed from current state",
            order_id
        ))
        .into());
    }

    // 3. Persist change (reuse update logic from repo)
    let update_dto = agrocore_domain::entities::order::UpdateOrderDto {
        status: Some(order.status.clone()),
        planned_date: order.planned_date,
        completed_at: order.completed_at,
        last_completed_at: order.last_completed_at,
        recurrence: order.recurrence.clone(),
        execution_policy: order.execution_policy.clone(),
        automation_state: order.automation_state.clone(),
        started_at: order.started_at,
        ..Default::default()
    };

    let updated = state
        .db
        .order_repo()
        .update(tenant_id, order_id, update_dto, auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;

    let worker_id = match state
        .db
        .worker_repo()
        .find_by_user_id(tenant_id, auth.0.user_id)
        .await
    {
        Ok(Some(worker)) => worker.id,
        Ok(None) => auth.0.user_id,
        Err(e) => {
            tracing::warn!("Failed to resolve worker for worklog: {}", e);
            auth.0.user_id
        }
    };
    let worklog = CreateWorkLogDto {
        worker_id,
        date: now,
        hours_worked: duration_minutes.unwrap_or(0) as f64 / 60.0,
        overtime_hours: 0.0,
        rest_period_hours: 0.0,
        task_description: updated.label.clone(),
        site_id: updated.site_ids.first().copied(),
        is_night_shift: false,
        breaks_taken: 0,
    };
    if let Err(e) = state.db.work_log_repo().create(tenant_id, worklog, auth.0.user_id).await {
        tracing::warn!(
            "Failed to create worklog for completed order {}: {}",
            order_id,
            e
        );
    }

    // 4. Process workflows
    let follow_ups = WorkflowService::process_status_transition(
        &updated,
        agrocore_domain::entities::OrderStatus::Completed,
    );
    for next_order_dto in follow_ups {
        if let Err(e) = state
            .db
            .order_repo()
            .create(tenant_id, next_order_dto, auth.0.user_id)
            .await
        {
            tracing::error!("Failed to create follow-up order: {}", e);
        }
    }

    let event = Event::new("api".into(), GlobalEvent::OrderUpdated(updated.clone()));
    let _ = state.messaging.publish("events.orders", &event).await;
    Ok(HttpResponse::Ok().json(OrderDto::from(updated)))
}

#[utoipa::path(
    post,
    path = "/api/v1/orders/{id}/start",
    responses(
        (status = 200, description = "Order started", body = OrderDto),
        (status = 404, description = "Order not found", body = ErrorResponse),
        (status = 400, description = "Order cannot be started", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn start_order(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let order_id = *path;
    let tenant_id = auth.0.tenant_id;

    // 1. Fetch current order with visibility check (prevents unauthorized access)
    let mut order = state
        .db
        .order_repo()
        .find_by_id_visible(tenant_id, order_id, auth.0.user_id, &auth.roles())
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;

    // 2. Execute domain logic
    if !order.start_at(Utc::now()) {
        return Err(SharedError::Validation(format!(
            "Order {} cannot be started from current state",
            order_id
        ))
        .into());
    }

    // 3. Persist change
    let update_dto = agrocore_domain::entities::order::UpdateOrderDto {
        status: Some(order.status),
        started_at: order.started_at,
        execution_policy: order.execution_policy.clone(),
        automation_state: order.automation_state.clone(),
        ..Default::default()
    };

    let updated = state
        .db
        .order_repo()
        .update(tenant_id, order_id, update_dto, auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;
    let event = Event::new("api".into(), GlobalEvent::OrderUpdated(updated.clone()));
    let _ = state.messaging.publish("events.orders", &event).await;
    Ok(HttpResponse::Ok().json(OrderDto::from(updated)))
}

#[utoipa::path(
    get,
    path = "/api/v1/orders/my-tasks",
    responses(
        (status = 200, description = "List my active orders", body = Vec<MyTask>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "orders",
    security(("bearer_auth" = []))
)]
pub async fn my_tasks(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listing active tasks for user: {}", auth.0.user_id);
    let tasks = state
        .db
        .order_repo()
        .find_my_tasks(auth.0.tenant_id, auth.0.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(tasks))
}

// =============================================================================
// Worker Task Status - separate status per worker for multi-worker tasks
// =============================================================================

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{id}/start-for-worker",
    responses(
        (status = 200, description = "Worker task status updated", body = String),
        (status = 404, description = "Task not found or not assigned", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn start_task_for_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let task_id = *path;
    let tenant_id = auth.0.tenant_id;
    let worker_id = auth.0.user_id;

    // Find existing status, create if not exists
    let status = state
        .db
        .worker_task_status_repo()
        .find_by_task_and_worker(tenant_id, task_id, worker_id)
        .await?;

    match status {
        Some(_) => {
            // Update existing to Started
            state
                .db
                .worker_task_status_repo()
                .update_status(
                    tenant_id,
                    task_id,
                    worker_id,
                    agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Started,
                )
                .await?;
            Ok(HttpResponse::Ok().json(serde_json::json!({"status": "started"})))
        }
        None => {
            // Create new status entry
            let dto = agrocore_domain::entities::worker_task_status::CreateWorkerTaskStatusDto {
                task_id,
                worker_id,
                tenant_id,
            };
            state
                .db
                .worker_task_status_repo()
                .create(tenant_id, dto)
                .await?;
            Ok(HttpResponse::Ok().json(serde_json::json!({"status": "created"})))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{id}/stop-for-worker",
    responses(
        (status = 200, description = "Worker task status updated", body = String),
        (status = 404, description = "Task not found or not assigned", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn stop_task_for_worker(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let task_id = *path;
    let tenant_id = auth.0.tenant_id;
    let worker_id = auth.0.user_id;

    state
        .db
        .worker_task_status_repo()
        .update_status(
            tenant_id,
            task_id,
            worker_id,
            agrocore_domain::entities::worker_task_status::WorkerTaskStatusType::Stopped,
        )
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "stopped"})))
}
