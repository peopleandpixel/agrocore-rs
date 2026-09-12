//! Calculation API Handlers

use crate::AppState;
use crate::dto::{
    DifficultySurchargeRequestDto, DifficultySurchargeResponseDto, FertilizerCalculationRequestDto,
    FertilizerCalculationResponseDto, FollowUpOrderDto, ForageDemandRequestDto,
    ForageDemandResponseDto, HarvestEstimationRequestDto, HarvestEstimationResponseDto,
    MaterialCalculationRequestDto, MaterialCalculationResponseDto, NitrogenDemandRequestDto,
    NitrogenDemandResponseDto, NutritionBalanceRequestDto, NutritionBalanceResponseDto,
    NutritionDemandRequestDto, NutritionDemandResponseDto, ProfitabilityCalculationRequestDto,
    ProfitabilityCalculationResponseDto, TreeCrownVolumeRequestDto, TreeCrownVolumeResponseDto,
    WaterRateCalculationRequestDto, WaterRateCalculationResponseDto, WeatherFetchRequestDto,
    WeatherFetchResultDto, WeatherProviderInfoDto, WeatherProvidersResponseDto,
    WorkflowFollowUpRequestDto, WorkflowFollowUpResponseDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::services::calculation::CalculationService;
use agrocore_domain::services::nutrition::{Fertilizer, NutrientValues, NutritionService};
use agrocore_domain::services::weather::{WeatherFetchResult, WeatherServiceType};
use agrocore_domain::services::workflow::WorkflowService;
use agrocore_shared::SharedError;
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

/// Configure calculation routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/calculate")
            // Nutrition calculations
            .service(
                web::resource("/nutrition/demand")
                    .route(web::post().to(calculate_nutrition_demand)),
            )
            .service(
                web::resource("/nutrition/fertilizer-amount")
                    .route(web::post().to(calculate_fertilizer_amount)),
            )
            .service(
                web::resource("/nutrition/balance")
                    .route(web::post().to(calculate_nutrition_balance)),
            )
            // Water calculations
            .service(web::resource("/water-rate").route(web::post().to(calculate_water_rate)))
            // Material calculations
            .service(web::resource("/material").route(web::post().to(calculate_material)))
            // Tree calculations
            .service(
                web::resource("/tree-crown-volume")
                    .route(web::post().to(calculate_tree_crown_volume)),
            )
            // Forage calculations
            .service(web::resource("/forage-demand").route(web::post().to(calculate_forage_demand)))
            // Nitrogen calculations
            .service(
                web::resource("/nitrogen-demand").route(web::post().to(calculate_nitrogen_demand)),
            )
            // Difficulty surcharge
            .service(
                web::resource("/difficulty-surcharge")
                    .route(web::post().to(calculate_difficulty_surcharge)),
            )
            // Profitability
            .service(web::resource("/profitability").route(web::post().to(calculate_profitability)))
            // Harvest estimation
            .service(web::resource("/harvest-estimation").route(web::post().to(estimate_harvest)))
            // Workflow follow-ups
            .service(
                web::resource("/workflow/follow-ups")
                    .route(web::post().to(get_workflow_follow_ups)),
            )
            // Weather providers
            .service(web::resource("/weather/fetch").route(web::post().to(fetch_weather)))
            .service(
                web::resource("/weather/providers").route(web::get().to(list_weather_providers)),
            ),
    );
}

// =============================================================================
// Nutrition Calculations
// =============================================================================

pub async fn calculate_nutrition_demand(
    _state: web::Data<AppState>,
    _auth: AuthUser,
    dto: web::Json<NutritionDemandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let site_area = dto.area_ha;

    let demand = NutrientValues {
        n: dto.soil_nitrogen.unwrap_or(0.0),
        p: dto.soil_phosphorus.unwrap_or(0.0),
        k: dto.soil_potassium.unwrap_or(0.0),
        mg: dto.organic_matter_percent.unwrap_or(0.0),
    };

    let total_demand = NutritionService::calculate_demand(site_area, dto.expected_yield, &demand);

    Ok(HttpResponse::Ok().json(NutritionDemandResponseDto {
        nitrogen_kg_per_ha: total_demand.n,
        phosphorus_kg_per_ha: total_demand.p,
        potassium_kg_per_ha: total_demand.k,
        magnesium_kg_per_ha: total_demand.mg,
        sulfur_kg_per_ha: 0.0,
        calcium_kg_per_ha: 0.0,
        total_nitrogen_kg: total_demand.n * site_area,
        total_phosphorus_kg: total_demand.p * site_area,
        total_potassium_kg: total_demand.k * site_area,
        area_ha: site_area,
    }))
}

