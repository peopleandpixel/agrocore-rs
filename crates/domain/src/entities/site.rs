#![allow(deprecated)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::{BbchStage, CropType, SiteType};
use crate::repositories::VisibilityAwareEntity;

use agrocore_shared::lpis::LpisCountry;
use geo::{LineString, Polygon};
use geozero::wkb;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct LpisData {
    /// LPIS country code (ES=SIGPAC, PT=iLPIS, FR=RPG, IT=SIAN, etc.)
    pub country: LpisCountry,
    /// Full LPIS reference (e.g., ES411234567890123 for SIGPAC)
    pub reference: String,
    /// Province/State/Department code
    pub province: Option<String>,
    /// Municipality/Commune code
    pub municipality: Option<String>,
    /// Aggregate code (if applicable)
    pub aggregate: Option<String>,
    /// Zone code (if applicable)
    pub zone: Option<String>,
    /// Polygon code (if applicable)
    pub polygon: Option<String>,
    /// Parcel code
    pub parcel: Option<String>,
    /// Enclosure code (if applicable)
    pub enclosure: Option<String>,
    /// Usage code (e.g., crop type code)
    pub usage_code: Option<String>,
    /// Usage description
    pub usage_description: Option<String>,
    /// Official area in hectares from LPIS
    pub official_area_ha: Option<f64>,
    /// Source dataset identifier
    pub source_dataset: Option<String>,
    /// Source year
    pub source_year: Option<i16>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq)]
pub struct GeoPoint {
    #[validate(range(min = -180.0, max = 180.0))]
    pub lng: f64,
    #[validate(range(min = -90.0, max = 90.0))]
    pub lat: f64,
}

impl sqlx::Type<sqlx::Postgres> for GeoPoint {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("geometry")
    }
}

impl sqlx::postgres::PgHasArrayType for GeoPoint {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_geometry")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for GeoPoint {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let wkb_wrapper: geozero::wkb::Decode<geo::Geometry<f64>> = sqlx::Decode::decode(value)?;
        let geometry = wkb_wrapper.geometry.ok_or("Failed to decode geometry")?;
        if let geo::Geometry::Point(p) = geometry {
            Ok(GeoPoint {
                lng: p.x(),
                lat: p.y(),
            })
        } else {
            Err("Expected Point geometry".into())
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for GeoPoint {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let geometry: geo::Geometry<f64> =
            geo::Geometry::Point(geo::Point::new(self.lng, self.lat));
        let wkb_wrapper = geozero::wkb::Encode(geometry);
        <geozero::wkb::Encode<geo::Geometry<f64>> as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&wkb_wrapper, buf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(transparent)]
pub struct Boundary(pub Vec<GeoPoint>);

impl std::ops::Deref for Boundary {
    type Target = Vec<GeoPoint>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl sqlx::Type<sqlx::Postgres> for Boundary {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("geometry")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Boundary {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let wkb_wrapper: wkb::Decode<geo::Geometry<f64>> = sqlx::Decode::decode(value)?;
        let geometry = wkb_wrapper.geometry.ok_or("Failed to decode geometry")?;
        if let geo::Geometry::Polygon(p) = geometry {
            let points = p
                .exterior()
                .0
                .iter()
                .map(|c| GeoPoint { lng: c.x, lat: c.y })
                .collect();
            Ok(Boundary(points))
        } else {
            Err("Expected Polygon geometry".into())
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Boundary {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let coords: Vec<(f64, f64)> = self.0.iter().map(|p| (p.lng, p.lat)).collect();
        let mut line_string = LineString::from(coords);

        // Ensure ring is closed for PostGIS Polygon
        let needs_close = line_string
            .0
            .first()
            .zip(line_string.0.last())
            .map(|(f, l)| f != l)
            .unwrap_or(false);

        #[allow(clippy::collapsible_if)]
        if needs_close {
            if let Some(first) = line_string.0.first().copied() {
                line_string.0.push(first);
            }
        }

        let polygon = Polygon::new(line_string, vec![]);
        let geometry = geo::Geometry::Polygon(polygon);
        let wkb_wrapper = wkb::Encode(geometry);
        <wkb::Encode<geo::Geometry<f64>> as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(
            &wkb_wrapper,
            buf,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct Plot {
    pub id: Uuid,
    pub label: String,
    #[validate(range(min = 0.0))]
    pub area: f64,
    pub boundary: Option<Boundary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, sqlx::FromRow)]
pub struct Site {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub business_id: Option<Uuid>,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    #[sqlx(json)]
    pub site_type: SiteType,
    #[sqlx(json)]
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    #[validate(range(min = 0.0))]
    pub gross_area: Option<f64>,
    #[sqlx(json)]
    pub plots: Vec<Plot>,
    #[sqlx(json)]
    pub row_config: Option<RowConfig>,
    #[sqlx(json)]
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
    #[sqlx(json)]
    pub sigpac_data: Option<SigpacData>,
    /// LPIS country for this site (ES=SIGPAC, PT=iLPIS, FR=RPG, IT=SIAN, etc.)
    #[sqlx(json)]
    pub lpis_country: Option<agrocore_shared::lpis::LpisCountry>,
    /// Generic LPIS data - can hold SIGPAC, iLPIS, RPG, etc. references
    #[sqlx(json)]
    pub lpis_data: Option<LpisData>,
    /// Deprecated: use lpis_data.reference instead
    #[deprecated(since = "0.5.8", note = "Use lpis_data.reference instead")]
    #[allow(deprecated)]
    pub regepac_id: Option<String>,
    pub boundary: Option<Boundary>,
    #[sqlx(json)]
    pub properties: Option<Vec<SiteProperty>>,
    #[sqlx(json)]
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

impl VisibilityAwareEntity for Site {}

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
    pub boundary: Option<Boundary>,
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
    pub boundary: Option<Boundary>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
}
