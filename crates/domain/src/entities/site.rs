//! Site entities

use crate::entities::spatial::types::{Boundary, GeoPoint, Plot};
use crate::entities::{CropType, SiteType};
pub use agrocore_shared::lpis::{LpisCountry, LpisParcel as LpisData};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Encode, Type};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct Site {
    pub id: Uuid,
    pub tenant_id: Uuid,
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
    pub gross_area: Option<f64>,
    #[sqlx(json)]
    pub plots: serde_json::Value,
    #[sqlx(json)]
    pub row_config: Option<serde_json::Value>,
    #[sqlx(json)]
    pub bbch_stage: Option<serde_json::Value>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    #[sqlx(json)]
    pub center: Option<GeoPoint>,
    #[sqlx(json)]
    pub sigpac_data: Option<SigpacData>,
    #[sqlx(json)]
    pub lpis_country: Option<agrocore_shared::lpis::LpisCountry>,
    #[sqlx(json)]
    pub lpis_data: Option<LpisData>,
    pub regepac_id: Option<String>,
    #[sqlx(json)]
    pub boundary: Option<Boundary>,
    #[sqlx(json)]
    pub properties: Option<serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<String>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Boundary>,
    pub properties: Option<Vec<SiteProperty>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSiteDto {
    pub label: Option<String>,
    pub site_type: Option<SiteType>,
    pub crop_type: Option<CropType>,
    pub variety: Option<String>,
    pub area: Option<f64>,
    pub gross_area: Option<f64>,
    pub plots: Option<serde_json::Value>,
    pub row_config: Option<serde_json::Value>,
    pub bbch_stage: Option<String>,
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
    pub lpis_country: Option<agrocore_shared::lpis::LpisCountry>,
    pub lpis_data: Option<LpisData>,
    pub properties: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
    pub is_temporary: Option<bool>,
    pub boundary: Option<Boundary>,
}

// sqlx Postgres support for RowConfig (stored as JSONB)
impl Type<sqlx::Postgres> for RowConfig {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for RowConfig {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for RowConfig {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}

// sqlx Postgres support for SigpacData (stored as JSONB)
impl Type<sqlx::Postgres> for SigpacData {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for SigpacData {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SigpacData {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}

// sqlx Postgres support for SiteType (stored as JSONB)
impl Type<sqlx::Postgres> for SiteType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for SiteType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SiteType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}

// sqlx Postgres support for CropType (stored as JSONB)
impl Type<sqlx::Postgres> for CropType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }
}

impl<'q> Encode<'q, sqlx::Postgres> for CropType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        let json_str: &str = json.as_str();
        <&str as Encode<'q, sqlx::Postgres>>::encode_by_ref(&json_str, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for CropType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let json: &str = <&str as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(serde_json::from_str(json)?)
    }
}
