use crate::AppState;
use crate::dto::{
    ErrorResponse, FertilizerCalculationRequestDto, FertilizerCalculationResponseDto,
    FertilizerRecommendationDto, NutritionDemandRequestDto, NutritionDemandResponseDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::services::nutrition::{Fertilizer, NutrientValues, NutritionService};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/nutrition/demand").route(web::post().to(calculate_demand)))
        .service(
            web::resource("/nutrition/fertilizer-amount")
                .route(web::post().to(calculate_fertilizer_amount)),
        );
}

#[utoipa::path(
    post,
    path = "/api/v1/nutrition/demand",
    request_body = NutritionDemandRequestDto,
    responses(
        (status = 200, description = "Nutrient demand calculated", body = NutritionDemandResponseDto),
        (status = 404, description = "Site not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "nutrition",
    security(("bearer_auth" = []))
)]
pub async fn calculate_demand(
    _state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<NutritionDemandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    let _tenant_id = agrocore_domain::TenantId(auth.0.tenant_id);

    // Use the area from the DTO directly (no site lookup needed)
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
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/nutrition/fertilizer-amount",
    request_body = FertilizerCalculationRequestDto,
    responses(
        (status = 200, description = "Fertilizer amount calculated", body = FertilizerCalculationResponseDto),
        (status = 401, description = "Unauthorized")
    ),
    tag = "nutrition",
    security(("bearer_auth" = []))
)]
pub async fn calculate_fertilizer_amount(
    _auth: AuthUser,
    dto: web::Json<FertilizerCalculationRequestDto>,
) -> Result<HttpResponse, ApiError> {
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
        recommendations_dto.push(FertilizerRecommendationDto {
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
