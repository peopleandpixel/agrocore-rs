use actix_web::{web, HttpResponse, Responder};
use crate::dto::{ErrorResponse, PaginatedResponseDto, PaginatedWeatherStationResponse, PaginatedWeatherDataResponse, PaginatedPhenologyResponse};
use crate::middleware::AuthExtractor as AuthUser;
use crate::AppState;
use agrocore_domain::entities::weather::{
    CreateWeatherStationDto, CreateWeatherDataDto, CreatePhenologyRecordDto
};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/weather/stations")
            .route(web::get().to(list_stations))
            .route(web::post().to(create_station))
    ).service(
        web::resource("/weather/stations/{id}")
            .route(web::get().to(get_station))
    ).service(
        web::resource("/weather/data")
            .route(web::get().to(list_weather_data))
            .route(web::post().to(create_weather_data))
    ).service(
        web::resource("/weather/phenology")
            .route(web::get().to(list_phenology))
            .route(web::post().to(create_phenology))
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
) -> impl Responder {
    match state.db.weather_station_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.weather_station_repo().find_by_id(auth.0.tenant_id.into(), *id).await {
        Ok(Some(s)) => HttpResponse::Ok().json(s),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "not_found".into(), message: "Station not found".into() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
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
) -> impl Responder {
    match state.db.weather_station_repo().create(auth.0.tenant_id.into(), dto.into_inner()).await {
        Ok(s) => HttpResponse::Created().json(s),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.weather_data_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.weather_data_repo().create(auth.0.tenant_id.into(), dto.into_inner()).await {
        Ok(wd) => HttpResponse::Created().json(wd),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
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
) -> impl Responder {
    match state.db.phenology_record_repo().find_all(auth.0.tenant_id.into(), query.0).await {
        Ok(result) => HttpResponse::Ok().json(PaginatedResponseDto {
            data: result.data,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.phenology_record_repo().create(auth.0.tenant_id.into(), dto.into_inner()).await {
        Ok(pr) => HttpResponse::Created().json(pr),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "internal".into(), message: e.to_string() }),
    }
}
