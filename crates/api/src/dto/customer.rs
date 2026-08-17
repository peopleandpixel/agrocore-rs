//! DTOs for Customer entity (Kundenverwaltung)

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomerDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company: Option<String>,
    pub customer_number: Option<String>,
    pub vat_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub preferred_delivery_location: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCustomerDto {
    #[validate(length(min = 2))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company: Option<String>,
    pub customer_number: Option<String>,
    pub vat_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub preferred_delivery_location: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCustomerDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company: Option<String>,
    pub customer_number: Option<String>,
    pub vat_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub preferred_delivery_location: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}
