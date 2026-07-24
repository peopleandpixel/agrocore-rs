use crate::AppState;
use crate::dto::PaginatedResponseDto;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::water::{
    CreateWaterQuotaDto, CreateWaterSourceDto, CreateWaterUsageDto, UpdateWaterQuotaDto,
    UpdateWaterSourceDto, UpdateWaterUsageDto,
};
use agrocore_shared::{Pagination, SharedError};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/water")
            .service(
                web::resource("/sources")
                    .route(web::get().to(list_water_sources))
                    .route(web::post().to(create_water_source)),
            )
            .service(
                web::resource("/sources/{id}")
                    .route(web::get().to(get_water_source))
                    .route(web::put().to(update_water_source))
                    .route(web::delete().to(delete_water_source)),
            )
            .service(
                web::resource("/usage")
                    .route(web::get().to(list_water_usage))
                    .route(web::post().to(create_water_usage)),
            )
            .service(
                web::resource("/usage/{id}")
                    .route(web::get().to(get_water_usage))
                    .route(web::put().to(update_water_usage))
                    .route(web::delete().to(delete_water_usage)),
            )
            .service(
                web::resource("/quotas")
                    .route(web::get().to(list_water_quotas))
                    .route(web::post().to(create_water_quota)),
            )
            .service(
                web::resource("/quotas/{id}")
                    .route(web::get().to(get_water_quota))
                    .route(web::put().to(update_water_quota))
                    .route(web::delete().to(delete_water_quota)),
            ),
    );
}

pub async fn list_water_sources(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .water_source_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_water_source(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWaterSourceDto>,
) -> Result<HttpResponse, ApiError> {
    let source = state
        .db
        .water_source_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(source))
}

pub async fn get_water_source(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let source = state
        .db
        .water_source_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Water source not found".into()))?;
    Ok(HttpResponse::Ok().json(source))
}

pub async fn update_water_source(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWaterSourceDto>,
) -> Result<HttpResponse, ApiError> {
    let source = state
        .db
        .water_source_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Water source not found".into()))?;
    Ok(HttpResponse::Ok().json(source))
}

pub async fn delete_water_source(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .water_source_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Water source not found".into()).into())
    }
}

pub async fn list_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .water_usage_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWaterUsageDto>,
) -> Result<HttpResponse, ApiError> {
    let usage = state
        .db
        .water_usage_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(usage))
}

pub async fn get_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let usage = state
        .db
        .water_usage_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Water usage not found".into()))?;
    Ok(HttpResponse::Ok().json(usage))
}

pub async fn update_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWaterUsageDto>,
) -> Result<HttpResponse, ApiError> {
    let usage = state
        .db
        .water_usage_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Water usage not found".into()))?;
    Ok(HttpResponse::Ok().json(usage))
}

pub async fn delete_water_usage(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .water_usage_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Water usage not found".into()).into())
    }
}

pub async fn list_water_quotas(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .water_quota_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data,
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_water_quota(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWaterQuotaDto>,
) -> Result<HttpResponse, ApiError> {
    let quota = state
        .db
        .water_quota_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(quota))
}

pub async fn get_water_quota(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let quota = state
        .db
        .water_quota_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Water quota not found".into()))?;
    Ok(HttpResponse::Ok().json(quota))
}

pub async fn update_water_quota(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWaterQuotaDto>,
) -> Result<HttpResponse, ApiError> {
    let quota = state
        .db
        .water_quota_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Water quota not found".into()))?;
    Ok(HttpResponse::Ok().json(quota))
}

pub async fn delete_water_quota(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .water_quota_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Water quota not found".into()).into())
    }
}
