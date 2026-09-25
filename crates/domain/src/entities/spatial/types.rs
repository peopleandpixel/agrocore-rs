//! Shared spatial types used by both site and spatial modules

use chrono::{DateTime, Utc};
use geo::prelude::{Contains, Intersects};
use geo::{Coord, Distance, Haversine, LineString, MultiPolygon, Point, Polygon};
use geozero::wkb;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Database, Encode, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq)]
pub struct GeoPoint {
    #[validate(range(min = -180.0, max = 180.0))]
    pub lng: f64,
    #[validate(range(min = -90.0, max = 90.0))]
    pub lat: f64,
}

impl GeoPoint {
    pub fn new(lng: f64, lat: f64) -> Self {
        Self { lng, lat }
    }
}

impl From<(f64, f64)> for GeoPoint {
    fn from((lng, lat): (f64, f64)) -> Self {
        Self { lng, lat }
    }
}

// sqlx Postgres support for GeoPoint (stored as JSONB)
impl Type<sqlx::Postgres> for GeoPoint {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for GeoPoint {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for GeoPoint {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Boundary {
    pub polygon: Vec<GeoPoint>,
    pub holes: Option<Vec<Vec<GeoPoint>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Plot {
    pub id: Option<Uuid>,
    pub label: String,
    pub area: f64,
    pub boundary: Boundary,
}

// sqlx Postgres support for Boundary (stored as JSONB)
impl Type<sqlx::Postgres> for Boundary {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for Boundary {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Boundary {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}
