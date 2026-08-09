use crate::AppState;
use crate::dto::{
    CreateVineyardDto, MaterialCalculationRequestDto, MaterialCalculationResponseDto,
    PaginatedResponseDto, PaginatedVineyardResponse, SiteDto, UpdateVineyardDto, VineyardDto,
    WaterRateCalculationRequestDto, WaterRateCalculationResponseDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_domain::services::calculation::{CalculationService, MaterialAmountRequest};
use agrocore_shared::SharedError;
use chrono::Utc;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/specialized/sites").route(web::get().to(list_specialized_sites)))
        .service(
            web::resource("/specialized/vineyards")
                .route(web::get().to(list_vineyards))
                .route(web::post().to(create_vineyard)),
        )
        .service(
            web::resource("/specialized/vineyards/site/{site_id}")
                .route(web::get().to(list_vineyards_by_site)),
        )
        .service(
            web::resource("/specialized/vineyards/{id}")
                .route(web::get().to(get_vineyard))
                .route(web::put().to(update_vineyard))
                .route(web::delete().to(delete_vineyard)),
        )
        .service(web::resource("/calculate/material").route(web::post().to(calculate_material)))
        .service(web::resource("/calculate/water-rate").route(web::post().to(calculate_water_rate)))
        .service(web::resource("/predict/harvest").route(web::get().to(predict_harvest)))
        .service(
            web::resource("/specialized/profitability")
                .route(web::post().to(calculate_profitability)),
        );
}

