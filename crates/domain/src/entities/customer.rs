use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;
use crate::repositories::VisibilityAwareEntity;

/// Kunden-CRM: Kontakte, Bestellhistorie, Vorlieben
///
/// Stellt Kunden dar, die Bestellungen (Arbeitsaufträge vom Typ `SalesOrder`)
/// aufgeben können. Unterstützt sowohl Einzelkunden als auch Firmen.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, sqlx::FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 30))]
    pub phone: Option<String>,
    pub address: Option<String>,
    /// Firma / Unternehmen (für B2B-Kunden)
    pub company: Option<String>,
    /// Kundennummer / Referenz-ID
    pub customer_number: Option<String>,
    /// Standard-MwSt.-Satz in Prozent
    pub vat_rate: Option<f64>,
    /// Standard-Zahlungsbedingungen (z.B. "net_30", "cod", "prepaid")
    pub payment_terms: Option<String>,
    /// Lieblingsort für Lieferungen / Abholung
    pub preferred_delivery_location: Option<serde_json::Value>,
    /// Vorlieben (z.B. Lieferzeit-Fenster, Produkttypen)
    #[sqlx(json)]
    pub preferences: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl VisibilityAwareEntity for Customer {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCustomerDto {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 30))]
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company: Option<String>,
    pub customer_number: Option<String>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub vat_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub preferred_delivery_location: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct UpdateCustomerDto {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 30))]
    pub phone: Option<String>,
    pub address: Option<String>,
    pub company: Option<String>,
    pub customer_number: Option<String>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub vat_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub preferred_delivery_location: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn customer_can_be_created_from_dto() {
        let dto = CreateCustomerDto {
            name: "Klaus Mueller".to_string(),
            email: Some("klaus@example.com".to_string()),
            phone: Some("+49 151 12345678".to_string()),
            address: Some("Bauernstr. 1, 12345 Dorf".to_string()),
            company: Some("Hof Müller GmbH".to_string()),
            customer_number: Some("K-1001".to_string()),
            vat_rate: Some(19.0),
            payment_terms: Some("net_14".to_string()),
            preferred_delivery_location: None,
            preferences: None,
        };

        let customer = Customer {
            id: Uuid::new_v4(),
            tenant_id: TenantId(Uuid::new_v4()),
            name: dto.name,
            email: dto.email,
            phone: dto.phone,
            address: dto.address,
            company: dto.company,
            customer_number: dto.customer_number,
            vat_rate: dto.vat_rate,
            payment_terms: dto.payment_terms,
            preferred_delivery_location: dto.preferred_delivery_location,
            preferences: dto.preferences,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(customer.name, "Klaus Mueller");
        assert!(customer.is_active);
    }
}