pub async fn calculate_fertilizer_amount(
    _auth: AuthUser,
    dto: web::Json<FertilizerCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let total_demand = NutrientValues {
        n: dto.nutrition_demand.total_nitrogen_kg / dto.area_ha,
        p: dto.nutrition_demand.total_phosphorus_kg / dto.area_ha,
        k: dto.nutrition_demand.total_potassium_kg / dto.area_ha,
        mg: dto.nutrition_demand.magnesium_kg_per_ha,
    };

    let fertilizers: Vec<Fertilizer> = dto
        .fertilizer_types
        .iter()
        .map(|fi| Fertilizer {
            name: fi.name.clone(),
            nutrient_content_percent: NutrientValues {
                n: fi.nitrogen_percent,
                p: fi.phosphorus_percent,
                k: fi.potassium_percent,
                mg: 0.0,
            },
        })
        .collect();

    let recommendations = NutritionService::calculate_fertilizer_recommendations(
        &total_demand,
        &fertilizers,
        dto.area_ha,
    );

    let mut recommendations_dto = Vec::new();
    for rec in recommendations {
        recommendations_dto.push(crate::dto::FertilizerRecommendationDto {
            fertilizer_name: rec.fertilizer_name,
            amount_kg_per_ha: rec.amount_kg_per_ha,
            total_amount_kg: rec.total_amount_kg,
            nitrogen_supplied_kg: rec.nitrogen_supplied_kg,
            phosphorus_supplied_kg: rec.phosphorus_supplied_kg,
            potassium_supplied_kg: rec.potassium_supplied_kg,
            cost_eur: rec.cost_eur,
        });
    }

    let total_cost = recommendations_dto.iter().map(|r| r.cost_eur).sum();
    let total_n = recommendations_dto
        .iter()
        .map(|r| r.nitrogen_supplied_kg)
        .sum();
    let total_p = recommendations_dto
        .iter()
        .map(|r| r.phosphorus_supplied_kg)
        .sum();
    let total_k = recommendations_dto
        .iter()
        .map(|r| r.potassium_supplied_kg)
        .sum();

    Ok(HttpResponse::Ok().json(FertilizerCalculationResponseDto {
        recommendations: recommendations_dto,
        total_cost_eur: total_cost,
        total_nitrogen_kg: total_n,
        total_phosphorus_kg: total_p,
        total_potassium_kg: total_k,
    }))
}

pub async fn calculate_nutrition_balance(
    _auth: AuthUser,
    dto: web::Json<NutritionBalanceRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let fertilizer = Fertilizer {
        name: dto.fertilizer.name.clone(),
        nutrient_content_percent: NutrientValues {
            n: dto.fertilizer.nitrogen_percent,
            p: dto.fertilizer.phosphorus_percent,
            k: dto.fertilizer.potassium_percent,
            mg: 0.0,
        },
    };

    let area_ha = dto.demand.area_ha;
    let balance = NutritionService::calculate_balance(
        &NutrientValues {
            n: dto.demand.total_nitrogen_kg / area_ha,
            p: dto.demand.total_phosphorus_kg / area_ha,
            k: dto.demand.total_potassium_kg / area_ha,
            mg: dto.demand.magnesium_kg_per_ha,
        },
        dto.applied_amount_kg,
        &fertilizer,
    );

    Ok(HttpResponse::Ok().json(NutritionBalanceResponseDto {
        nitrogen_balance_kg: balance.n,
        phosphorus_balance_kg: balance.p,
        potassium_balance_kg: balance.k,
        magnesium_balance_kg: balance.mg,
    }))
}

