use crate::AppState;
use crate::dto::{
    ApplicatorLicenseDto, CreateApplicatorLicenseDto, CreateComplianceChecklistDto, ErrorResponse,
    PaginatedResponseDto, UpdateApplicatorLicenseDto, UpdateComplianceChecklistDto,
    UpdateFertilizerRecordDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::plant_protection::{
    CreateApplicatorLicenseDto as DomainCreateApplicatorLicenseDto,
    UpdateApplicatorLicenseDto as DomainUpdateApplicatorLicenseDto,
};
use agrocore_shared::{Pagination, SharedError};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/compliance/checklists")
            .route(web::get().to(list_checklists))
            .route(web::post().to(create_checklist)),
    )
    .service(
        web::resource("/compliance/checklists/{id}")
            .route(web::get().to(get_checklist))
            .route(web::put().to(update_checklist))
            .route(web::delete().to(delete_checklist)),
    )
    .service(web::resource("/compliance/fertilizer").route(web::get().to(list_fertilizer_records)))
    .service(
        web::resource("/compliance/fertilizer/{id}")
            .route(web::get().to(get_fertilizer_record))
            .route(web::put().to(update_fertilizer_record))
            .route(web::delete().to(delete_fertilizer_record)),
    )
    .service(
        web::resource("/compliance/plant-protection")
            .route(web::get().to(list_plant_protection_records)),
    )
    .service(
        web::resource("/compliance/applicator-licenses")
            .route(web::get().to(list_applicator_licenses))
            .route(web::post().to(create_applicator_license)),
    )
    .service(
        web::resource("/compliance/applicator-licenses/{id}")
            .route(web::get().to(get_applicator_license))
            .route(web::put().to(update_applicator_license))
            .route(web::delete().to(delete_applicator_license)),
    )
    .service(web::resource("/compliance/audit-logs").route(web::get().to(list_audit_logs)));
}

pub async fn list_checklists(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .compliance_checklist_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn list_fertilizer_records(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .fertilizer_record_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn list_plant_protection_records(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .plant_protection_record_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_checklist(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateComplianceChecklistDto>,
) -> Result<HttpResponse, ApiError> {
    let checklist = state
        .db
        .compliance_checklist_repo()
        .create(auth.0.tenant_id, dto.into_inner().into(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(checklist))
}

pub async fn get_checklist(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let checklist = state
        .db
        .compliance_checklist_repo()
        .find_by_id(auth.0.tenant_id, *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Checklist not found".into()))?;
    Ok(HttpResponse::Ok().json(checklist))
}

pub async fn update_checklist(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateComplianceChecklistDto>,
) -> Result<HttpResponse, ApiError> {
    let checklist = state
        .db
        .compliance_checklist_repo()
        .update(
            auth.0.tenant_id,
            *id,
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Checklist not found".into()))?;
    Ok(HttpResponse::Ok().json(checklist))
}

pub async fn delete_checklist(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .compliance_checklist_repo()
        .delete(auth.0.tenant_id, *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Checklist not found".into()).into())
    }
}

pub async fn get_fertilizer_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .fertilizer_record_repo()
        .find_by_id(auth.0.tenant_id, *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Fertilizer record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn update_fertilizer_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateFertilizerRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .fertilizer_record_repo()
        .update(
            auth.0.tenant_id,
            *id,
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Fertilizer record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn delete_fertilizer_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .fertilizer_record_repo()
        .delete(auth.0.tenant_id, *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Fertilizer record not found".into()).into())
    }
}

pub async fn list_audit_logs(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .audit_log_repo()
        .find_all(auth.0.tenant_id, query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

// =============================================================================
// Applicator License endpoints
// =============================================================================

#[utoipa::path(
    get,
    path = "/api/v1/compliance/applicator-licenses",
    responses(
        (status = 200, description = "List applicator licenses", body = Vec<ApplicatorLicenseDto>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "compliance",
    security(("bearer_auth" = []))
)]
pub async fn list_applicator_licenses(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let licenses = state
        .db
        .plant_protection_record_repo()
        .find_all_applicator_licenses(auth.0.tenant_id)
        .await?;

    let dtos: Vec<ApplicatorLicenseDto> = licenses.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(dtos))
}

#[utoipa::path(
    get,
    path = "/api/v1/compliance/applicator-licenses/{id}",
    responses(
        (status = 200, description = "Applicator license details", body = ApplicatorLicenseDto),
        (status = 404, description = "License not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "compliance",
    security(("bearer_auth" = []))
)]
pub async fn get_applicator_license(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let license = state
        .db
        .plant_protection_record_repo()
        .find_applicator_license_by_user(auth.0.tenant_id, *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Applicator license not found".into()))?;

    Ok(HttpResponse::Ok().json(ApplicatorLicenseDto::from(license)))
}

#[utoipa::path(
    post,
    path = "/api/v1/compliance/applicator-licenses",
    request_body = CreateApplicatorLicenseDto,
    responses(
        (status = 201, description = "Applicator license created", body = ApplicatorLicenseDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "compliance",
    security(("bearer_auth" = []))
)]
pub async fn create_applicator_license(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateApplicatorLicenseDto>,
) -> Result<HttpResponse, ApiError> {
    let domain_dto: DomainCreateApplicatorLicenseDto = dto.into_inner().into();
    let license = state
        .db
        .plant_protection_record_repo()
        .create_applicator_license(auth.0.tenant_id, domain_dto)
        .await?;

    Ok(HttpResponse::Created().json(ApplicatorLicenseDto::from(license)))
}

#[utoipa::path(
    put,
    path = "/api/v1/compliance/applicator-licenses/{id}",
    request_body = UpdateApplicatorLicenseDto,
    responses(
        (status = 200, description = "Applicator license updated", body = ApplicatorLicenseDto),
        (status = 404, description = "License not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "compliance",
    security(("bearer_auth" = []))
)]
pub async fn update_applicator_license(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateApplicatorLicenseDto>,
) -> Result<HttpResponse, ApiError> {
    let domain_dto: DomainUpdateApplicatorLicenseDto = dto.into_inner().into();
    let license = state
        .db
        .plant_protection_record_repo()
        .update_applicator_license(auth.0.tenant_id, *id, domain_dto)
        .await?
        .ok_or_else(|| SharedError::NotFound("Applicator license not found".into()))?;

    Ok(HttpResponse::Ok().json(ApplicatorLicenseDto::from(license)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/compliance/applicator-licenses/{id}",
    responses(
        (status = 204, description = "Applicator license deleted"),
        (status = 404, description = "License not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "compliance",
    security(("bearer_auth" = []))
)]
pub async fn delete_applicator_license(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .plant_protection_record_repo()
        .delete_applicator_license(auth.0.tenant_id, *id)
        .await?;

    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Applicator license not found".into()).into())
    }
}
