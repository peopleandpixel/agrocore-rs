use actix_web::{web, HttpResponse, Responder};
use crate::AppState;
use crate::middleware::AuthExtractor as AuthUser;
use crate::dto::{
    ErrorResponse, NutritionDemandRequestDto, NutritionDemandResponseDto,
    FertilizerCalculationRequestDto, FertilizerCalculationResponseDto
};
use agrocore_domain::services::nutrition::NutritionService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/nutrition/demand")
            .route(web::post().to(calculate_demand))
    ).service(
        web::resource("/nutrition/fertilizer-amount")
            .route(web::post().to(calculate_fertilizer_amount))
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
) -> impl Responder {
    let tenant_id = auth.0.tenant_id.into();
    
    let site = match state.db.site_repo().find_by_id(tenant_id, dto.site_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return HttpResponse::NotFound().json(ErrorResponse { 
            error: "not_found".into(), 
            message: "Site not found".into() 
        }),
        Err(e) => return HttpResponse::InternalServerError().json(ErrorResponse { 
            error: "internal".into(), 
            message: e.to_string() 
        }),
    };

    let total_demand = NutritionService::calculate_demand(
        site.area,
        dto.target_yield_t_ha,
        &dto.demand_per_t
    );

    HttpResponse::Ok().json(NutritionDemandResponseDto {
        total_demand,
    })
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
) -> impl Responder {
    let amount = NutritionService::calculate_fertilizer_amount(
        &dto.demand,
        &dto.fertilizer
    );

    HttpResponse::Ok().json(FertilizerCalculationResponseDto {
        fertilizer_amount_kg: amount,
    })
}
