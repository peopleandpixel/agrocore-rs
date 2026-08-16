//! Inventory DTOs

use agrocore_domain::entities::inventory::{
    InventoryCategory, InventoryItem, InventoryLocation, InventoryMethod, InventoryTransaction,
    UnitOfMeasure,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedInventoryItemResponse {
    pub data: Vec<InventoryItemDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryItemDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<InventoryItem> for InventoryItemDto {
    fn from(item: InventoryItem) -> Self {
        InventoryItemDto {
            id: item.id,
            tenant_id: item.tenant_id.into(),
            category: serde_json::to_string(&item.category).unwrap_or_default(),
            name: item.name,
            sku: item.sku,
            description: item.description,
            unit: serde_json::to_string(&item.unit).unwrap_or_default(),
            minimum_stock: item.minimum_stock,
            inventory_method: serde_json::to_string(&item.inventory_method).unwrap_or_default(),
            is_active: item.is_active,
            created_at: item.created_at.to_string(),
            updated_at: item.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInventoryItemRequest {
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateInventoryItemRequest {
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedInventoryTransactionResponse {
    pub data: Vec<InventoryTransactionDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryTransactionDto {
    pub id: Uuid,
    pub item_id: Uuid,
    pub transaction_type: String,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: String,
}

impl From<InventoryTransaction> for InventoryTransactionDto {
    fn from(tx: InventoryTransaction) -> Self {
        InventoryTransactionDto {
            id: tx.id,
            item_id: tx.item_id,
            transaction_type: serde_json::to_string(&tx.transaction_type).unwrap_or_default(),
            quantity: tx.quantity,
            unit_cost: tx.unit_cost,
            total_cost: tx.total_cost,
            batch_number: tx.batch_number,
            expiration_date: tx.expiration_date,
            location: tx.location,
            notes: tx.notes,
            created_by: tx.created_by,
            created_at: tx.created_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockInRequest {
    pub item_id: Uuid,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockOutRequest {
    pub item_id: Uuid,
    pub quantity: f64,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransferRequest {
    pub item_id: Uuid,
    pub quantity: f64,
    pub from_location: String,
    pub to_location: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdjustRequest {
    pub item_id: Uuid,
    pub quantity: f64,
    pub notes: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryBalanceDto {
    pub item_id: Uuid,
    pub item_name: String,
    pub category: String,
    pub unit: String,
    pub total_quantity: f64,
    pub available_quantity: f64,
    pub reserved_quantity: f64,
    pub average_unit_cost: Option<f64>,
    pub total_value: Option<f64>,
    pub minimum_stock: f64,
    pub is_below_minimum: bool,
    pub inventory_method: String,
}

impl From<agrocore_domain::entities::inventory::InventoryBalance> for InventoryBalanceDto {
    fn from(b: agrocore_domain::entities::inventory::InventoryBalance) -> Self {
        InventoryBalanceDto {
            item_id: b.item_id,
            item_name: b.item_name,
            category: serde_json::to_string(&b.category).unwrap_or_default(),
            unit: serde_json::to_string(&b.unit).unwrap_or_default(),
            total_quantity: b.total_quantity,
            available_quantity: b.available_quantity,
            reserved_quantity: b.reserved_quantity,
            average_unit_cost: b.average_unit_cost,
            total_value: b.total_value,
            minimum_stock: b.minimum_stock,
            is_below_minimum: b.is_below_minimum,
            inventory_method: serde_json::to_string(&b.inventory_method).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedInventoryLocationResponse {
    pub data: Vec<InventoryLocationDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryLocationDto {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

impl From<InventoryLocation> for InventoryLocationDto {
    fn from(loc: InventoryLocation) -> Self {
        InventoryLocationDto {
            id: loc.id,
            name: loc.name,
            code: loc.code,
            description: loc.description,
            created_at: loc.created_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInventoryLocationRequest {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateInventoryLocationRequest {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}

// Helper to parse category/unit/method from JSON strings
fn parse_category(s: &str) -> Result<InventoryCategory, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid category: {}", e))
}

fn parse_unit(s: &str) -> Result<UnitOfMeasure, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid unit: {}", e))
}

fn parse_method(s: &str) -> Result<InventoryMethod, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid inventory method: {}", e))
}

pub fn parse_category_owned(s: &str) -> Result<InventoryCategory, String> {
    parse_category(s)
}

pub fn parse_unit_owned(s: &str) -> Result<UnitOfMeasure, String> {
    parse_unit(s)
}

pub fn parse_method_owned(s: &str) -> Result<InventoryMethod, String> {
    parse_method(s)
}
