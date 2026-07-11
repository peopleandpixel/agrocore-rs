use crate::AppState;
use crate::dto::{
    ErrorResponse, FertilizerCalculationRequestDto, FertilizerCalculationResponseDto,
    NutritionDemandRequestDto, NutritionDemandResponseDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::services::nutrition::NutritionService;
use agrocore_shared::SharedError;

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
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<NutritionDemandRequestDto>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = auth.0.tenant_id;

    let site = state
        .db
        .site_repo()
        .find_by_id(tenant_id, dto.site_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Site not found".into()))?;

    let total_demand =
        NutritionService::calculate_demand(site.area, dto.target_yield_t_ha, &dto.demand_per_t);

    Ok(HttpResponse::Ok().json(NutritionDemandResponseDto { total_demand }))
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
    let amount = NutritionService::calculate_fertilizer_amount(&dto.demand, &dto.fertilizer);

    Ok(HttpResponse::Ok().json(FertilizerCalculationResponseDto {
        fertilizer_amount_kg: amount,
    }))
}
