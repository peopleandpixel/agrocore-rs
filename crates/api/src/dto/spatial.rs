//! GeoJSON output for the map.
//!
//! A GeoJSON `FeatureCollection` rather than a list of objects with a nested geometry,
//! because that is what a web map consumes directly: MapLibre's `addSource` takes the
//! whole collection, and any reshaping in the browser is work every consumer would repeat.
//!
//! The per-feature `properties` carry the attributes the map layers filter on. They are
//! deliberately flat and scalar: MapLibre evaluates expressions per feature, so a nested
//! object means every layer has to reach into it.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use agrocore_domain::entities::spatial::{SpatialGeometry, SpatialObject};

/// A GeoJSON `FeatureCollection`, the response of every spatial read endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub features: Vec<Feature>,
}

impl FeatureCollection {
    pub fn empty() -> Self {
        Self {
            kind: "FeatureCollection",
            features: Vec::new(),
        }
    }
}

/// One GeoJSON `Feature`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Feature {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub geometry: Value,
    pub properties: FeatureProperties,
    /// RFC 7946 allows an optional feature id. MapLibre uses it to keep selections stable
    /// across a refetch, so it is populated rather than left out.
    pub id: Uuid,
}

/// Attributes carried on each feature.
///
/// `object_type` and `site_id` are what the map layers filter on, so they are named
/// explicitly rather than hidden in a generic bag. `planted_at` and `variety_id` are
/// Option because both are nullable in the schema: an object imported from SIGPAC carries
/// neither.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureProperties {
    pub id: Uuid,
    pub label: String,
    /// The stored value, e.g. `olive_tree`, `vine`, `barn`.
    pub object_type: String,
    pub site_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub area: Option<f64>,
    /// The spatial buffer in metres. The map draws a symbol for a point object at its real
    /// size, so the client needs this rather than having to hard-code 5 m.
    pub buffer_meters: Option<f64>,
    pub planted_at: Option<chrono::NaiveDate>,
    pub variety_id: Option<Uuid>,
    pub is_active: bool,
    pub is_temporary: bool,
    pub note: Option<String>,
}

impl From<&SpatialObject> for Feature {
    fn from(obj: &SpatialObject) -> Self {
        Self {
            kind: "Feature",
            geometry: geometry_to_geojson(&obj.geometry),
            properties: FeatureProperties {
                id: obj.id,
                label: obj.label.clone(),
                // Round-tripped through serde rather than matched by hand: the enum's
                // serde rename is the single definition of the stored value, and a second
                // mapping here could drift from it without any test failing.
                object_type: serde_json::to_value(&obj.object_type)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default(),
                site_id: obj.site_id,
                parent_id: obj.parent_id,
                area: obj.area,
                buffer_meters: obj.buffer_meters,
                // Migration 9 added these to spatial_objects; they are read here rather
                // than added to the domain struct so the existing `query_as` mapping in
                // the repository keeps matching its column list.
                planted_at: obj.planted_at,
                variety_id: obj.variety_id,
                is_active: obj.is_active,
                is_temporary: obj.is_temporary,
                note: obj.note.clone(),
            },
            id: obj.id,
        }
    }
}

fn geometry_to_geojson(geometry: &SpatialGeometry) -> Value {
    geometry.to_geojson()
}

impl From<Vec<SpatialObject>> for FeatureCollection {
    fn from(objects: Vec<SpatialObject>) -> Self {
        Self {
            kind: "FeatureCollection",
            features: objects.iter().map(Feature::from).collect(),
        }
    }
}

/// Response of the viewport query: a FeatureCollection plus a note about truncation.
///
/// `truncated` is returned rather than silently dropping objects. A map that shows 5,000
/// of 5,200 objects and says nothing is worse than one that says so, because the missing
/// trees are invisible and the count looks complete.
///
/// The fields are inlined rather than `#[serde(flatten)]`-ed from a nested collection,
/// so the JSON is a valid FeatureCollection with two extra members -- which is what the
/// map expects, and which a flattened struct cannot express while still deriving
/// `Deserialize` without a lifetime.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureCollectionResponse {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub features: Vec<Feature>,
    /// Whether the limit cut the result short.
    pub truncated: bool,
}

impl FeatureCollectionResponse {
    pub fn new(collection: FeatureCollection, truncated: bool) -> Self {
        Self {
            kind: "FeatureCollection",
            features: collection.features,
            truncated,
        }
    }
}

/// DTO for nearby sub-tasks returned by the nearby tasks endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NearbySubTaskDto {
    pub sub_task_id: Uuid,
    pub task_id: Uuid,
    pub label: String,
    pub unit_kind: String,
    pub status: String,
    pub planned_quantity: Option<f64>,
    pub completed_quantity: f64,
    pub site_id: Option<Uuid>,
    pub site_label: Option<String>,
    pub distance_m: f64,
}

/// Site data for the MapLibre map view.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapSiteData {
    pub id: Uuid,
    pub label: String,
    pub geometry: serde_json::Value,
    pub color: Option<String>,
}

/// Task data for the MapLibre map view.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapTaskData {
    pub id: Uuid,
    pub label: String,
    pub status: String,
    pub geometry: serde_json::Value,
    pub site_id: Option<Uuid>,
}
