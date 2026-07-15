use crate::AppState;
use uuid::Uuid;
use crate::dto::CreateUserDto;
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::tenant::CreateTenantDto;
use agrocore_domain::entities::user::UserRole;
use agrocore_shared::SharedError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, ToSchema)]
pub struct SystemStatusResponse {
    pub initialized: bool,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct InitialSetupRequest {
    pub admin: CreateUserDto,
    pub tenant: CreateTenantDto,
}

#[utoipa::path(
    get,
    path = "/api/v1/system/status",
    responses(
        (status = 200, description = "System status", body = SystemStatusResponse)
    ),
    tag = "system"
)]
pub async fn get_status(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let count = state.db.user_repo().count_all().await?;
    Ok(HttpResponse::Ok().json(SystemStatusResponse {
        initialized: count > 0,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/system/setup",
    request_body = InitialSetupRequest,
    responses(
        (status = 201, description = "Initial setup completed"),
        (status = 400, description = "Invalid request or system already initialized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn initial_setup(
    state: web::Data<AppState>,
    dto: web::Json<InitialSetupRequest>,
) -> Result<HttpResponse, ApiError> {
    // 1. Check if already initialized
    let count = state.db.user_repo().count_all().await?;
    if count > 0 {
        return Err(SharedError::Validation("System is already initialized".into()).into());
    }

    dto.admin
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    dto.tenant
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;

    // 2. Create Tenant
    let tenant = state.db.tenant_repo().create(dto.tenant.clone()).await?;

    // 3. Create Admin User
    let mut admin_dto = dto.admin.clone();
    admin_dto.roles = Some(vec![UserRole::Admin]);

    state
        .db
        .user_repo()
        .create(tenant.id, admin_dto.into(), Uuid::nil())
        .await?;
    Ok(HttpResponse::Created().finish())
}

#[utoipa::path(
    delete,
    path = "/api/v1/system/tenant",
    responses(
        (status = 200, description = "Tenant deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn delete_tenant(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<HttpResponse, ApiError> {
    if state.db.tenant_repo().delete(auth.0.tenant_id).await? {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Tenant not found".into()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn initial_setup_request_deserializes_with_setup_modules() {
        let payload = json!({
            "admin": {
                "firstname": "Anna",
                "lastname": "Meyer",
                "email": "anna@example.com",
                "password": "secure-pass-123",
                "roles": null,
                "internal_cost_per_hour": null,
                "external_cost_per_hour": null,
                "language": null
            },
            "tenant": {
                "name": "AgroCore",
                "slug": "agrocore",
                "config": {
                    "default_language": "de",
                    "supported_languages": ["de", "en", "es", "fr", "pt"],
                    "timezone": "Europe/Lisbon",
                    "enabled_modules": [
                        "field_management",
                        "PlantProtection",
                        "Fertilization",
                        "Harvest",
                        "WorkLog",
                        "CostTracking",
                        "Maps",
                        "Reports"
                    ],
                    "custom_field_schemas": {
                        "company_profile": {
                            "name": "AgroCore",
                            "address": "Rua Nova 1",
                            "country": "Portugal",
                            "email": "office@example.com",
                            "phone": "+351912345678"
                        }
                    },
                    "logo_url": null,
                    "primary_color": null,
                    "validation_rules": null
                }
            }
        });

        let request: InitialSetupRequest = serde_json::from_value(payload).expect("setup request");
        assert_eq!(request.tenant.name, "AgroCore");
        assert_eq!(
            request
                .tenant
                .config
                .as_ref()
                .unwrap()
                .enabled_modules
                .len(),
            8
        );
    }
}
