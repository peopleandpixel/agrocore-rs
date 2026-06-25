use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::entities::{OrderStatus, OrderType};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Vec<Uuid>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub results: Option<String>,
    pub weather: Option<WeatherInfo>,
    pub custom_fields: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OrderArticle {
    pub article_id: Uuid,
    pub label: String,
    pub quantity_per_hectare: f64,
    pub unit: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub order_type: OrderType,
    pub site_ids: Vec<Uuid>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateOrderDto {
    pub label: Option<String>,
    pub status: Option<OrderStatus>,
    pub site_ids: Option<Vec<Uuid>>,
    pub assigned_worker_ids: Option<Vec<Uuid>>,
    pub planned_date: Option<DateTime<Utc>>,
    pub deadline_date: Option<DateTime<Utc>>,
    pub articles: Option<Vec<OrderArticle>>,
    pub quantities: Option<serde_json::Value>,
    pub results: Option<String>,
    pub weather: Option<WeatherInfo>,
    pub custom_fields: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTask {
    pub order_id: Uuid,
    pub label: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub site_count: u32,
    pub total_area: f64,
    pub deadline_date: Option<DateTime<Utc>>,
    pub planned_date: Option<DateTime<Utc>>,
}