// =============================================================================
// Water Calculations
// =============================================================================

pub async fn calculate_water_rate(
    _auth: AuthUser,
    dto: web::Json<WaterRateCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

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

// =============================================================================
// Material Calculations
// =============================================================================

pub async fn calculate_material(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<MaterialCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "worker"])?;

    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    // Get site for area info
    let site = state
        .db
        .site_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), dto.site_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;

    let method = dto.method.clone();
    let net_area = site.area;
    let gross_area = site.gross_area.unwrap_or(site.area); // Could be different in real implementation
    let is_steep = site.slope.map(|s| s > 15.0).unwrap_or(false); // Steep if slope > 15%

    let request = agrocore_domain::services::calculation::MaterialAmountRequest {
        method,
        net_area,
        gross_area: Some(gross_area),
        lane_width: None,
        total_strike_length: None,
        is_steep,
        dosage_per_ha: dto.dose_per_ha,
        application_date: dto
            .application_date
            .clone()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now),
    };

    let total_material = CalculationService::calculate_material_amount(request);

    Ok(HttpResponse::Ok().json(MaterialCalculationResponseDto {
        treated_area_ha: net_area,
        total_material_amount: total_material,
    }))
}

// =============================================================================
// Tree Crown Volume
// =============================================================================

pub async fn calculate_tree_crown_volume(
    _auth: AuthUser,
    dto: web::Json<TreeCrownVolumeRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let volume = CalculationService::calculate_tree_crown_volume(
        dto.crown_diameter,
        dto.tree_height,
        dto.trees_per_ha,
    );

    Ok(HttpResponse::Ok().json(TreeCrownVolumeResponseDto {
        crown_volume_m3_per_ha: volume,
    }))
}

// =============================================================================
// Forage Demand
// =============================================================================

pub async fn calculate_forage_demand(
    _auth: AuthUser,
    dto: web::Json<ForageDemandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let demand = CalculationService::calculate_forage_demand(
        dto.body_weight_kg,
        dto.demand_percent,
        dto.animal_count,
    );

    Ok(HttpResponse::Ok().json(ForageDemandResponseDto {
        daily_forage_dm_kg: demand,
    }))
}

// =============================================================================
// Nitrogen Demand
// =============================================================================

pub async fn calculate_nitrogen_demand(
    _auth: AuthUser,
    dto: web::Json<NitrogenDemandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let nitrogen = CalculationService::calculate_nitrogen_demand(dto.area_ha, dto.demand_per_ha);

    Ok(HttpResponse::Ok().json(NitrogenDemandResponseDto {
        total_nitrogen_kg: nitrogen,
    }))
}

// =============================================================================
// Difficulty Surcharge
// =============================================================================

pub async fn calculate_difficulty_surcharge(
    _auth: AuthUser,
    dto: web::Json<DifficultySurchargeRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let adjusted = CalculationService::calculate_difficulty_surcharge(
        dto.base_rate,
        dto.is_steep,
        dto.is_heavy_soil,
        dto.is_narrow,
    );

    let multiplier = adjusted / dto.base_rate;

    Ok(HttpResponse::Ok().json(DifficultySurchargeResponseDto {
        adjusted_rate: adjusted,
        multiplier,
    }))
}

// =============================================================================
// Profitability
// =============================================================================

pub async fn calculate_profitability(
    _auth: AuthUser,
    dto: web::Json<ProfitabilityCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let profit = CalculationService::calculate_profitability(
        dto.yield_amount,
        dto.price_per_unit,
        dto.material_costs,
        dto.labor_costs,
        dto.machinery_costs,
        dto.area_ha,
    );

    let revenue = dto.yield_amount * dto.price_per_unit;
    let total_costs = dto.material_costs + dto.labor_costs + dto.machinery_costs;

    Ok(
        HttpResponse::Ok().json(ProfitabilityCalculationResponseDto {
            profit_per_ha: profit,
            revenue_per_ha: revenue / dto.area_ha,
            costs_per_ha: total_costs / dto.area_ha,
        }),
    )
}

// =============================================================================
// Harvest Estimation
// =============================================================================

