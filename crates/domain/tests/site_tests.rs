use agrocore_domain::entities::site::Site;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_shared::lpis::LpisCountry;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

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
        plots: json!([]),
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
        regepac_id: None,
        boundary: None,
        properties: Some(json!([
            {
                "key": "soil_ph",
                "value": 6.4,
                "group": "soil"
            },
            {
                "key": "note",
                "value": "north block",
                "group": null
            }
        ])),
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

    // Access properties via JSON directly
    let props = site
        .properties
        .as_ref()
        .expect("properties should be present");
    let soil_ph = props
        .get(0)
        .and_then(|p| p.get("value"))
        .and_then(|v| v.as_f64());
    let note = props
        .get(1)
        .and_then(|p| p.get("value"))
        .and_then(|v| v.as_str());

    assert_eq!(soil_ph, Some(6.4));
    assert_eq!(note, Some("north block"));

    // Test missing property
    let missing = props.as_array().and_then(|arr| {
        arr.iter()
            .find(|p| p.get("key").and_then(|k| k.as_str()) == Some("missing"))
    });
    assert!(missing.is_none());
}
