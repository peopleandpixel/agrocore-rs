use chrono::{DateTime, Utc};
use geo::prelude::{Contains, Intersects};
use geo::{Coord, Distance, Haversine, LineString, MultiPolygon, Point, Polygon};
use geozero::wkb;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeInfo;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::site::GeoPoint;
use crate::entities::tenant::TenantId;
use crate::repositories::VisibilityAwareEntity;

// use crate::entities::user::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum SpatialObjectType {
    #[serde(rename = "farm")]
    Farm,
    #[serde(rename = "site")]
    Site,
    #[serde(rename = "field")]
    Field,
    #[serde(rename = "pasture")]
    Pasture,
    #[serde(rename = "barn")]
    Barn,
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "shed")]
    Shed,
    #[serde(rename = "pen")]
    Pen,
    #[serde(rename = "tree")]
    Tree,
    #[serde(rename = "cork_oak")]
    CorkOak,
    #[serde(rename = "olive_tree")]
    OliveTree,
    #[serde(rename = "water_point")]
    WaterPoint,
    #[serde(rename = "fence")]
    Fence,
    #[serde(rename = "other")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PolygonGeometry {
    pub exterior: Vec<GeoPoint>,
    pub holes: Vec<Vec<GeoPoint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "geometry_type", rename_all = "snake_case")]
pub enum SpatialGeometry {
    Point {
        point: GeoPoint,
    },
    LineString {
        points: Vec<GeoPoint>,
    },
    Polygon {
        exterior: Vec<GeoPoint>,
        holes: Vec<Vec<GeoPoint>>,
    },
    MultiPolygon {
        polygons: Vec<PolygonGeometry>,
    },
}

impl sqlx::Type<sqlx::Postgres> for SpatialGeometry {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl sqlx::postgres::PgHasArrayType for SpatialGeometry {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_geometry")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SpatialGeometry {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let wkb_wrapper: wkb::Decode<geo::Geometry<f64>> = sqlx::Decode::decode(value)?;
        let geometry = wkb_wrapper.geometry.ok_or("Failed to decode geometry")?;

