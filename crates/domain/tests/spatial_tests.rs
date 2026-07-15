use agrocore_domain::entities::site::GeoPoint;
use agrocore_domain::entities::spatial::{SpatialObject, SpatialObjectType};
use chrono::Utc;
use uuid::Uuid;

fn point(lng: f64, lat: f64) -> GeoPoint {
    GeoPoint { lng, lat }
}

fn square(min_lng: f64, min_lat: f64, max_lng: f64, max_lat: f64) -> Vec<GeoPoint> {
    vec![
        point(min_lng, min_lat),
        point(max_lng, min_lat),
        point(max_lng, max_lat),
        point(min_lng, max_lat),
    ]
}

#[test]
fn spatial_objects_can_exist_without_site() {
    let farm = SpatialObject {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        site_id: None,
        parent_id: None,
        label: String::from("Main Farm"),
        object_type: SpatialObjectType::Farm,
        geometry: agrocore_domain::entities::spatial::SpatialGeometry::Polygon {
            exterior: square(14.0, 47.0, 14.1, 47.1),
            holes: vec![],
        },
        area: Some(12.0),
        buffer_meters: None,
        properties: None,
        custom_fields: None,
        note: None,
        is_active: true,
        is_temporary: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: None,
        updated_by: None,
    };

    assert!(farm.site_id.is_none());
    assert!(farm.contains_point(&point(14.05, 47.05)));
}

#[test]
fn overlapping_objects_all_match_the_same_position() {
    let field = SpatialObject {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        site_id: None,
        parent_id: None,
        label: String::from("Field A"),
        object_type: SpatialObjectType::Field,
        geometry: agrocore_domain::entities::spatial::SpatialGeometry::Polygon {
            exterior: square(14.0, 47.0, 14.1, 47.1),
            holes: vec![],
        },
        area: Some(10.0),
        buffer_meters: None,
        properties: None,
        custom_fields: None,
        note: None,
        is_active: true,
        is_temporary: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: None,
        updated_by: None,
    };
    let barn = SpatialObject {
        id: Uuid::new_v4(),
        tenant_id: field.tenant_id,
        site_id: None,
        parent_id: Some(field.id),
        label: String::from("Barn"),
        object_type: SpatialObjectType::Barn,
        geometry: agrocore_domain::entities::spatial::SpatialGeometry::Polygon {
            exterior: square(14.04, 47.04, 14.06, 47.06),
            holes: vec![],
        },
        area: Some(0.4),
        buffer_meters: None,
        properties: None,
        custom_fields: None,
        note: None,
        is_active: true,
        is_temporary: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: None,
        updated_by: None,
    };

    let matches = agrocore_domain::entities::spatial::objects_containing_point(
        [&field, &barn],
        &point(14.05, 47.05),
    );
    assert_eq!(matches.len(), 2);
    assert!(matches.iter().any(|object| object.id == field.id));
    assert!(matches.iter().any(|object| object.id == barn.id));
}

#[test]
fn point_objects_use_a_small_radius() {
    let tree = SpatialObject {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        site_id: None,
        parent_id: None,
        label: String::from("Cork Oak"),
        object_type: SpatialObjectType::CorkOak,
        geometry: agrocore_domain::entities::spatial::SpatialGeometry::Point {
            point: point(14.0, 47.0),
        },
        area: None,
        buffer_meters: Some(15.0),
        properties: None,
        custom_fields: None,
        note: None,
        is_active: true,
        is_temporary: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: None,
        updated_by: None,
    };

    assert!(tree.contains_point(&point(14.0001, 47.0001)));
    assert!(!tree.contains_point(&point(14.01, 47.01)));
}
