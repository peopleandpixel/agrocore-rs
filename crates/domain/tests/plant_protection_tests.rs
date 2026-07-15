use agrocore_domain::entities::plant_protection::PlantProtectionAreaMethod;
use chrono::{TimeZone, Utc};

#[test]
fn test_calculate_treated_area_net() {
    let method = PlantProtectionAreaMethod::NetArea;
    let area = 1.0;
    let date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, false, date),
        1.0
    );
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, true, date),
        1.25
    );
}

#[test]
fn test_calculate_treated_area_gross() {
    let method = PlantProtectionAreaMethod::GrossArea;
    let net_area = 1.0;
    let gross_area = 1.2;
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        method.calculate_treated_area(net_area, Some(gross_area), None, None, false, date),
        1.2
    );
    assert_eq!(
        method.calculate_treated_area(net_area, None, None, None, false, date),
        1.0
    );
}

#[test]
fn test_calculate_treated_area_herbicide() {
    let method = PlantProtectionAreaMethod::HerbicideProtection;
    let area = 1.0;
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, false, date),
        0.3
    );
    // With lane width 2.0m, 0.5m strip -> 0.5/2.0 = 0.25
    assert_eq!(
        method.calculate_treated_area(area, None, Some(2.0), None, false, date),
        0.25
    );
}

#[test]
fn test_calculate_treated_area_ground() {
    let method = PlantProtectionAreaMethod::GroundProtection;
    let area = 1.0;
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, false, date),
        1.0
    );
    assert_eq!(
        method.calculate_treated_area(area, None, Some(2.0), None, false, date),
        0.7
    );
}

#[test]
fn test_calculate_treated_area_tree_crown() {
    let method = PlantProtectionAreaMethod::TreeCrownVolume;
    let area = 1.0;
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    // Currently same as LeafWallMeasured logic in the basic implementation
    assert_eq!(
        method.calculate_treated_area(area, None, Some(2.0), Some(5000.0), false, date),
        1.0
    );
}

#[test]
fn test_calculate_treated_area_leaf_wall() {
    let method = PlantProtectionAreaMethod::LeafWallMeasured;
    let area = 1.0;
    let lane_width = 2.0; // meters
    let strike_length = 5000.0; // meters
    let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    // 5000 * 2 = 10000 m2 = 1.0 ha
    assert_eq!(
        method.calculate_treated_area(
            area,
            None,
            Some(lane_width),
            Some(strike_length),
            false,
            date
        ),
        1.0
    );
}

#[test]
fn test_steepness_factors() {
    let method = PlantProtectionAreaMethod::NetArea;
    let area = 1.0;

    let old_date = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
    let mid_date = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
    let new_date = Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap();

    assert_eq!(
        method.calculate_treated_area(area, None, None, None, true, old_date),
        1.25
    );
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, true, mid_date),
        1.15
    );
    assert_eq!(
        method.calculate_treated_area(area, None, None, None, true, new_date),
        1.15
    );
}
