use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NutrientValues {
    pub n: f64,  // Stickstoff (kg)
    pub p: f64,  // Phosphor (kg)
    pub k: f64,  // Kalium (kg)
    pub mg: f64, // Magnesium (kg)
}

impl NutrientValues {
    pub fn total_kg(&self) -> f64 {
        self.n + self.p + self.k + self.mg
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Fertilizer {
    pub name: String,
    pub nutrient_content_percent: NutrientValues, // Gehalt in % oder kg/100kg
}

pub struct NutritionService;

impl NutritionService {
    /// Berechnet den Nährstoffbedarf basierend auf Fläche und Zielertrag/Standardbedarf.
    pub fn calculate_demand(
        area_ha: f64,
        target_yield_t_ha: f64,
        demand_per_t: &NutrientValues,
    ) -> NutrientValues {
        NutrientValues {
            n: area_ha * target_yield_t_ha * demand_per_t.n,
            p: area_ha * target_yield_t_ha * demand_per_t.p,
            k: area_ha * target_yield_t_ha * demand_per_t.k,
            mg: area_ha * target_yield_t_ha * demand_per_t.mg,
        }
    }

    /// Berechnet die benötigte Düngermenge (kg) basierend auf dem Nährstoffbedarf und dem Düngergehalt.
    /// Nutzt primär den Stickstoff-Bedarf als Leitnährstoff.
    pub fn calculate_fertilizer_amount(demand: &NutrientValues, fertilizer: &Fertilizer) -> f64 {
        if fertilizer.nutrient_content_percent.n <= 0.0 {
            return 0.0;
        }
        (demand.n / fertilizer.nutrient_content_percent.n) * 100.0
    }

    /// Berechnet die Nährstoffbilanz nach der Ausbringung.
    pub fn calculate_balance(
        demand: &NutrientValues,
        applied_amount_kg: f64,
        fertilizer: &Fertilizer,
    ) -> NutrientValues {
        let applied_nutrients = NutrientValues {
            n: (applied_amount_kg * fertilizer.nutrient_content_percent.n) / 100.0,
            p: (applied_amount_kg * fertilizer.nutrient_content_percent.p) / 100.0,
            k: (applied_amount_kg * fertilizer.nutrient_content_percent.k) / 100.0,
            mg: (applied_amount_kg * fertilizer.nutrient_content_percent.mg) / 100.0,
        };

        NutrientValues {
            n: applied_nutrients.n - demand.n,
            p: applied_nutrients.p - demand.p,
            k: applied_nutrients.k - demand.k,
            mg: applied_nutrients.mg - demand.mg,
        }
    }

    /// Berechnet Düngerempfehlungen basierend auf Nährstoffbedarf und verfügbaren Düngern.
    pub fn calculate_fertilizer_recommendations(
        demand: &NutrientValues,
        fertilizers: &[Fertilizer],
        area_ha: f64,
    ) -> Vec<FertilizerRecommendation> {
        let mut recommendations = Vec::new();
        let mut remaining_demand = demand.clone();

        for fertilizer in fertilizers {
            if fertilizer.nutrient_content_percent.n <= 0.0 {
                continue;
            }

            let amount_per_ha = Self::calculate_fertilizer_amount(&remaining_demand, fertilizer);
            let total_amount = amount_per_ha * area_ha;

            let n_supplied = (total_amount * fertilizer.nutrient_content_percent.n) / 100.0;
            let p_supplied = (total_amount * fertilizer.nutrient_content_percent.p) / 100.0;
            let k_supplied = (total_amount * fertilizer.nutrient_content_percent.k) / 100.0;
            let mg_supplied = (total_amount * fertilizer.nutrient_content_percent.mg) / 100.0;

            recommendations.push(FertilizerRecommendation {
                fertilizer_name: fertilizer.name.clone(),
                amount_kg_per_ha: amount_per_ha,
                total_amount_kg: total_amount,
                nitrogen_supplied_kg: n_supplied,
                phosphorus_supplied_kg: p_supplied,
                potassium_supplied_kg: k_supplied,
                magnesium_supplied_kg: mg_supplied,
                cost_eur: total_amount * 0.5, // Placeholder cost
            });

            // Update remaining demand
            remaining_demand = NutrientValues {
                n: (remaining_demand.n - n_supplied).max(0.0),
                p: (remaining_demand.p - p_supplied).max(0.0),
                k: (remaining_demand.k - k_supplied).max(0.0),
                mg: (remaining_demand.mg - mg_supplied).max(0.0),
            };

            // Stop if N demand is met
            if remaining_demand.n <= 0.0 {
                break;
            }
        }

        recommendations
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FertilizerRecommendation {
    pub fertilizer_name: String,
    pub amount_kg_per_ha: f64,
    pub total_amount_kg: f64,
    pub nitrogen_supplied_kg: f64,
    pub phosphorus_supplied_kg: f64,
    pub potassium_supplied_kg: f64,
    pub magnesium_supplied_kg: f64,
    pub cost_eur: f64,
}
