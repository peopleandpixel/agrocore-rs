use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct NutrientValues {
    pub n: f64, // Stickstoff (kg)
    pub p: f64, // Phosphor (kg)
    pub k: f64, // Kalium (kg)
    pub mg: f64, // Magnesium (kg)
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
    pub fn calculate_fertilizer_amount(
        demand: &NutrientValues,
        fertilizer: &Fertilizer,
    ) -> f64 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_demand() {
        let demand_per_t = NutrientValues { n: 2.5, p: 0.8, k: 3.0, mg: 0.5 };
        let demand = NutritionService::calculate_demand(2.0, 10.0, &demand_per_t);
        
        assert_eq!(demand.n, 50.0);
        assert_eq!(demand.p, 16.0);
        assert_eq!(demand.k, 60.0);
        assert_eq!(demand.mg, 10.0);
    }

    #[test]
    fn test_calculate_fertilizer_amount() {
        let demand = NutrientValues { n: 50.0, p: 16.0, k: 60.0, mg: 10.0 };
        let fertilizer = Fertilizer {
            name: "KAS".into(),
            nutrient_content_percent: NutrientValues { n: 27.0, p: 0.0, k: 0.0, mg: 0.0 },
        };
        
        let amount = NutritionService::calculate_fertilizer_amount(&demand, &fertilizer);
        // (50 / 27) * 100 = 185.185...
        assert!(amount > 185.1 && amount < 185.2);
    }

    #[test]
    fn test_calculate_balance() {
        let demand = NutrientValues { n: 50.0, p: 16.0, k: 60.0, mg: 10.0 };
        let fertilizer = Fertilizer {
            name: "NPK 15-15-15".into(),
            nutrient_content_percent: NutrientValues { n: 15.0, p: 15.0, k: 15.0, mg: 0.0 },
        };
        
        // Bringe 400kg aus -> 60kg N, 60kg P, 60kg K
        let balance = NutritionService::calculate_balance(&demand, 400.0, &fertilizer);
        
        assert_eq!(balance.n, 10.0);
        assert_eq!(balance.p, 44.0);
        assert_eq!(balance.k, 0.0);
        assert_eq!(balance.mg, -10.0);
    }
}
