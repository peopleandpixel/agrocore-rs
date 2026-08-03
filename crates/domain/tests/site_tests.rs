use agrocore_domain::entities::site::{Site, SiteProperty};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_shared::lpis::LpisCountry;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

fn sample_site() -> Site {
    Site {
        id: Uuid::new_v4(),
        tenant_id: TenantId(Uuid::new_v4()),
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
        lpis_country: Some(LpisCountry::Es),
        lpis_data: None,
        #[allow(deprecated)]
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
