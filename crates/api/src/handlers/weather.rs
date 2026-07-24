use crate::AppState;
use crate::dto::{
    ErrorResponse, PaginatedPhenologyResponse, PaginatedResponseDto, PaginatedWeatherDataResponse,
    PaginatedWeatherStationResponse,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::weather::{
    CreatePhenologyRecordDto, CreateWeatherDataDto, CreateWeatherStationDto,
    UpdatePhenologyRecordDto, UpdateWeatherDataDto, UpdateWeatherStationDto,
};
use agrocore_shared::SharedError;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/weather")
            .service(
                web::resource("/stations")
                    .route(web::get().to(list_stations))
                    .route(web::post().to(create_station)),
            )
            .service(
                web::resource("/stations/{id}")
                    .route(web::get().to(get_station))
                    .route(web::put().to(update_station))
                    .route(web::delete().to(delete_station)),
            )
            .service(
                web::resource("/data")
                    .route(web::get().to(list_weather_data))
                    .route(web::post().to(create_weather_data)),
            )
            .service(
                web::resource("/data/{id}")
                    .route(web::get().to(get_weather_data))
                    .route(web::put().to(update_weather_data))
                    .route(web::delete().to(delete_weather_data)),
            )
            .service(
                web::resource("/phenology")
                    .route(web::get().to(list_phenology))
                    .route(web::post().to(create_phenology)),
            )
            .service(
                web::resource("/phenology/{id}")
                    .route(web::get().to(get_phenology))
                    .route(web::put().to(update_phenology))
                    .route(web::delete().to(delete_phenology)),
            ),
    );
}

#[utoipa::path(
    get,
    path = "/api/v1/weather/stations",
    responses(
        (status = 200, description = "List weather stations", body = PaginatedWeatherStationResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_stations(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .weather_station_repo()
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

#[utoipa::path(
    get,
    path = "/api/v1/weather/stations/{id}",
    responses(
        (status = 200, description = "Get weather station", body = agrocore_domain::entities::weather::WeatherStation),
        (status = 404, description = "Station not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Station ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_station(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let station = state
        .db
        .weather_station_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Station not found".into()))?;
    Ok(HttpResponse::Ok().json(station))
}

pub async fn update_station(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWeatherStationDto>,
) -> Result<HttpResponse, ApiError> {
    let station = state
        .db
        .weather_station_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Station not found".into()))?;
    Ok(HttpResponse::Ok().json(station))
}

pub async fn delete_station(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .weather_station_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Station not found".into()).into())
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/weather/stations",
    request_body = CreateWeatherStationDto,
    responses(
        (status = 201, description = "Station created", body = agrocore_domain::entities::weather::WeatherStation)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_station(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWeatherStationDto>,
) -> Result<HttpResponse, ApiError> {
    let station = state
        .db
        .weather_station_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(station))
}

#[utoipa::path(
    get,
    path = "/api/v1/weather/data",
    responses(
        (status = 200, description = "List weather data", body = PaginatedWeatherDataResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_weather_data(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .weather_data_repo()
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

#[utoipa::path(
    post,
    path = "/api/v1/weather/data",
    request_body = CreateWeatherDataDto,
    responses(
        (status = 201, description = "Weather data created", body = agrocore_domain::entities::weather::WeatherData)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_weather_data(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateWeatherDataDto>,
) -> Result<HttpResponse, ApiError> {
    let wd = state
        .db
        .weather_data_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Created().json(wd))
}

pub async fn get_weather_data(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let data = state
        .db
        .weather_data_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Weather data not found".into()))?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn update_weather_data(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateWeatherDataDto>,
) -> Result<HttpResponse, ApiError> {
    let data = state
        .db
        .weather_data_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Weather data not found".into()))?;
    Ok(HttpResponse::Ok().json(data))
}

pub async fn delete_weather_data(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .weather_data_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Weather data not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/weather/phenology",
    responses(
        (status = 200, description = "List phenology records", body = PaginatedPhenologyResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_phenology(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .phenology_record_repo()
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

#[utoipa::path(
    post,
    path = "/api/v1/weather/phenology",
    request_body = CreatePhenologyRecordDto,
    responses(
        (status = 201, description = "Phenology record created", body = agrocore_domain::entities::weather::PhenologyRecord)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_phenology(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreatePhenologyRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let pr = state
        .db
        .phenology_record_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(pr))
}

pub async fn get_phenology(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .phenology_record_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Phenology record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn update_phenology(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdatePhenologyRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let record = state
        .db
        .phenology_record_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Phenology record not found".into()))?;
    Ok(HttpResponse::Ok().json(record))
}

pub async fn delete_phenology(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .phenology_record_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Phenology record not found".into()).into())
    }
}
