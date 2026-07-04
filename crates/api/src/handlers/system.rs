use crate::dto::{CreateUserDto, ErrorResponse};
use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
use agrocore_domain::entities::tenant::CreateTenantDto;
use agrocore_domain::entities::user::UserRole;
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
pub async fn get_status(state: web::Data<AppState>) -> impl Responder {
    match state.db.user_repo().count_all().await {
        Ok(count) => HttpResponse::Ok().json(SystemStatusResponse {
            initialized: count > 0,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "database".into(),
            message: e.to_string(),
        }),
    }
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
) -> impl Responder {
    // 1. Check if already initialized
    match state.db.user_repo().count_all().await {
        Ok(count) if count > 0 => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "already_initialized".into(),
                message: "System is already initialized".into(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "database".into(),
                message: e.to_string(),
            });
        }
        _ => {}
    }

    if let Err(e) = dto.admin.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation".into(),
            message: e.to_string(),
        });
    }
    if let Err(e) = dto.tenant.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "validation".into(),
            message: e.to_string(),
        });
    }

    // 2. Create Tenant
    let tenant = match state.db.tenant_repo().create(dto.tenant.clone()).await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "database".into(),
                message: e.to_string(),
            })
        }
    };

    // 3. Create Admin User
    let mut admin_dto = dto.admin.clone();
    admin_dto.roles = Some(vec![UserRole::Admin]);

    match state
        .db
        .user_repo()
        .create(tenant.id, admin_dto.into())
        .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "database".into(),
            message: e.to_string(),
        }),
    }
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
) -> impl Responder {
    match state.db.tenant_repo().delete(auth.0.tenant_id).await {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
            error: "not_found".into(),
            message: "Tenant not found".into(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "database".into(),
            message: e.to_string(),
        }),
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
