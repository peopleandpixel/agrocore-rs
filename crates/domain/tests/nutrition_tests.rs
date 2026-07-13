use agrocore_domain::services::nutrition::{Fertilizer, NutritionService, NutrientValues};

#[test]
fn test_calculate_demand() {
    let demand_per_t = NutrientValues {
        n: 2.5,
        p: 0.8,
        k: 3.0,
        mg: 0.5,
    };
    let demand = NutritionService::calculate_demand(2.0, 10.0, &demand_per_t);

    assert_eq!(demand.n, 50.0);
    assert_eq!(demand.p, 16.0);
    assert_eq!(demand.k, 60.0);
    assert_eq!(demand.mg, 10.0);
}

#[test]
fn test_calculate_fertilizer_amount() {
    let demand = NutrientValues {
        n: 50.0,
        p: 16.0,
        k: 60.0,
        mg: 10.0,
    };
    let fertilizer = Fertilizer {
        name: "KAS".into(),
        nutrient_content_percent: NutrientValues {
            n: 27.0,
            p: 0.0,
            k: 0.0,
            mg: 0.0,
        },
    };

    let amount = NutritionService::calculate_fertilizer_amount(&demand, &fertilizer);
    // (50 / 27) * 100 = 185.185...
    assert!(amount > 185.1 && amount < 185.2);
}

#[test]
fn test_calculate_balance() {
    let demand = NutrientValues {
        n: 50.0,
        p: 16.0,
        k: 60.0,
        mg: 10.0,
    };
    let fertilizer = Fertilizer {
        name: "NPK 15-15-15".into(),
        nutrient_content_percent: NutrientValues {
            n: 15.0,
            p: 15.0,
            k: 15.0,
            mg: 0.0,
        },
    };

    // Bringe 400kg aus -> 60kg N, 60kg P, 60kg K
    let balance = NutritionService::calculate_balance(&demand, 400.0, &fertilizer);

    assert_eq!(balance.n, 10.0);
    assert_eq!(balance.p, 44.0);
    assert_eq!(balance.k, 0.0);
    assert_eq!(balance.mg, -10.0);
}

#[test]
fn test_calculate_fertilizer_amount_handles_zero_n() {
    let demand = NutrientValues {
        n: 50.0,
        p: 0.0,
        k: 0.0,
        mg: 0.0,
    };
    let fertilizer = Fertilizer {
        name: "Water".into(),
        nutrient_content_percent: NutrientValues {
            n: 0.0,
            p: 0.0,
            k: 0.0,
            mg: 0.0,
        },
    };

    assert_eq!(
        NutritionService::calculate_fertilizer_amount(&demand, &fertilizer),
        0.0
    );
}

#[test]
fn test_calculate_demand_scales_all_nutrients() {
    let demand_per_t = NutrientValues {
        n: 1.0,
        p: 2.0,
        k: 3.0,
        mg: 4.0,
    };
    let demand = NutritionService::calculate_demand(3.0, 4.0, &demand_per_t);
    assert_eq!(demand.n, 12.0);
    assert_eq!(demand.p, 24.0);
    assert_eq!(demand.k, 36.0);
    assert_eq!(demand.mg, 48.0);
}