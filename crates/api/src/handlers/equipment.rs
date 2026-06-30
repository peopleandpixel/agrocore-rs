use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use agrocore_domain::repositories::EquipmentRepository;
use crate::dto::{EquipmentDto, CreateEquipmentDto, UpdateEquipmentDto, ErrorResponse, PaginatedResponseDto, PaginatedEquipmentResponse};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/equipments",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List equipment", body = PaginatedEquipmentResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn list_equipments(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> impl Responder {
    tracing::info!("Listing equipment for tenant: {}", auth.0.tenant_id);
    match state.db.equipment_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data.into_iter().map(EquipmentDto::from).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => {
            tracing::error!("Failed to list equipment: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal".into(),
                message: e.to_string(),
            })
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}",
    responses(
        (status = 200, description = "Equipment details", body = EquipmentDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_equipment(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let equipment_id = *path;
    tracing::info!("Getting equipment {} for tenant: {}", equipment_id, auth.0.tenant_id);
    match state.db.equipment_repo().find_by_id(auth.0.tenant_id.into(), equipment_id).await {
        Ok(Some(equipment)) => HttpResponse::Ok().json(EquipmentDto::from(equipment)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Equipment not found".into() }),
        Err(e) => {
            tracing::error!("Failed to get equipment {}: {}", equipment_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/equipments",
    request_body = CreateEquipmentDto,
    responses(
        (status = 201, description = "Equipment created", body = EquipmentDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn create_equipment(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateEquipmentDto>,
) -> impl Responder {
    tracing::info!("Creating equipment for tenant: {}", auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Equipment validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.equipment_repo().create(auth.0.tenant_id.into(), dto.0.into(), auth.0.user_id).await {
        Ok(equipment) => {
            HttpResponse::Created().json(EquipmentDto::from(equipment))
        },
        Err(e) => {
            tracing::error!("Failed to create equipment: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/equipments/{id}",
    request_body = UpdateEquipmentDto,
    responses(
        (status = 200, description = "Equipment updated", body = EquipmentDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn update_equipment(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateEquipmentDto>,
) -> impl Responder {
    let equipment_id = *path;
    tracing::info!("Updating equipment {} for tenant: {}", equipment_id, auth.0.tenant_id);
    if let Err(e) = dto.0.validate() {
        tracing::warn!("Equipment update validation failed: {}", e);
        return HttpResponse::BadRequest().json(ErrorResponse { error: "validation".into(), message: e.to_string() });
    }
    match state.db.equipment_repo().update(auth.0.tenant_id.into(), equipment_id, dto.0.into(), auth.0.user_id).await {
        Ok(Some(equipment)) => {
            HttpResponse::Ok().json(EquipmentDto::from(equipment))
        },
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Equipment not found".into() }),
        Err(e) => {
            tracing::error!("Failed to update equipment {}: {}", equipment_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/equipments/{id}",
    responses(
        (status = 200, description = "Equipment deleted"),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn delete_equipment(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> impl Responder {
    let equipment_id = *path;
    tracing::info!("Deleting equipment {} for tenant: {}", equipment_id, auth.0.tenant_id);
    match state.db.equipment_repo().delete(auth.0.tenant_id.into(), equipment_id).await {
        Ok(true) => {
            HttpResponse::Ok().json(serde_json::json!({"deleted": true}))
        },
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Equipment not found".into() }),
        Err(e) => {
            tracing::error!("Failed to delete equipment {}: {}", equipment_id, e);
            HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() })
        },
    }
}