        match geometry {
            geo::Geometry::Point(p) => Ok(SpatialGeometry::Point {
                point: GeoPoint {
                    lng: p.x(),
                    lat: p.y(),
                },
            }),
            geo::Geometry::LineString(ls) => Ok(SpatialGeometry::LineString {
                points: ls
                    .0
                    .into_iter()
                    .map(|c| GeoPoint { lng: c.x, lat: c.y })
                    .collect(),
            }),
            geo::Geometry::Polygon(poly) => {
                let exterior = poly
                    .exterior()
                    .0
                    .iter()
                    .map(|c| GeoPoint { lng: c.x, lat: c.y })
                    .collect();
                let holes = poly
                    .interiors()
                    .iter()
                    .map(|ls| {
                        ls.0.iter()
                            .map(|c| GeoPoint { lng: c.x, lat: c.y })
                            .collect()
                    })
                    .collect();
                Ok(SpatialGeometry::Polygon { exterior, holes })
            }
            geo::Geometry::MultiPolygon(mp) => {
                let polygons =
                    mp.0.into_iter()
                        .map(|poly| {
                            let exterior = poly
                                .exterior()
                                .0
                                .iter()
                                .map(|c| GeoPoint { lng: c.x, lat: c.y })
                                .collect();
                            let holes = poly
                                .interiors()
                                .iter()
                                .map(|ls| {
                                    ls.0.iter()
                                        .map(|c| GeoPoint { lng: c.x, lat: c.y })
                                        .collect()
                                })
                                .collect();
                            PolygonGeometry { exterior, holes }
                        })
                        .collect();
                Ok(SpatialGeometry::MultiPolygon { polygons })
            }
            _ => Err("Unsupported geometry type".into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SpatialGeometry {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let geometry: geo::Geometry<f64> = match self {
            SpatialGeometry::Point { point } => {
                geo::Geometry::Point(Point::new(point.lng, point.lat))
            }
            SpatialGeometry::LineString { points } => geo::Geometry::LineString(LineString::from(
                points.iter().map(|p| (p.lng, p.lat)).collect::<Vec<_>>(),
            )),
            SpatialGeometry::Polygon { exterior, holes } => {
                let ext_ls =
                    LineString::from(exterior.iter().map(|p| (p.lng, p.lat)).collect::<Vec<_>>());
                let int_ls = holes
                    .iter()
                    .map(|h| LineString::from(h.iter().map(|p| (p.lng, p.lat)).collect::<Vec<_>>()))
                    .collect();
                geo::Geometry::Polygon(Polygon::new(ext_ls, int_ls))
            }
            SpatialGeometry::MultiPolygon { polygons } => {
                let polys = polygons
                    .iter()
                    .map(|p| {
                        let ext_ls = LineString::from(
                            p.exterior
                                .iter()
                                .map(|p| (p.lng, p.lat))
                                .collect::<Vec<_>>(),
                        );
                        let int_ls = p
                            .holes
                            .iter()
                            .map(|h| {
                                LineString::from(
                                    h.iter().map(|p| (p.lng, p.lat)).collect::<Vec<_>>(),
                                )
                            })
                            .collect();
                        Polygon::new(ext_ls, int_ls)
                    })
                    .collect();
                geo::Geometry::MultiPolygon(MultiPolygon::new(polys))
            }
        };

        let wkb_wrapper = wkb::Encode(geometry);
        <wkb::Encode<geo::Geometry<f64>> as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(
            &wkb_wrapper,
            buf,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct SpatialProperty {
    pub key: String,
    pub value: serde_json::Value,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct SpatialObject {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub site_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[sqlx(json)]
    pub object_type: SpatialObjectType,
    pub geometry: SpatialGeometry,
    #[validate(range(min = 0.0))]
    pub area: Option<f64>,
    #[validate(range(min = 0.0))]
    pub buffer_meters: Option<f64>,
    #[sqlx(json)]
    pub properties: Option<Vec<SpatialProperty>>,
    #[sqlx(json)]
    pub custom_fields: Option<serde_json::Value>,
    pub note: Option<String>,
    pub is_active: bool,
    pub is_temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl VisibilityAwareEntity for SpatialObject {}

impl SpatialObject {
    pub fn contains_point(&self, point: &GeoPoint) -> bool {
        self.geometry.contains_point(point, self.buffer_meters)
    }
}

impl SpatialGeometry {
    pub fn contains_point(&self, point: &GeoPoint, buffer_meters: Option<f64>) -> bool {
        match self {
            SpatialGeometry::Point { point: target } => {
                let radius_meters = buffer_meters.unwrap_or(5.0);
                let source = Point::new(target.lng, target.lat);
                let query = Point::new(point.lng, point.lat);
                Haversine.distance(source, query) <= radius_meters
            }
            SpatialGeometry::LineString { points } => {
                let radius_meters = buffer_meters.unwrap_or(5.0);
                line_string_distance_meters(points, point) <= radius_meters
            }
            SpatialGeometry::Polygon { exterior, holes } => {
                polygon_contains_point(exterior, holes, point)
            }
            SpatialGeometry::MultiPolygon { polygons } => polygons
                .iter()
                .any(|polygon| polygon_contains_point(&polygon.exterior, &polygon.holes, point)),
        }
    }
}

fn line_string_distance_meters(points: &[GeoPoint], point: &GeoPoint) -> f64 {
    if points.len() < 2 {
        return f64::INFINITY;
    }

    let mut min_distance = f64::INFINITY;
    for pair in points.windows(2) {
        let distance = distance_point_to_segment_meters(point, &pair[0], &pair[1]);
        if distance < min_distance {
            min_distance = distance;
        }
    }

    min_distance
}

fn distance_point_to_segment_meters(point: &GeoPoint, a: &GeoPoint, b: &GeoPoint) -> f64 {
    let (px, py) = project_to_local_meters(point, point);
    let (ax, ay) = project_to_local_meters(a, point);
    let (bx, by) = project_to_local_meters(b, point);

    let ab_x = bx - ax;
    let ab_y = by - ay;
    let ap_x = px - ax;
    let ap_y = py - ay;
    let ab_len_sq = ab_x * ab_x + ab_y * ab_y;
    if ab_len_sq == 0.0 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }

    let t = ((ap_x * ab_x) + (ap_y * ab_y)) / ab_len_sq;
    let t = t.clamp(0.0, 1.0);
    let closest_x = ax + t * ab_x;
    let closest_y = ay + t * ab_y;
    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
}

fn project_to_local_meters(point: &GeoPoint, origin: &GeoPoint) -> (f64, f64) {
    let lat_scale = 111_320.0;
    let lng_scale = 111_320.0 * origin.lat.to_radians().cos().abs().max(0.000_001);
    (
        (point.lng - origin.lng) * lng_scale,
        (point.lat - origin.lat) * lat_scale,
    )
}

fn polygon_contains_point(
    exterior: &[GeoPoint],
    holes: &[Vec<GeoPoint>],
    point: &GeoPoint,
) -> bool {
    if exterior.len() < 3 {
        return false;
    }

    let polygon = Polygon::new(
        to_closed_linestring(exterior),
        holes
            .iter()
            .map(|hole| to_closed_linestring(hole))
            .collect(),
    );
    let query = Point::new(point.lng, point.lat);
    polygon.contains(&query) || polygon.intersects(&query)
}

fn to_closed_linestring(points: &[GeoPoint]) -> LineString<f64> {
    let mut coords: Vec<Coord<f64>> = points
        .iter()
        .map(|point| Coord {
            x: point.lng,
            y: point.lat,
        })
        .collect();

    #[allow(clippy::collapsible_if)]
    if let (Some(first_coord), Some(last_coord)) = (coords.first().copied(), coords.last().copied())
    {
        if first_coord != last_coord {
            coords.push(first_coord);
        }
    }

    LineString::from(coords)
}

pub fn objects_containing_point<'a, I>(objects: I, point: &GeoPoint) -> Vec<&'a SpatialObject>
where
    I: IntoIterator<Item = &'a SpatialObject>,
{
    objects
        .into_iter()
        .filter(|object| object.is_active && object.contains_point(point))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
            tenant_id: TenantId(Uuid::new_v4()),
            site_id: None,
            parent_id: None,
            label: String::from("Main Farm"),
            object_type: SpatialObjectType::Farm,
            geometry: SpatialGeometry::Polygon {
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
            tenant_id: TenantId(Uuid::new_v4()),
            site_id: None,
            parent_id: None,
            label: String::from("Field A"),
            object_type: SpatialObjectType::Field,
            geometry: SpatialGeometry::Polygon {
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
            geometry: SpatialGeometry::Polygon {
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

        let matches = objects_containing_point([&field, &barn], &point(14.05, 47.05));
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|object| object.id == field.id));
        assert!(matches.iter().any(|object| object.id == barn.id));
    }

    #[test]
    fn point_objects_use_a_small_radius() {
        let tree = SpatialObject {
            id: Uuid::new_v4(),
            tenant_id: TenantId(Uuid::new_v4()),
            site_id: None,
            parent_id: None,
            label: String::from("Cork Oak"),
            object_type: SpatialObjectType::CorkOak,
            geometry: SpatialGeometry::Point {
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
}
