//! Calculation DTOs (Nutrition, Water, Material, etc.)

use agrocore_domain::entities::CropType;
use agrocore_domain::entities::plant_protection::PlantProtectionAreaMethod;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// Nutrition Calculation DTOs
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct NutritionDemandRequestDto {
    pub crop_type: CropType,
    pub expected_yield: f64,
    pub area_ha: f64,
    pub soil_nitrogen: Option<f64>,
    pub soil_phosphorus: Option<f64>,
    pub soil_potassium: Option<f64>,
    pub organic_matter_percent: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct NutritionDemandResponseDto {
    pub nitrogen_kg_per_ha: f64,
    pub phosphorus_kg_per_ha: f64,
    pub potassium_kg_per_ha: f64,
    pub magnesium_kg_per_ha: f64,
    pub sulfur_kg_per_ha: f64,
    pub calcium_kg_per_ha: f64,
    pub total_nitrogen_kg: f64,
    pub total_phosphorus_kg: f64,
    pub total_potassium_kg: f64,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct FertilizerCalculationRequestDto {
    pub nutrition_demand: NutritionDemandResponseDto,
    pub fertilizer_types: Vec<FertilizerInputDto>,
    pub area_ha: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FertilizerCalculationResponseDto {
    pub recommendations: Vec<FertilizerRecommendationDto>,
    pub total_cost_eur: f64,
    pub total_nitrogen_kg: f64,
    pub total_phosphorus_kg: f64,
    pub total_potassium_kg: f64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FertilizerInputDto {
    pub name: String,
    pub nitrogen_percent: f64,
    pub phosphorus_percent: f64,
    pub potassium_percent: f64,
    pub cost_per_ton_eur: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FertilizerRecommendationDto {
    pub fertilizer_name: String,
    pub amount_kg_per_ha: f64,
    pub total_amount_kg: f64,
    pub nitrogen_supplied_kg: f64,
    pub phosphorus_supplied_kg: f64,
    pub potassium_supplied_kg: f64,
    pub cost_eur: f64,
}

// =============================================================================
// Water Rate Calculation DTOs
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct WaterRateCalculationRequestDto {
    #[validate(range(min = 0.1))]
    pub speed_kmh: f64,
    #[validate(range(min = 0.1))]
    pub nozzle_flow_lmin: f64,
    #[validate(range(min = 0.1))]
    pub lane_width: f64,
    #[validate(range(min = 1))]
    pub number_of_nozzles: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WaterRateCalculationResponseDto {
    pub water_rate_lha: f64,
}

// =============================================================================
// Material Calculation DTOs
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct MaterialCalculationRequestDto {
    pub method: PlantProtectionAreaMethod,
    pub site_id: Uuid,
    #[validate(range(min = 0.0))]
    pub dose_per_ha: f64,
    pub application_date: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MaterialCalculationResponseDto {
    pub treated_area_ha: f64,
    pub total_material_amount: f64,
}
