use crate::AppState;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::harvest::{
    CreateColdChainLogDto, CreateHarvestDeliveryDto, CreateHarvestLotDto, CreateHarvestSeasonDto,
    UpdateColdChainLogDto, UpdateHarvestDeliveryDto, UpdateHarvestLotDto, UpdateHarvestSeasonDto,
};
use agrocore_shared::Pagination;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/harvest")
            .service(
                web::resource("/seasons")
                    .route(web::get().to(list_seasons))
                    .route(web::post().to(create_season)),
            )
            .service(
                web::resource("/seasons/{id}")
                    .route(web::get().to(get_season))
                    .route(web::put().to(update_season))
                    .route(web::delete().to(delete_season)),
            )
            .service(
                web::resource("/lots")
                    .route(web::get().to(list_lots))
                    .route(web::post().to(create_lot)),
            )
            .service(
                web::resource("/lots/{id}")
                    .route(web::get().to(get_lot))
                    .route(web::put().to(update_lot))
                    .route(web::delete().to(delete_lot)),
            )
            .service(
                web::resource("/deliveries")
                    .route(web::get().to(list_deliveries))
                    .route(web::post().to(create_delivery)),
            )
            .service(
                web::resource("/deliveries/{id}")
                    .route(web::get().to(get_delivery))
                    .route(web::put().to(update_delivery))
                    .route(web::delete().to(delete_delivery)),
            )
            .service(
                web::resource("/cold-chain")
                    .route(web::get().to(list_cold_chain_logs))
                    .route(web::post().to(create_cold_chain_log)),
            )
            .service(
                web::resource("/cold-chain/{id}")
                    .route(web::get().to(get_cold_chain_log))
                    .route(web::put().to(update_cold_chain_log))
                    .route(web::delete().to(delete_cold_chain_log)),
            ),
    );
}

// Seasons
pub async fn list_seasons(
    state: web::Data<AppState>,
    auth: AuthUser,
    p: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_season_repo().find_all(auth.0.tenant_id, p.into_inner()).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn create_season(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateHarvestSeasonDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_season_repo().create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id).await?;
    Ok(HttpResponse::Created().json(res))
}

pub async fn get_season(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_season_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Season not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn update_season(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateHarvestSeasonDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_season_repo().update(auth.0.tenant_id, id.into_inner(), dto.into_inner(), auth.0.user_id).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Season not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn delete_season(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = state.db.harvest_season_repo().delete(auth.0.tenant_id, id.into_inner()).await?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(agrocore_shared::SharedError::NotFound("Season not found".into()).into())
    }
}

// Lots
pub async fn list_lots(
    state: web::Data<AppState>,
    auth: AuthUser,
    p: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_lot_repo().find_all(auth.0.tenant_id, p.into_inner()).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn create_lot(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateHarvestLotDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_lot_repo().create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id).await?;
    Ok(HttpResponse::Created().json(res))
}

pub async fn get_lot(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_lot_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Lot not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn update_lot(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateHarvestLotDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_lot_repo().update(auth.0.tenant_id, id.into_inner(), dto.into_inner(), auth.0.user_id).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Lot not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn delete_lot(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = state.db.harvest_lot_repo().delete(auth.0.tenant_id, id.into_inner()).await?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(agrocore_shared::SharedError::NotFound("Lot not found".into()).into())
    }
}

// Deliveries
pub async fn list_deliveries(
    state: web::Data<AppState>,
    auth: AuthUser,
    p: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_delivery_repo().find_all(auth.0.tenant_id, p.into_inner()).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn create_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateHarvestDeliveryDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_delivery_repo().create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id).await?;
    Ok(HttpResponse::Created().json(res))
}

pub async fn get_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_delivery_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Delivery not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn update_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateHarvestDeliveryDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.harvest_delivery_repo().update(auth.0.tenant_id, id.into_inner(), dto.into_inner(), auth.0.user_id).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Delivery not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn delete_delivery(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = state.db.harvest_delivery_repo().delete(auth.0.tenant_id, id.into_inner()).await?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(agrocore_shared::SharedError::NotFound("Delivery not found".into()).into())
    }
}

// Cold Chain
pub async fn list_cold_chain_logs(
    state: web::Data<AppState>,
    auth: AuthUser,
    p: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.cold_chain_log_repo().find_all(auth.0.tenant_id, p.into_inner()).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn create_cold_chain_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateColdChainLogDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.cold_chain_log_repo().create(auth.0.tenant_id, dto.into_inner()).await?;
    Ok(HttpResponse::Created().json(res))
}

pub async fn get_cold_chain_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.cold_chain_log_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Log not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn update_cold_chain_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateColdChainLogDto>,
) -> Result<HttpResponse, ApiError> {
    let res = state.db.cold_chain_log_repo().update(auth.0.tenant_id, id.into_inner(), dto.into_inner()).await?
        .ok_or_else(|| agrocore_shared::SharedError::NotFound("Log not found".into()))?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn delete_cold_chain_log(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = state.db.cold_chain_log_repo().delete(auth.0.tenant_id, id.into_inner()).await?;
    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(agrocore_shared::SharedError::NotFound("Log not found".into()).into())
    }
}
