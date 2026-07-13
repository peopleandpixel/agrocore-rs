use agrocore_domain::entities::plant_protection::PlantProtectionAreaMethod;
use agrocore_domain::services::calculation::CalculationService;
use agrocore_domain::services::calculation::MaterialAmountRequest;
use chrono::TimeZone;
use chrono::Utc;

#[test]
fn test_calculate_material_amount() {
    let method = PlantProtectionAreaMethod::NetArea;
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let amount = CalculationService::calculate_material_amount(MaterialAmountRequest {
        method,
        net_area: 1.0,
        gross_area: None,
        lane_width: None,
        total_strike_length: None,
        is_steep: false,
        dosage_per_ha: 5.0,
        application_date: date,
    });
    assert_eq!(amount, 5.0);
}

#[test]
fn test_calculate_water_rate() {
    // Beispiel: 6 km/h, 1.5 L/min pro Düse, 2m Reihenbreite, 10 Düsen
    // (15 * 600) / (6 * 2) = 9000 / 12 = 750 L/ha
    let rate = CalculationService::calculate_water_rate(6.0, 1.5, 2.0, 10);
    assert_eq!(rate, 750.0);
}

#[test]
fn test_calculate_water_rate_rejects_zero_values() {
    assert_eq!(
        CalculationService::calculate_water_rate(0.0, 1.5, 2.0, 10),
        0.0
    );
    assert_eq!(
        CalculationService::calculate_water_rate(6.0, 1.5, 0.0, 10),
        0.0
    );
}

#[test]
fn test_calculate_tree_crown_volume() {
    // 3m Durchmesser, 4m Höhe, 400 Bäume/ha
    // Fläche = 3² * PI / 4 = 7.0685...
    // Volumen = 7.0685 * 4 * 400 = 11309.73...
    let vol = CalculationService::calculate_tree_crown_volume(3.0, 4.0, 400);
    assert!(vol > 11309.0 && vol < 11310.0);
}

#[test]
fn test_calculate_tree_crown_volume_rejects_zero_values() {
    assert_eq!(
        CalculationService::calculate_tree_crown_volume(0.0, 4.0, 400),
        0.0
    );
    assert_eq!(
        CalculationService::calculate_tree_crown_volume(3.0, 0.0, 400),
        0.0
    );
}

#[test]
fn test_calculate_forage_demand() {
    // 500kg Kuh, 3% Bedarf, 10 Tiere
    // 500 * 0.03 * 10 = 150kg
    let demand = CalculationService::calculate_forage_demand(500.0, 3.0, 10);
    assert_eq!(demand, 150.0);
}

#[test]
fn test_calculate_forage_demand_rejects_zero_values() {
    assert_eq!(
        CalculationService::calculate_forage_demand(0.0, 3.0, 10),
        0.0
    );
    assert_eq!(
        CalculationService::calculate_forage_demand(500.0, 0.0, 10),
        0.0
    );
}

#[test]
fn test_calculate_nitrogen_demand() {
    // 2.5 ha, 140kg N/ha
    let n_demand = CalculationService::calculate_nitrogen_demand(2.5, 140.0);
    assert_eq!(n_demand, 350.0);
}

#[test]
fn test_calculate_nitrogen_demand_rejects_zero_values() {
    assert_eq!(
        CalculationService::calculate_nitrogen_demand(0.0, 140.0),
        0.0
    );
    assert_eq!(CalculationService::calculate_nitrogen_demand(2.5, 0.0), 0.0);
}

#[test]
fn test_calculate_difficulty_surcharge() {
    // Basis 100€, steil + schwerer Boden
    // 100 * (1 + 0.3 + 0.15) = 145
    let price = CalculationService::calculate_difficulty_surcharge(100.0, true, true, false);
    assert_eq!(price, 145.0);
}

#[test]
fn test_calculate_difficulty_surcharge_all_factors() {
    let price = CalculationService::calculate_difficulty_surcharge(100.0, true, true, true);
    assert_eq!(price, 155.0);
}

#[test]
fn test_estimate_harvest_date() {
    // BBCH 75 -> 89 (14 Punkte). 20 Grad, Basis 10 Grad -> 10 GDD/Tag.
    // 14 * 15 / 10 = 21 Tage.
    let days = CalculationService::estimate_harvest_date(75, 89, 20.0, 10.0);
    assert_eq!(days, Some(21));
}

#[test]
fn test_estimate_harvest_date_handles_no_growth_or_finished_state() {
    assert_eq!(
        CalculationService::estimate_harvest_date(89, 89, 20.0, 10.0),
        Some(0)
    );
    assert_eq!(
        CalculationService::estimate_harvest_date(75, 89, 5.0, 10.0),
        None
    );
}

#[test]
fn test_calculate_profitability_handles_zero_area() {
    assert_eq!(
        CalculationService::calculate_profitability(100.0, 2.0, 50.0, 50.0, 50.0, 0.0),
        0.0
    );
}