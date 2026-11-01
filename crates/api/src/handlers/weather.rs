use crate::AppState;
use crate::dto::{
    ErrorResponse, PaginatedPhenologyResponse, PaginatedResponseDto, PaginatedWeatherDataResponse,
    PaginatedWeatherStationResponse, WeatherFetchResultDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::WeatherDataProvider;
use agrocore_domain::entities::weather::{
    CreatePhenologyRecordDto, CreateWeatherDataDto, CreateWeatherStationDto,
    UpdatePhenologyRecordDto, UpdateWeatherDataDto, UpdateWeatherStationDto,
};
use agrocore_logging::warn;
use agrocore_shared::SharedError;
use agrocore_weather_service::providers::OpenMeteoProvider;
use uuid::Uuid;
use validator::Validate;

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
            )
            .service(
                web::resource("/frost-warnings")
                    .route(web::get().to(list_frost_warnings))
                    .route(web::post().to(create_frost_warning)),
            )
            .service(
                web::resource("/frost-warnings/{id}")
                    .route(web::get().to(get_frost_warning))
                    .route(web::put().to(update_frost_warning))
                    .route(web::delete().to(delete_frost_warning)),
            )
            .service(
                web::resource("/frost-warnings/active")
                    .route(web::get().to(get_active_frost_warnings)),
            )
            .service(
                web::resource("/gdd")
                    .route(web::get().to(list_gdd))
                    .route(web::post().to(create_gdd)),
            )
            .service(web::resource("/gdd/accumulated").route(web::get().to(get_accumulated_gdd)))
            .service(
                web::resource("/pest-risks")
                    .route(web::get().to(list_pest_risks))
                    .route(web::post().to(create_pest_risk)),
            )
            .service(
                web::resource("/current-with-fallback")
                    .route(web::get().to(get_current_weather_with_fallback)),
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
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

// --- Frost Warning Handlers ---

pub async fn list_frost_warnings(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .frost_warning_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(crate::dto::PaginatedResponseDto {
        data: result
            .data
            .into_iter()
            .map(crate::dto::FrostWarningDto::from)
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_frost_warning(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<crate::dto::CreateFrostWarningDto>,
) -> Result<HttpResponse, ApiError> {
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let fw = state
        .db
        .frost_warning_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(crate::dto::FrostWarningDto::from(fw)))
}

pub async fn get_frost_warning(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let fw = state
        .db
        .frost_warning_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Frost warning not found".into()))?;
    Ok(HttpResponse::Ok().json(crate::dto::FrostWarningDto::from(fw)))
}

pub async fn update_frost_warning(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<crate::dto::UpdateFrostWarningDto>,
) -> Result<HttpResponse, ApiError> {
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let fw = state
        .db
        .frost_warning_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Frost warning not found".into()))?;
    Ok(HttpResponse::Ok().json(crate::dto::FrostWarningDto::from(fw)))
}

pub async fn delete_frost_warning(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let success = state
        .db
        .frost_warning_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?;
    if success {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Frost warning not found".into()).into())
    }
}

pub async fn get_active_frost_warnings(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .frost_warning_repo()
        .find_active(agrocore_domain::TenantId(auth.0.tenant_id))
        .await?;
    Ok(HttpResponse::Ok().json(
        result
            .into_iter()
            .map(crate::dto::FrostWarningDto::from)
            .collect::<Vec<_>>(),
    ))
}

// --- Growing Degree Day Handlers ---

pub async fn list_gdd(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<crate::dto::SitePagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .growing_degree_day_repo()
        .find_by_site(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.site_id,
            query.pagination.clone(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(crate::dto::PaginatedResponseDto {
        data: result
            .data
            .into_iter()
            .map(crate::dto::GrowingDegreeDayDto::from)
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_gdd(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<crate::dto::CreateGrowingDegreeDayDto>,
) -> Result<HttpResponse, ApiError> {
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let gdd = state
        .db
        .growing_degree_day_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner().into(),
        )
        .await?;
    Ok(HttpResponse::Created().json(crate::dto::GrowingDegreeDayDto::from(gdd)))
}

pub async fn get_accumulated_gdd(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<crate::dto::GddQuery>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .growing_degree_day_repo()
        .accumulated_gdd(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.site_id,
            query.from,
            query.to,
            query.crop_type.clone(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(crate::dto::AccumulatedGddResponse {
        site_id: query.site_id,
        crop_type: query.crop_type.clone(),
        accumulated_gdd: result,
    }))
}

// --- Pest Risk Handlers ---

pub async fn list_pest_risks(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<crate::dto::SitePagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .pest_risk_repo()
        .find_by_site(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.site_id,
            query.pagination.clone(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(crate::dto::PaginatedResponseDto {
        data: result
            .data
            .into_iter()
            .map(crate::dto::PestRiskDto::from)
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn create_pest_risk(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<crate::dto::CreatePestRiskDto>,
) -> Result<HttpResponse, ApiError> {
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let pr = state
        .db
        .pest_risk_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner().into(),
        )
        .await?;
    Ok(HttpResponse::Created().json(crate::dto::PestRiskDto::from(pr)))
}

/// Returns current weather data for the tenant's company location.
///
/// If the tenant has configured weather stations with recent data, returns the
/// latest reading. If no stations exist or no recent data is available, falls
/// back to fetching weather from Open-Meteo using the company coordinates stored
/// in the tenant configuration.
#[utoipa::path(
    get,
    path = "/api/v1/weather/current-with-fallback",
    responses(
        (status = 200, description = "Current weather data with fallback", body = WeatherFetchResultDto),
        (status = 400, description = "No coordinates configured", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_weather_with_fallback(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = auth.0.tenant_id;

    // Step 1: Try to get weather data from configured stations
    let tenant = state
        .db
        .tenant_repo()
        .find_by_id(tenant_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Tenant not found".into()))?;

    // Check if there are any active weather stations with recent data
    let stations = state
        .db
        .weather_station_repo()
        .find_all(
            agrocore_domain::TenantId(tenant_id),
            agrocore_shared::Pagination::default(),
        )
        .await?;

    let has_active_stations = stations.data.iter().any(|s| s.is_active);

    if has_active_stations {
        // Try to get the most recent weather data from stations
        let weather_data = state
            .db
            .weather_data_repo()
            .find_all(
                agrocore_domain::TenantId(tenant_id),
                agrocore_shared::Pagination::default(),
            )
            .await?;

        if let Some(latest) = weather_data.data.first() {
            // Return sensor data if available
            return Ok(HttpResponse::Ok().json(WeatherFetchResultDto {
                temperature_c: latest.temperature_c,
                humidity_percent: latest.humidity_percent,
                precipitation_mm: latest.precipitation_mm,
                wind_speed_kmh: latest.wind_speed_kmh,
                wind_direction_deg: latest.wind_direction_deg,
                solar_radiation_wm2: latest.solar_radiation_wm2,
                pressure_hpa: latest.pressure_hpa,
                soil_temperature_c: latest.soil_temperature_c,
                soil_moisture_percent: latest.soil_moisture_percent,
                leaf_wetness: latest.leaf_wetness,
                timestamp: latest.timestamp,
                provider: "sensor".to_string(),
            }));
        }
    }

    // Step 2: Fallback to Open-Meteo using company coordinates
    let (lat, lon) = match &tenant.config.company_profile {
        Some(profile) => match &profile.coordinates {
            Some(coords) => (coords.lat, coords.lng),
            None => {
                warn!(
                    "Tenant {} has no company coordinates configured for weather fallback",
                    tenant_id
                );
                return Err(SharedError::Internal(
                    "No company coordinates configured. Please set your company location on the map first."
                        .to_string(),
                )
                .into());
            }
        },
        None => {
            warn!(
                "Tenant {} has no company profile configured for weather fallback",
                tenant_id
            );
            return Err(SharedError::Internal(
                "No company profile configured. Please complete your company setup first."
                    .to_string(),
            )
            .into());
        }
    };

    // Fetch from Open-Meteo (free, no API key required)
    let provider = OpenMeteoProvider;
    let result = provider.fetch_current(lat, lon, None).await.map_err(|e| {
        warn!("Open-Meteo fallback failed: {}", e);
        SharedError::Internal(format!("Weather service unavailable: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(WeatherFetchResultDto {
        temperature_c: result.temperature_c,
        humidity_percent: result.humidity_percent,
        precipitation_mm: result.precipitation_mm,
        wind_speed_kmh: result.wind_speed_kmh,
        wind_direction_deg: result.wind_direction_deg,
        solar_radiation_wm2: result.solar_radiation_wm2,
        pressure_hpa: result.pressure_hpa,
        soil_temperature_c: result.soil_temperature_c,
        soil_moisture_percent: result.soil_moisture_percent,
        leaf_wetness: result.leaf_wetness,
        timestamp: result.timestamp,
        provider: "openmeteo-fallback".to_string(),
    }))
}
