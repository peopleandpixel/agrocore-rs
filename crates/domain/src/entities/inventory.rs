// Copyright 2024 peopleandpixel
//
// Licensed under the GPL-3.0-or-later license;
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.en.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::tenant::TenantId;

// Note: utoipa does not support enums with internal data (e.g. Other(String)) in ToSchema.
// We omit ToSchema from these enums and use String serialization in DTOs for OpenAPI.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InventoryCategory {
    #[serde(rename = "seed")]
    Seed,
    #[serde(rename = "fertilizer")]
    Fertilizer,
    #[serde(rename = "pesticide")]
    Pesticide,
    #[serde(rename = "herbicide")]
    Herbicide,
    #[serde(rename = "fuel")]
    Fuel,
    #[serde(rename = "feed")]
    Feed,
    #[serde(rename = "medicine")]
    Medicine,
    #[serde(rename = "spare_part")]
    SparePart,
    #[serde(rename = "equipment")]
    Equipment,
    #[serde(rename = "other")]
    Other(String),
}

impl Default for InventoryCategory {
    fn default() -> Self {
        InventoryCategory::Other(String::new())
    }
}

impl std::fmt::Display for InventoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryCategory::Seed => write!(f, "seed"),
            InventoryCategory::Fertilizer => write!(f, "fertilizer"),
            InventoryCategory::Pesticide => write!(f, "pesticide"),
            InventoryCategory::Herbicide => write!(f, "herbicide"),
            InventoryCategory::Fuel => write!(f, "fuel"),
            InventoryCategory::Feed => write!(f, "feed"),
            InventoryCategory::Medicine => write!(f, "medicine"),
            InventoryCategory::SparePart => write!(f, "spare_part"),
            InventoryCategory::Equipment => write!(f, "equipment"),
            InventoryCategory::Other(s) => write!(f, "other:{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnitOfMeasure {
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "l")]
    Liter,
    #[serde(rename = "bag")]
    Bag,
    #[serde(rename = "unit")]
    Unit,
    #[serde(rename = "box")]
    Box,
    #[serde(rename = "bale")]
    Bale,
    #[serde(rename = "liter")]
    LiterSpelled,
    #[serde(rename = "custom")]
    Custom(String),
}

impl Default for UnitOfMeasure {
    fn default() -> Self {
        UnitOfMeasure::Custom(String::new())
    }
}

impl std::fmt::Display for UnitOfMeasure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitOfMeasure::Kg => write!(f, "kg"),
            UnitOfMeasure::Liter => write!(f, "l"),
            UnitOfMeasure::Bag => write!(f, "bag"),
            UnitOfMeasure::Unit => write!(f, "unit"),
            UnitOfMeasure::Box => write!(f, "box"),
            UnitOfMeasure::Bale => write!(f, "bale"),
            UnitOfMeasure::LiterSpelled => write!(f, "liter"),
            UnitOfMeasure::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

// Simple enum without internal data - safe for utoipa ToSchema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, utoipa::ToSchema)]
pub enum InventoryMethod {
    #[default]
    #[serde(rename = "FIFO")]
    Fifo,
    #[serde(rename = "FEFO")]
    Fefo,
}

// Simple enum without internal data - safe for utoipa ToSchema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub enum TransactionType {
    #[serde(rename = "stock_in")]
    StockIn,
    #[serde(rename = "stock_out")]
    StockOut,
    #[serde(rename = "transfer")]
    Transfer,
    #[serde(rename = "adjustment")]
    Adjustment,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct InventoryItem {
    pub id: Uuid,
    pub tenant_id: TenantId,
    #[sqlx(json)]
    #[serde(serialize_with = "serialize_category_as_string")]
    #[serde(deserialize_with = "deserialize_category_from_string")]
    #[schema(value_type = String)]
    pub category: InventoryCategory,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    #[sqlx(json)]
    #[serde(serialize_with = "serialize_uom_as_string")]
    #[serde(deserialize_with = "deserialize_uom_from_string")]
    #[schema(value_type = String)]
    pub unit: UnitOfMeasure,
    pub minimum_stock: f64,
    #[sqlx(json)]
    pub inventory_method: InventoryMethod,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn serialize_category_as_string<S: serde::Serializer>(
    cat: &InventoryCategory,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&cat.to_string())
}

