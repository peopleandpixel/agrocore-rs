//! Demo API Data Transfer Objects

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Request to seed demo data
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct DemoSeedRequest {
    /// Tenant name for the demo
    #[validate(length(min = 1, max = 100))]
    pub tenant: String,
    /// Admin username for the demo
    #[validate(length(min = 1, max = 100))]
    pub user: String,
    /// Whether to reset existing demo data first
    #[serde(default)]
    pub reset: bool,
}

/// Response after demo seeding
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DemoSeedResponse {
    pub success: bool,
    pub message: String,
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

/// Demo data summary
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DemoDataSummary {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub sites_count: usize,
    pub equipment_count: usize,
    pub orders_count: usize,
    pub workers_count: usize,
    pub inventory_items_count: usize,
    pub animals_count: usize,
}
