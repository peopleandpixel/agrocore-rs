use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

use crate::entities::tenant::TenantId;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PlantProtectionRecord {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    pub product_name: String,
    pub active_substance: String,
    pub dosage_per_ha: f64,
    pub total_quantity: f64,
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub pre_harvest_days: u32,
    pub re_entry_days: u32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum PlantProtectionAreaMethod {
    #[serde(rename = "net_area")]
    NetArea,
    #[serde(rename = "gross_area")]
    GrossArea,
    #[serde(rename = "leaf_wall_measured")]
    LeafWallMeasured,
    #[serde(rename = "leaf_wall_gross")]
    LeafWallGross,
    #[serde(rename = "ground_protection")]
    GroundProtection,
    #[serde(rename = "herbicide_protection")]
    HerbicideProtection,
    #[serde(rename = "tree_crown_volume")]
    TreeCrownVolume,
}

impl PlantProtectionAreaMethod {
    pub fn calculate_treated_area(
        &self,
        net_area: f64,
        gross_area: Option<f64>,
        lane_width: Option<f64>,
        total_strike_length: Option<f64>,
        is_steep: bool,
        application_date: DateTime<Utc>,
    ) -> f64 {
        let base_area = match self {
            Self::NetArea => net_area,
            Self::GrossArea => gross_area.unwrap_or(net_area),
            Self::LeafWallMeasured => {
                if let (Some(lw), Some(tsl)) = (lane_width, total_strike_length) {
                    if lw > 0.0 {
                        tsl * lw / 10000.0 // length * width in m^2 / 10000 = ha
                    } else {
                        net_area
                    }
                } else {
                    net_area
                }
            }
            Self::LeafWallGross => {
                // Leaf Wall Gross often considers a standard width or 
                // is derived from the net area with a crop-specific factor.
                // For now, we use a factor of 1.2 if not otherwise defined,
                // but prioritize gross_area if available.
                gross_area.unwrap_or(net_area * 1.2)
            }
            Self::GroundProtection => {
                // Ground protection (e.g. greening fertilization) often targets 
                // only the lanes, not the area under the plants.
                if let Some(lw) = lane_width {
                    if lw > 0.0 {
                        // Assume 1.5m spreader width or lane minus plant strip
                        // For generic approach, we take 0.7 of net_area if not specified
                        net_area * 0.7
                    } else {
                        net_area
                    }
                } else {
                    net_area
                }
            }
            Self::HerbicideProtection => {
                // Usually only a strip under the plants is treated.
                // Standard factor 0.3, or calculated if lane_width is known.
                if let Some(lw) = lane_width {
                    if lw > 0.0 {
                        // Assume 0.5m herbicide strip width
                        net_area * (0.5 / lw)
                    } else {
                        net_area * 0.3
                    }
                } else {
                    net_area * 0.3
                }
            }
            Self::TreeCrownVolume => {
                // Specialized logic for tree crops (Olives, Cork, Almonds).
                // Often based on leaf wall area or tree-specific volume metrics.
                // For a quick-win, we treat it similar to LeafWallMeasured 
                // but with an additional vertical factor (default 1.0 ha-equivalent).
                if let (Some(lw), Some(tsl)) = (lane_width, total_strike_length) {
                    if lw > 0.0 {
                        tsl * lw / 10000.0
                    } else {
                        net_area
                    }
                } else {
                    net_area
                }
            }
        };

        if is_steep {
            let threshold_2024_01_01 = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc);
            let threshold_2024_07_11 = DateTime::parse_from_rfc3339("2024-07-11T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc);

            let factor = if application_date < threshold_2024_01_01 {
                1.25
            } else if application_date < threshold_2024_07_11 {
                1.15
            } else {
                1.15 // Default for newer orders
            };
            base_area * factor
        } else {
            base_area
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_treated_area_net() {
        let method = PlantProtectionAreaMethod::NetArea;
        let area = 1.0;
        let date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(method.calculate_treated_area(area, None, None, None, false, date), 1.0);
        assert_eq!(method.calculate_treated_area(area, None, None, None, true, date), 1.25);
    }

    #[test]
    fn test_calculate_treated_area_gross() {
        let method = PlantProtectionAreaMethod::GrossArea;
        let net_area = 1.0;
        let gross_area = 1.2;
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(method.calculate_treated_area(net_area, Some(gross_area), None, None, false, date), 1.2);
        assert_eq!(method.calculate_treated_area(net_area, None, None, None, false, date), 1.0);
    }

    #[test]
    fn test_calculate_treated_area_herbicide() {
        let method = PlantProtectionAreaMethod::HerbicideProtection;
        let area = 1.0;
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(method.calculate_treated_area(area, None, None, None, false, date), 0.3);
        // With lane width 2.0m, 0.5m strip -> 0.5/2.0 = 0.25
        assert_eq!(method.calculate_treated_area(area, None, Some(2.0), None, false, date), 0.25);
    }

    #[test]
    fn test_calculate_treated_area_ground() {
        let method = PlantProtectionAreaMethod::GroundProtection;
        let area = 1.0;
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(method.calculate_treated_area(area, None, None, None, false, date), 1.0);
        assert_eq!(method.calculate_treated_area(area, None, Some(2.0), None, false, date), 0.7);
    }

    #[test]
    fn test_calculate_treated_area_tree_crown() {
        let method = PlantProtectionAreaMethod::TreeCrownVolume;
        let area = 1.0;
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        // Currently same as LeafWallMeasured logic in the basic implementation
        assert_eq!(method.calculate_treated_area(area, None, Some(2.0), Some(5000.0), false, date), 1.0);
    }

    #[test]
    fn test_calculate_treated_area_leaf_wall() {
        let method = PlantProtectionAreaMethod::LeafWallMeasured;
        let area = 1.0;
        let lane_width = 2.0; // meters
        let strike_length = 5000.0; // meters
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        // 5000 * 2 = 10000 m2 = 1.0 ha
        assert_eq!(method.calculate_treated_area(area, None, Some(lane_width), Some(strike_length), false, date), 1.0);
    }

    #[test]
    fn test_steepness_factors() {
        let method = PlantProtectionAreaMethod::NetArea;
        let area = 1.0;
        
        let old_date = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();
        let mid_date = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
        let new_date = Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap();

        assert_eq!(method.calculate_treated_area(area, None, None, None, true, old_date), 1.25);
        assert_eq!(method.calculate_treated_area(area, None, None, None, true, mid_date), 1.15);
        assert_eq!(method.calculate_treated_area(area, None, None, None, true, new_date), 1.15);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePlantProtectionDto {
    pub site_id: Uuid,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub product_name: String,
    #[validate(length(min = 1))]
    pub active_substance: String,
    #[validate(range(min = 0.0))]
    pub dosage_per_ha: f64,
    #[validate(range(min = 0.0))]
    pub total_quantity: f64,
    #[validate(range(min = 0.0))]
    pub area_ha: f64,
    pub application_date: DateTime<Utc>,
    pub pre_harvest_days: u32,
    pub re_entry_days: u32,
    pub weather_conditions: Option<String>,
    pub applicator_license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplicatorLicense {
    pub id: Uuid,
    pub user_id: Uuid,
    pub license_type: LicenseType,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LicenseType {
    Basic,
    Advanced,
    Professional,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateApplicatorLicenseDto {
    pub user_id: Uuid,
    pub license_type: LicenseType,
    #[validate(length(min = 1))]
    pub license_number: String,
    #[validate(length(min = 1))]
    pub issued_by: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}