fn deserialize_category_from_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<InventoryCategory, D::Error> {
    let s = String::deserialize(deserializer)?;
    if let Some(rest) = s.strip_prefix("other:") {
        Ok(InventoryCategory::Other(rest.to_string()))
    } else {
        match s.as_str() {
            "seed" => Ok(InventoryCategory::Seed),
            "fertilizer" => Ok(InventoryCategory::Fertilizer),
            "pesticide" => Ok(InventoryCategory::Pesticide),
            "herbicide" => Ok(InventoryCategory::Herbicide),
            "fuel" => Ok(InventoryCategory::Fuel),
            "feed" => Ok(InventoryCategory::Feed),
            "medicine" => Ok(InventoryCategory::Medicine),
            "spare_part" => Ok(InventoryCategory::SparePart),
            "equipment" => Ok(InventoryCategory::Equipment),
            _ => Ok(InventoryCategory::Other(s)),
        }
    }
}

fn serialize_uom_as_string<S: serde::Serializer>(
    uom: &UnitOfMeasure,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&uom.to_string())
}

fn deserialize_uom_from_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<UnitOfMeasure, D::Error> {
    let s = String::deserialize(deserializer)?;
    if let Some(rest) = s.strip_prefix("custom:") {
        Ok(UnitOfMeasure::Custom(rest.to_string()))
    } else {
        match s.as_str() {
            "kg" => Ok(UnitOfMeasure::Kg),
            "l" => Ok(UnitOfMeasure::Liter),
            "bag" => Ok(UnitOfMeasure::Bag),
            "unit" => Ok(UnitOfMeasure::Unit),
            "box" => Ok(UnitOfMeasure::Box),
            "bale" => Ok(UnitOfMeasure::Bale),
            "liter" => Ok(UnitOfMeasure::LiterSpelled),
            _ => Ok(UnitOfMeasure::Custom(s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateInventoryItemDto {
    #[serde(serialize_with = "serialize_category_as_string")]
    #[serde(deserialize_with = "deserialize_category_from_string")]
    #[schema(value_type = String)]
    pub category: InventoryCategory,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    #[serde(serialize_with = "serialize_uom_as_string")]
    #[serde(deserialize_with = "deserialize_uom_from_string")]
    #[schema(value_type = String)]
    pub unit: UnitOfMeasure,
    pub minimum_stock: f64,
    pub inventory_method: InventoryMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateInventoryItemDto {
    #[serde(serialize_with = "serialize_category_as_string")]
    #[serde(deserialize_with = "deserialize_category_from_string")]
    #[schema(value_type = String)]
    pub category: InventoryCategory,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    #[serde(serialize_with = "serialize_uom_as_string")]
    #[serde(deserialize_with = "deserialize_uom_from_string")]
    #[schema(value_type = String)]
    pub unit: UnitOfMeasure,
    pub minimum_stock: f64,
    pub inventory_method: InventoryMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub item_id: Uuid,
    #[sqlx(json)]
    pub transaction_type: TransactionType,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub reference_id: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateInventoryTransactionDto {
    pub item_id: Uuid,
    pub transaction_type: TransactionType,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct InventoryBalance {
    pub item_id: Uuid,
    pub item_name: String,
    #[serde(serialize_with = "serialize_category_as_string")]
    #[serde(deserialize_with = "deserialize_category_from_string")]
    #[schema(value_type = String)]
    pub category: InventoryCategory,
    #[serde(serialize_with = "serialize_uom_as_string")]
    #[serde(deserialize_with = "deserialize_uom_from_string")]
    #[schema(value_type = String)]
    pub unit: UnitOfMeasure,
    pub total_quantity: f64,
    pub available_quantity: f64,
    pub reserved_quantity: f64,
    pub average_unit_cost: Option<f64>,
    pub total_value: Option<f64>,
    pub minimum_stock: f64,
    pub is_below_minimum: bool,
    pub inventory_method: InventoryMethod,
    pub latest_transactions: Vec<InventoryTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct InventoryLocation {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateInventoryLocationDto {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateInventoryLocationDto {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}
