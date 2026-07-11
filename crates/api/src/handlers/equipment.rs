use crate::AppState;
use crate::dto::{
    CreateEquipmentDto, EquipmentDto, ErrorResponse, PaginatedEquipmentResponse,
    PaginatedResponseDto, UpdateEquipmentDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::repositories::EquipmentRepository;
use agrocore_shared::SharedError;
use validator::Validate;

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
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Listing equipment for tenant: {}", auth.0.tenant_id);
    let result = state
        .db
        .equipment_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(EquipmentDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
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
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Getting equipment {} for tenant: {}",
        equipment_id,
        auth.0.tenant_id
    );
    let equipment = state
        .db
        .equipment_repo()
        .find_by_id(auth.0.tenant_id, equipment_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;
    Ok(HttpResponse::Ok().json(EquipmentDto::from(equipment)))
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
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Creating equipment for tenant: {}", auth.0.tenant_id);
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let equipment = state
        .db
        .equipment_repo()
        .create(auth.0.tenant_id, dto.0.into(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(EquipmentDto::from(equipment)))
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
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Updating equipment {} for tenant: {}",
        equipment_id,
        auth.0.tenant_id
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let equipment = state
        .db
        .equipment_repo()
        .update(auth.0.tenant_id, equipment_id, dto.0.into(), auth.0.user_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;
    Ok(HttpResponse::Ok().json(EquipmentDto::from(equipment)))
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
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Deleting equipment {} for tenant: {}",
        equipment_id,
        auth.0.tenant_id
    );
    if state
        .db
        .equipment_repo()
        .delete(auth.0.tenant_id, equipment_id)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Equipment not found".into()).into())
    }
}