pub async fn estimate_harvest(
    _auth: AuthUser,
    dto: web::Json<HarvestEstimationRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let days = CalculationService::estimate_harvest_date(
        dto.current_bbch,
        dto.target_bbch,
        dto.avg_temp,
        dto.base_temp,
    );

    let daily_gdd = (dto.avg_temp - dto.base_temp).max(0.0);

    Ok(HttpResponse::Ok().json(HarvestEstimationResponseDto {
        days_to_harvest: days,
        current_bbch: dto.current_bbch,
        target_bbch: dto.target_bbch,
        daily_gdd,
    }))
}

// =============================================================================
// Workflow Follow-ups
// =============================================================================

pub async fn get_workflow_follow_ups(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<WorkflowFollowUpRequestDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_any_role(vec!["admin", "manager", "worker"])?;

    let order = state
        .db
        .order_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), dto.order_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Order not found".into()))?;

    // Parse status string to OrderStatus
    let next_status = match dto.next_status.as_str() {
        "Draft" => agrocore_domain::entities::OrderStatus::Draft,
        "Planned" => agrocore_domain::entities::OrderStatus::Planned,
        "InProgress" => agrocore_domain::entities::OrderStatus::InProgress,
        "Completed" => agrocore_domain::entities::OrderStatus::Completed,
        "Cancelled" => agrocore_domain::entities::OrderStatus::Cancelled,
        _ => return Err(ApiError::validation("Invalid status".to_string())),
    };

    let follow_ups = WorkflowService::process_status_transition(&order, next_status);

    let follow_up_dtos: Vec<FollowUpOrderDto> = follow_ups
        .into_iter()
        .map(|fu| FollowUpOrderDto {
            label: fu.label,
            order_type: fu.order_type.to_string(),
            site_ids: fu.site_ids,
            assigned_worker_ids: fu.assigned_worker_ids.unwrap_or_default(),
            planned_date: fu.planned_date,
            parent_order_id: fu.parent_order_id.unwrap_or(Uuid::nil()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(WorkflowFollowUpResponseDto {
        follow_up_orders: follow_up_dtos,
    }))
}

// =============================================================================
// Weather Providers
// =============================================================================

pub async fn fetch_weather(
    _state: web::Data<AppState>,
    _auth: AuthUser,
    dto: web::Json<WeatherFetchRequestDto>,
) -> Result<HttpResponse, ApiError> {
    dto.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let provider = dto.provider.as_deref().unwrap_or("openmeteo");
    let _service_type = match provider.to_lowercase().as_str() {
        "openweather" => WeatherServiceType::OpenWeather,
        "wunderground" | "weatherunderground" => WeatherServiceType::WeatherUnderground,
        _ => WeatherServiceType::OpenMeteo,
    };

    // For now, return mock data - in production this would call the actual provider
    let result = WeatherFetchResult {
        temperature_c: Some(20.5),
        humidity_percent: Some(65.0),
        precipitation_mm: Some(0.0),
        wind_speed_kmh: Some(12.0),
        wind_direction_deg: Some(270),
        solar_radiation_wm2: Some(800.0),
        pressure_hpa: Some(1013.25),
        soil_temperature_c: Some(18.0),
        soil_moisture_percent: Some(45.0),
        leaf_wetness: Some(false),
        timestamp: Utc::now(),
    };

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
        provider: provider.to_string(),
    }))
}

pub async fn list_weather_providers(
    _state: web::Data<AppState>,
    _auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let providers = vec![
        WeatherProviderInfoDto {
            id: "openmeteo".to_string(),
            name: "Open-Meteo".to_string(),
            requires_api_key: false,
            free_tier: true,
        },
        WeatherProviderInfoDto {
            id: "openweather".to_string(),
            name: "OpenWeather".to_string(),
            requires_api_key: true,
            free_tier: true,
        },
        WeatherProviderInfoDto {
            id: "wunderground".to_string(),
            name: "Weather Underground".to_string(),
            requires_api_key: true,
            free_tier: false,
        },
    ];

    Ok(HttpResponse::Ok().json(WeatherProvidersResponseDto { providers }))
}
