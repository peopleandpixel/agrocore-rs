//! Calculation DTOs (Nutrition, Water, Material, Tree, Forage, Nitrogen, Difficulty, Profitability)

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
    pub area_ha: f64,
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
// Nutrition Balance DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct NutritionBalanceRequestDto {
    pub demand: NutritionDemandResponseDto,
    pub applied_amount_kg: f64,
    pub fertilizer: FertilizerInputDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NutritionBalanceResponseDto {
    pub nitrogen_balance_kg: f64,
    pub phosphorus_balance_kg: f64,
    pub potassium_balance_kg: f64,
    pub magnesium_balance_kg: f64,
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

// =============================================================================
// Tree Crown Volume Calculation DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct TreeCrownVolumeRequestDto {
    #[validate(range(min = 0.01))]
    pub crown_diameter: f64,
    #[validate(range(min = 0.01))]
    pub tree_height: f64,
    #[validate(range(min = 1))]
    pub trees_per_ha: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TreeCrownVolumeResponseDto {
    pub crown_volume_m3_per_ha: f64,
}

// =============================================================================
// Forage Demand Calculation DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ForageDemandRequestDto {
    #[validate(range(min = 0.1))]
    pub body_weight_kg: f64,
    #[validate(range(min = 0.1, max = 100.0))]
    pub demand_percent: f64,
    #[validate(range(min = 1))]
    pub animal_count: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ForageDemandResponseDto {
    pub daily_forage_dm_kg: f64,
}

// =============================================================================
// Nitrogen Demand Calculation DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct NitrogenDemandRequestDto {
    #[validate(range(min = 0.01))]
    pub area_ha: f64,
    #[validate(range(min = 0.1))]
    pub demand_per_ha: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NitrogenDemandResponseDto {
    pub total_nitrogen_kg: f64,
}

// =============================================================================
// Difficulty Surcharge Calculation DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct DifficultySurchargeRequestDto {
    #[validate(range(min = 0.01))]
    pub base_rate: f64,
    pub is_steep: bool,
    pub is_heavy_soil: bool,
    pub is_narrow: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DifficultySurchargeResponseDto {
    pub adjusted_rate: f64,
    pub multiplier: f64,
}

// =============================================================================
// Profitability Calculation DTOs (NEW - standalone)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ProfitabilityCalculationRequestDto {
    #[validate(range(min = 0.01))]
    pub yield_amount: f64,
    #[validate(range(min = 0.01))]
    pub price_per_unit: f64,
    pub material_costs: f64,
    pub labor_costs: f64,
    pub machinery_costs: f64,
    #[validate(range(min = 0.01))]
    pub area_ha: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfitabilityCalculationResponseDto {
    pub profit_per_ha: f64,
    pub revenue_per_ha: f64,
    pub costs_per_ha: f64,
}

// =============================================================================
// Harvest Estimation DTOs (NEW - standalone)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct HarvestEstimationRequestDto {
    #[validate(range(min = 0, max = 99))]
    pub current_bbch: u32,
    #[validate(range(min = 0, max = 99))]
    pub target_bbch: u32,
    #[validate(range(min = -50.0, max = 60.0))]
    pub avg_temp: f64,
    #[validate(range(min = -50.0, max = 60.0))]
    pub base_temp: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HarvestEstimationResponseDto {
    pub days_to_harvest: Option<u32>,
    pub current_bbch: u32,
    pub target_bbch: u32,
    pub daily_gdd: f64,
}

// =============================================================================
// Workflow Follow-ups DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct WorkflowFollowUpRequestDto {
    pub order_id: Uuid,
    pub next_status: String, // OrderStatus as string
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkflowFollowUpResponseDto {
    pub follow_up_orders: Vec<FollowUpOrderDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FollowUpOrderDto {
    pub label: String,
    pub order_type: String,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<chrono::DateTime<chrono::Utc>>,
    pub parent_order_id: Uuid,
}

// =============================================================================
// Weather Provider DTOs (NEW)
// =============================================================================

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct WeatherFetchRequestDto {
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: f64,
    #[validate(range(min = -180.0, max = 180.0))]
    pub longitude: f64,
    pub provider: Option<String>, // "openmeteo", "openweather", "wunderground"
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WeatherFetchResultDto {
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_direction_deg: Option<i32>,
    pub solar_radiation_wm2: Option<f64>,
    pub pressure_hpa: Option<f64>,
    pub soil_temperature_c: Option<f64>,
    pub soil_moisture_percent: Option<f64>,
    pub leaf_wetness: Option<bool>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub provider: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WeatherProviderInfoDto {
    pub id: String,
    pub name: String,
    pub requires_api_key: bool,
    pub free_tier: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WeatherProvidersResponseDto {
    pub providers: Vec<WeatherProviderInfoDto>,
}
