use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::{BbchStage, CropType, SiteType};
use crate::repositories::VisibilityAwareEntity;

#[cfg(feature = "mongodb")]
use crate::entities::user::UserRole;
#[cfg(feature = "mongodb")]
use mongodb::bson::{doc, Document};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct SigpacData {
    pub province: u8,
    pub municipality: u16,
    pub aggregate: u16,
    pub zone: u16,
    pub polygon: u16,
    pub parcel: u16,
    pub enclosure: u16,
    pub usage_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct GeoPoint {
    #[validate(range(min = -180.0, max = 180.0))]
    pub lng: f64,
    #[validate(range(min = -90.0, max = 90.0))]
    pub lat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Plot {
    pub id: Uuid,
    pub label: String,
    #[validate(range(min = 0.0))]
    pub area: f64,
    pub boundary: Option<Vec<GeoPoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RowConfig {
    #[validate(range(min = 0.0))]
    pub stick_distance: f64,
    #[validate(range(min = 0.0))]
    pub lane_width: f64,
    #[validate(range(min = 0))]
    pub number_of_rows: u32,
    #[validate(range(min = 0.0))]
    pub avg_strike_length: f64,
    #[validate(range(min = 0.0))]
    pub total_strike_length: f64,
    #[validate(range(min = 0))]
    pub number_of_vines: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct SiteProperty {
    pub key: String,
    pub value: serde_json::Value,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Site {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub business_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    #[validate(range(min = 0.0))]
    pub gross_area: Option<f64>,
    pub plots: Vec<Plot>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<BbchStage>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: bool,
    pub is_temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Site {
    pub const PROP_GRAZING_QUALITY: &'static str = "grazing_quality";
    pub const PROP_FORAGE_DEMAND: &'static str = "forage_demand";
    pub const PROP_GRASS_SPECIES: &'static str = "grass_species";
    pub const PROP_CROWN_DIAMETER: &'static str = "crown_diameter";
    pub const PROP_TREE_HEIGHT: &'static str = "tree_height";

    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties
            .as_ref()?
            .iter()
            .find(|p| p.key == key)
            .map(|p| &p.value)
    }

    pub fn get_property_as_f64(&self, key: &str) -> Option<f64> {
        self.get_property(key)?.as_f64()
    }

    pub fn get_property_as_str(&self, key: &str) -> Option<&str> {
        self.get_property(key)?.as_str()
    }
}

impl VisibilityAwareEntity for Site {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(user_id: Uuid, roles: &[UserRole]) -> Document {
        if roles.contains(&UserRole::Admin)
            || roles.contains(&UserRole::Manager)
            || roles.contains(&UserRole::Worker)
        {
            doc! {}
        } else {
            // Viewer can only see sites assigned to them (if any)
            doc! { "assigned_user_ids": user_id.to_string() }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    #[validate(range(min = 0.0))]
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<BbchStage>,
    pub planted_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateSiteDto {
    pub label: Option<String>,
    pub variety: Option<String>,
    pub area: Option<f64>,
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<BbchStage>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Vec<GeoPoint>>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn sample_site() -> Site {
        Site {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            business_id: None,
            label: String::from("North Field"),
            site_type: SiteType::Field,
            crop_type: CropType::Grape,
            variety: Some(String::from("Syrah")),
            area: 12.5,
            gross_area: Some(13.0),
            plots: vec![],
            row_config: None,
            bbch_stage: None,
            planted_date: None,
            cleared_date: None,
            soil_type: Some(String::from("Loam")),
            slope: Some(4.0),
            slope_facing: Some(String::from("South")),
            altitude: Some(250.0),
            organic: Some(true),
            organic_eligible: None,
            center: None,
            sigpac_data: None,
            regepac_id: None,
            boundary: None,
            properties: Some(vec![
                SiteProperty {
                    key: String::from("soil_ph"),
                    value: json!(6.4),
                    group: Some(String::from("soil")),
                },
                SiteProperty {
                    key: String::from("note"),
                    value: json!("north block"),
                    group: None,
                },
            ]),
            custom_fields: None,
            note1: None,
            note2: None,
            is_active: true,
            is_temporary: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn site_property_helpers_return_expected_values() {
        let site = sample_site();
        assert_eq!(site.get_property_as_f64("soil_ph"), Some(6.4));
        assert_eq!(site.get_property_as_str("note"), Some("north block"));
        assert!(site.get_property("missing").is_none());
    }
}
