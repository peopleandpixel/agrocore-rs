use crate::AppState;
use crate::dto::{
    CreateEquipmentDto, CreateFuelConsumptionRequest, CreateUsageLogRequest, EquipmentDto,
    EquipmentFilterDto, ErrorResponse, FuelConsumptionDto, MaintenanceCostSummaryDto,
    MaintenanceLogDto, MaintenanceRecordDto, PaginatedEquipmentResponse, PaginatedResponseDto,
    UpdateEquipmentDto, UsageLogDto, UsageSummaryDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
#[allow(unused_imports)]
use agrocore_domain::repositories::EquipmentRepository;
use agrocore_shared::{Pagination, SharedError};
use chrono::Utc;
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
    tracing::info!(
        "Listing equipment for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .equipment_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
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
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let equipment = state
        .db
        .equipment_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
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
    tracing::info!(
        "Creating equipment for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let equipment = state
        .db
        .equipment_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
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
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let equipment = state
        .db
        .equipment_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            equipment_id,
            dto.0.into(),
            auth.0.user_id,
        )
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
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    if state
        .db
        .equipment_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Equipment not found".into()).into())
    }
}

/// Search and filter equipment with query parameters.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/search",
    params(
        ("search" = Option<String>, Query, description = "Full-text search on label and code"),
        ("equipment_type" = Option<String>, Query, description = "Filter by equipment type"),
        ("in_usage" = Option<bool>, Query, description = "Filter by in_usage status"),
        ("needs_maintenance" = Option<bool>, Query, description = "Only show equipment needing maintenance"),
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Filtered equipment list", body = PaginatedEquipmentResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn search_equipments(
    state: web::Data<AppState>,
    auth: AuthUser,
    filter: web::Query<EquipmentFilterDto>,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        "Searching equipment for tenant: {} with filters",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .equipment_repo()
        .find_all_filtered(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.0,
            filter.search.as_deref(),
            filter.equipment_type.as_deref(),
            filter.in_usage,
            filter.needs_maintenance,
        )
        .await?;

    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(EquipmentDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

/// List equipment that needs maintenance (next_maintenance_date <= now).
#[utoipa::path(
    get,
    path = "/api/v1/equipments/maintenance",
    responses(
        (status = 200, description = "Equipment needing maintenance", body = PaginatedEquipmentResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn list_maintenance_due(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!(
        "Listing maintenance-due equipment for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .equipment_repo()
        .find_maintenance_due(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(EquipmentDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

/// Record a maintenance event for equipment.
#[utoipa::path(
    post,
    path = "/api/v1/equipments/{id}/maintenance",
    request_body = MaintenanceRecordDto,
    responses(
        (status = 200, description = "Maintenance recorded", body = EquipmentDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn record_maintenance(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<MaintenanceRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Recording maintenance for equipment {} in tenant: {}",
        equipment_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let updated = state
        .db
        .equipment_repo()
        .record_maintenance(
            agrocore_domain::TenantId(auth.0.tenant_id),
            equipment_id,
            dto.0.hours,
            dto.0.note,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;

    Ok(HttpResponse::Ok().json(EquipmentDto::from(updated)))
}

/// Get maintenance log for a specific equipment.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}/maintenance",
    responses(
        (status = 200, description = "Maintenance log", body = Vec<MaintenanceLogDto>),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_equipment_maintenance_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    let log = state
        .db
        .equipment_repo()
        .get_maintenance_log(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?;
    Ok(HttpResponse::Ok().json(log))
}

/// Get maintenance cost summary for a specific equipment.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}/maintenance-cost-summary",
    responses(
        (status = 200, description = "Maintenance cost summary", body = MaintenanceCostSummaryDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_maintenance_cost_summary(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    let summary = state
        .db
        .equipment_repo()
        .get_maintenance_cost_summary(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;
    Ok(HttpResponse::Ok().json(summary))
}

/// Get fuel consumption history for a specific equipment.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}/fuel-consumption",
    responses(
        (status = 200, description = "Fuel consumption history", body = Vec<FuelConsumptionDto>),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_fuel_consumption(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Getting fuel consumption for equipment {} in tenant: {}",
        equipment_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let records = state
        .db
        .equipment_repo()
        .get_fuel_consumption(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?;
    Ok(HttpResponse::Ok().json(records))
}

/// Record a fuel consumption entry for equipment.
#[utoipa::path(
    post,
    path = "/api/v1/equipments/{id}/fuel-consumption",
    request_body = CreateFuelConsumptionRequest,
    responses(
        (status = 201, description = "Fuel consumption recorded", body = FuelConsumptionDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn record_fuel_consumption(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<CreateFuelConsumptionRequest>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Recording fuel consumption for equipment {} in tenant: {}",
        equipment_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let record = state
        .db
        .equipment_repo()
        .record_fuel_consumption(
            agrocore_domain::TenantId(auth.0.tenant_id),
            equipment_id,
            dto.0.liters,
            dto.0.cost_per_liter,
            dto.0.operation_type.as_deref(),
            dto.0.field_id,
            dto.0.hours_operated,
            dto.0.notes.as_deref(),
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;
    Ok(HttpResponse::Created().json(record))
}

/// Get usage log history for a specific equipment.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}/usage",
    responses(
        (status = 200, description = "Usage log history", body = Vec<UsageLogDto>),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_usage_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Getting usage log for equipment {} in tenant: {}",
        equipment_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let records = state
        .db
        .equipment_repo()
        .get_usage_log(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?;
    Ok(HttpResponse::Ok().json(records))
}

/// Get usage summary for a specific equipment.
#[utoipa::path(
    get,
    path = "/api/v1/equipments/{id}/usage-summary",
    responses(
        (status = 200, description = "Usage summary", body = UsageSummaryDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn get_usage_summary(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    let summary = state
        .db
        .equipment_repo()
        .get_usage_summary(agrocore_domain::TenantId(auth.0.tenant_id), equipment_id)
        .await?;
    Ok(HttpResponse::Ok().json(summary))
}

/// Record a usage log entry for equipment.
#[utoipa::path(
    post,
    path = "/api/v1/equipments/{id}/usage",
    request_body = CreateUsageLogRequest,
    responses(
        (status = 201, description = "Usage log recorded", body = UsageLogDto),
        (status = 404, description = "Equipment not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "equipment",
    security(("bearer_auth" = []))
)]
pub async fn record_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<CreateUsageLogRequest>,
) -> Result<HttpResponse, ApiError> {
    let equipment_id = *path;
    tracing::info!(
        "Recording usage for equipment {} in tenant: {}",
        equipment_id,
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let record = state
        .db
        .equipment_repo()
        .record_usage(
            agrocore_domain::TenantId(auth.0.tenant_id),
            equipment_id,
            dto.0.worker_id,
            dto.0.task_id,
            dto.0.operation_type.as_deref(),
            dto.0.started_at.unwrap_or_else(|| {
                chrono::DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339())
                    .unwrap()
                    .with_timezone(&Utc)
            }),
            dto.0
                .ended_at
                .map(|s| s.to_rfc3339().parse().unwrap_or(Utc::now())),
            dto.0.hours_operated.unwrap_or(0.0),
            dto.0.note.as_deref(),
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Equipment not found".into()))?;
    Ok(HttpResponse::Created().json(record))
}
