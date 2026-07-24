use crate::AppState;
use crate::dto::PaginatedResponseDto;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::olive::{
    CreateOliveGroveDto, CreateOliveOilRecordDto, UpdateOliveGroveDto, UpdateOliveOilRecordDto,
};
use agrocore_domain::entities::vineyard::{
    CreateKelterDeliveryDto, CreateVineyardDto, UpdateKelterDeliveryDto, UpdateVineyardDto,
};
use agrocore_shared::{Pagination, SharedError};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/specialized")
            .service(
                web::resource("/olive-groves")
                    .route(web::get().to(list_olive_groves))
                    .route(web::post().to(create_olive_grove)),
            )
            .service(
                web::resource("/olive-groves/{id}")
                    .route(web::get().to(get_olive_grove))
                    .route(web::put().to(update_olive_grove))
                    .route(web::delete().to(delete_olive_grove)),
            )
            .service(
                web::resource("/olive-oil-records")
                    .route(web::get().to(list_olive_oil_records))
                    .route(web::post().to(create_olive_oil_record)),
            )
            .service(
                web::resource("/olive-oil-records/{id}")
                    .route(web::get().to(get_olive_oil_record))
                    .route(web::put().to(update_olive_oil_record))
                    .route(web::delete().to(delete_olive_oil_record)),
            )
            .service(
                web::resource("/vineyards")
                    .route(web::get().to(list_vineyards))
                    .route(web::post().to(create_vineyard)),
            )
            .service(
                web::resource("/vineyards/{id}")
                    .route(web::get().to(get_vineyard))
                    .route(web::put().to(update_vineyard))
                    .route(web::delete().to(delete_vineyard)),
            )
            .service(
                web::resource("/kelter-deliveries")
                    .route(web::get().to(list_kelter_deliveries))
                    .route(web::post().to(create_kelter_delivery)),
            )
            .service(
                web::resource("/kelter-deliveries/{id}")
                    .route(web::get().to(get_kelter_delivery))
                    .route(web::put().to(update_kelter_delivery))
                    .route(web::delete().to(delete_kelter_delivery)),
            ),
    );
}

// Olive Grove
pub async fn list_olive_groves(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .olive_grove_repo()
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

pub async fn create_olive_grove(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateOliveGroveDto>,
) -> Result<HttpResponse, ApiError> {
    let grove = state
        .db
        .olive_grove_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(grove))
}

pub async fn get_olive_grove(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let grove = state
        .db
        .olive_grove_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Olive grove not found".into()))?;
    Ok(HttpResponse::Ok().json(grove))
}

pub async fn update_olive_grove(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateOliveGroveDto>,
) -> Result<HttpResponse, ApiError> {
    let grove = state
        .db
        .olive_grove_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Olive grove not found".into()))?;
    Ok(HttpResponse::Ok().json(grove))
}

pub async fn delete_olive_grove(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .olive_grove_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Olive grove not found".into()).into())
    }
}

// Olive Oil Record
pub async fn list_olive_oil_records(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .olive_oil_record_repo()
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

pub async fn create_olive_oil_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateOliveOilRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .olive_oil_record_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(record))
}

pub async fn get_olive_oil_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .olive_oil_record_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Olive oil record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn update_olive_oil_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateOliveOilRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .olive_oil_record_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Olive oil record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn delete_olive_oil_record(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .olive_oil_record_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Olive oil record not found".into()).into())
    }
}

// Vineyard
pub async fn list_vineyards(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .vineyard_repo()
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

pub async fn create_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateVineyardDto>,
) -> Result<HttpResponse, ApiError> {
    let v = state
        .db
        .vineyard_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(v))
}

pub async fn get_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let v = state
        .db
        .vineyard_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Vineyard not found".into()))?;
    Ok(HttpResponse::Ok().json(v))
}

pub async fn update_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateVineyardDto>,
) -> Result<HttpResponse, ApiError> {
    let v = state
        .db
        .vineyard_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Vineyard not found".into()))?;
    Ok(HttpResponse::Ok().json(v))
}

pub async fn delete_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .vineyard_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Vineyard not found".into()).into())
    }
}

// Kelter Delivery
pub async fn list_kelter_deliveries(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .kelter_delivery_repo()
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

pub async fn create_kelter_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateKelterDeliveryDto>,
) -> Result<HttpResponse, ApiError> {
    let kd = state
        .db
        .kelter_delivery_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(kd))
}

pub async fn get_kelter_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let kd = state
        .db
        .kelter_delivery_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Kelter delivery not found".into()))?;
    Ok(HttpResponse::Ok().json(kd))
}

pub async fn update_kelter_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateKelterDeliveryDto>,
) -> Result<HttpResponse, ApiError> {
    let kd = state
        .db
        .kelter_delivery_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Kelter delivery not found".into()))?;
    Ok(HttpResponse::Ok().json(kd))
}

pub async fn delete_kelter_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .kelter_delivery_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Kelter delivery not found".into()).into())
    }
}