pub async fn list_vineyards(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .vineyard_repo()
        .find_all(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedVineyardResponse {
        data: result.data.into_iter().map(VineyardDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn list_vineyards_by_site(
    state: web::Data<AppState>,
    auth: AuthUser,
    site_id: web::Path<Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .vineyard_repo()
        .find_by_site(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *site_id,
            query.into_inner(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedVineyardResponse {
        data: result.data.into_iter().map(VineyardDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn get_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let vineyard = state
        .db
        .vineyard_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Vineyard not found".into()))?;
    Ok(HttpResponse::Ok().json(VineyardDto::from(vineyard)))
}

pub async fn create_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateVineyardDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let vineyard = state
        .db
        .vineyard_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(VineyardDto::from(vineyard)))
}

pub async fn update_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
    dto: web::Json<UpdateVineyardDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let vineyard = state
        .db
        .vineyard_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *id,
            dto.into_inner().into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Vineyard not found".into()))?;
    Ok(HttpResponse::Ok().json(VineyardDto::from(vineyard)))
}

pub async fn delete_vineyard(
    state: web::Data<AppState>,
    auth: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .vineyard_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *id)
        .await?
    {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(SharedError::NotFound("Vineyard not found".into()).into())
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct HarvestPredictionQuery {
    pub site_id: Uuid,
    pub target_bbch: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/predict/harvest",
    params(HarvestPredictionQuery),
    responses(
        (status = 200, description = "Harvest prediction", body = serde_json::Value),
        (status = 404, description = "Site or data not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn predict_harvest(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<HarvestPredictionQuery>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = agrocore_domain::TenantId(auth.0.tenant_id);

    // 1. Aktuelle Phänologie abrufen
    let phenology = state
        .db
        .phenology_record_repo()
        .find_all(tenant_id, agrocore_shared::Pagination::default())
        .await;
    let latest_bbch = match phenology {
        Ok(res) => res
            .data
            .iter()
            .filter(|p| p.site_id == query.site_id)
            .max_by_key(|p| p.observation_date)
            .map(|p| p.stage.to_u32())
            .unwrap_or(10), // Default: Beginn des Wachstums
        Err(_) => 10,
    };

    // 2. Wetterdaten abrufen (für Durchschnittstemperatur)
    // Vereinfacht: Wir nehmen die letzten 7 Tage
    let avg_temp = 18.5; // Dummy, in Realität aggregiert aus Wetter-Repo

    let days_to_harvest = CalculationService::estimate_harvest_date(
        latest_bbch,
        query.target_bbch.unwrap_or(89),
        avg_temp,
        10.0, // Basis-Temp
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "site_id": query.site_id,
        "current_bbch": latest_bbch,
        "predicted_days": days_to_harvest,
        "confidence": "medium"
    })))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ProfitabilityRequest {
    pub site_id: Uuid,
    pub yield_amount: f64,
    pub price_per_unit: f64,
    pub material_costs: f64,
    pub labor_costs: f64,
    pub machinery_costs: f64,
}

#[utoipa::path(
    post,
    path = "/specialized/profitability",
    request_body = ProfitabilityRequest,
    responses(
        (status = 200, description = "Profitability analysis", body = serde_json::Value)
    ),
    security(("bearer_auth" = []))
)]
pub async fn calculate_profitability(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<ProfitabilityRequest>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = agrocore_domain::TenantId(auth.0.tenant_id);
    let site = state
        .db
        .site_repo()
        .find_by_id(tenant_id, dto.site_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;

    let profit_per_ha = CalculationService::calculate_profitability(
        dto.yield_amount,
        dto.price_per_unit,
        dto.material_costs,
        dto.labor_costs,
        dto.machinery_costs,
        site.area,
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "site_id": dto.site_id,
        "profit_per_ha": profit_per_ha,
        "total_profit": profit_per_ha * site.area,
        "currency": "EUR"
    })))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct SpecializedSiteQuery {
    pub site_type: Option<SiteType>,
    pub crop_type: Option<CropType>,
    #[serde(flatten)]
    pub pagination: agrocore_shared::Pagination,
}

pub async fn list_specialized_sites(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<SpecializedSiteQuery>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = agrocore_domain::TenantId(auth.0.tenant_id);

    // In einer echten Implementierung würde hier ein spezialisierter Repository-Aufruf stehen,
    // der nach site_type und crop_type filtert.
    // Für diesen Quick-Win nutzen wir das vorhandene Site-Repo und filtern (in-memory oder via Repo-Support).

    let result = state
        .db
        .site_repo()
        .find_all(tenant_id, query.pagination.clone())
        .await?;
    let filtered_data: Vec<SiteDto> = result
        .data
        .into_iter()
        .filter(|s| {
            let type_match = query
                .site_type
                .as_ref()
                .map(|t| &s.site_type == t)
                .unwrap_or(true);
            let crop_match = query
                .crop_type
                .as_ref()
                .map(|c| &s.crop_type == c)
                .unwrap_or(true);
            type_match && crop_match
        })
        .map(SiteDto::from)
        .collect();

    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: filtered_data,
        total: result.total, // Hinweis: total stimmt hier nicht exakt, wenn gefiltert wurde
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

pub async fn calculate_material(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<MaterialCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = agrocore_domain::TenantId(auth.0.tenant_id);

    // 1. Site-Daten abrufen
    let site = state
        .db
        .site_repo()
        .find_by_id(tenant_id, dto.site_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;

    let application_date = dto
        .application_date
        .as_ref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    // 2. Berechnung durchführen
    let treated_area = dto.method.calculate_treated_area(
        site.area,
        site.gross_area,
        site.row_config.as_ref().map(|rc| rc.lane_width),
        site.row_config.as_ref().map(|rc| rc.total_strike_length),
        site.slope.map(|s| s > 15.0).unwrap_or(false), // Annahme: Steil ab 15% Steigung
        application_date,
    );

    let total_amount = CalculationService::calculate_material_amount(MaterialAmountRequest {
        method: dto.method.clone(),
        net_area: site.area,
        gross_area: site.gross_area,
        lane_width: site.row_config.as_ref().map(|rc| rc.lane_width),
        total_strike_length: site.row_config.as_ref().map(|rc| rc.total_strike_length),
        is_steep: site.slope.map(|s| s > 15.0).unwrap_or(false),
        dosage_per_ha: dto.dose_per_ha,
        application_date,
    });

    Ok(HttpResponse::Ok().json(MaterialCalculationResponseDto {
        treated_area_ha: treated_area,
        total_material_amount: total_amount,
    }))
}

pub async fn calculate_water_rate(
    _auth: AuthUser,
    dto: web::Json<WaterRateCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    let water_rate = CalculationService::calculate_water_rate(
        dto.speed_kmh,
        dto.nozzle_flow_lmin,
        dto.lane_width,
        dto.number_of_nozzles,
    );

    Ok(HttpResponse::Ok().json(WaterRateCalculationResponseDto {
        water_rate_lha: water_rate,
    }))
}
