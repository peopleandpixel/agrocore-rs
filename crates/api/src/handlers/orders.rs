use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use crate::dto::{OrderDto, CreateOrderDto, UpdateOrderDto, ErrorResponse, PaginatedResponseDto, PaginatedOrderResponse};
use agrocore_domain::entities::order::MyTask;
use crate::middleware::AuthExtractor as AuthUser;
use agrocore_messaging::{Event, GlobalEvent};
use crate::AppState;

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
) -> impl Responder {
    tracing::info!("Listing orders for tenant: {}", auth.0.tenant_id);
    match state.db.order_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(OrderDto::from).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => {
            tracing::error!("Failed to list orders: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal".into(),
                message: e.to_string(),
            })
        },
    }
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
) -> impl Responder {
    let order_id = *path;
    tracing::info!("Getting order {} for tenant: {}", order_id, auth.0.tenant_id);
    match state.db.order_repo().find_by_id(auth.0.tenant_id.into(), order_id).await {
        Ok(Some(o)) => HttpResponse::Ok().json(OrderDto::from(o)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Order not found".into() }),
        Err(e) => {
            tracing::error!("Failed to get order {}: {}", order_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    tracing::info!("Creating order for tenant: {}", auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Order validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.order_repo().create(auth.0.tenant_id.into(), dto.0.into(), auth.0.user_id).await {
        Ok(o) => {
            let event = Event::new("api".into(), GlobalEvent::OrderCreated(o.clone()));
            let _ = state.messaging.publish("events.orders", &event).await;
            HttpResponse::Created().json(OrderDto::from(o))
        },
        Err(e) => {
            tracing::error!("Failed to create order: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let order_id = *path;
    tracing::info!("Updating order {} for tenant: {}", order_id, auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Order update validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.order_repo().update(auth.0.tenant_id.into(), order_id, dto.0.into(), auth.0.user_id).await {
        Ok(Some(o)) => {
            let event = Event::new("api".into(), GlobalEvent::OrderUpdated(o.clone()));
            let _ = state.messaging.publish("events.orders", &event).await;
            HttpResponse::Ok().json(OrderDto::from(o))
        },
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Order not found".into() }),
        Err(e) => {
            tracing::error!("Failed to update order {}: {}", order_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    let order_id = *path;
    tracing::info!("Deleting order {} for tenant: {}", order_id, auth.0.tenant_id);
    match state.db.order_repo().delete(auth.0.tenant_id.into(), order_id).await {
        Ok(true) => {
            let event = Event::new("api".into(), GlobalEvent::OrderDeleted(order_id));
            let _ = state.messaging.publish("events.orders", &event).await;
            HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
        },
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Order not found".into() }),
        Err(e) => {
            tracing::error!("Failed to delete order {}: {}", order_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
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
) -> impl Responder {
    tracing::info!("Listing active tasks for user: {}", auth.0.user_id);
    match state.db.order_repo().find_my_tasks(auth.0.tenant_id.into(), auth.0.user_id).await {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => {
            tracing::error!("Failed to list active tasks: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}
