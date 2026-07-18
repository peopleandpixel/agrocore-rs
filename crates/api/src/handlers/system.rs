use crate::AppState;
use crate::dto::CreateUserDto;
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::tenant::CreateTenantDto;
use agrocore_domain::entities::user::UserRole;
use agrocore_shared::SharedError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
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
    let count = match state.db.user_repo().count_all().await {
        Ok(c) => c,
        Err(e) if e.to_string().contains("does not exist") => 0,
        Err(e) => return Err(e.into()),
    };
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
    dto.admin
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    dto.tenant
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;

    let pool = state.db.pool();
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

    // 1. Check if already initialized (within transaction for safety)
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

    if count > 0 {
        return Err(SharedError::Validation("System is already initialized".into()).into());
    }

    // 2. Create Tenant
    let tenant_id = Uuid::new_v4();
    let tenant = sqlx::query_as::<_, agrocore_domain::entities::tenant::Tenant>(
        r#"INSERT INTO tenants (id, name, slug, config, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, true, NOW(), NOW())
           RETURNING *"#,
    )
    .bind(tenant_id)
    .bind(&dto.tenant.name)
    .bind(&dto.tenant.slug)
    .bind(serde_json::to_value(dto.tenant.config.clone().unwrap_or_default()).unwrap())
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| agrocore_infrastructure::PostgresDb::map_db_error(e))?;

    // 3. Create Admin User
    let admin_id = Uuid::new_v4();
    // Replicating Argon2 hashing from PgUserRepo to ensure atomicity within the transaction
    use argon2::PasswordHasher;
    let password_hash = argon2::Argon2::default()
        .hash_password(dto.admin.password.as_bytes())
        .map_err(|e| SharedError::Internal(format!("Hashing error: {}", e)))?
        .to_string();

    let roles = vec![UserRole::Admin];

    sqlx::query(
        r#"INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())"#)
    .bind(admin_id)
    .bind(tenant.id)
    .bind(&dto.admin.firstname)
    .bind(&dto.admin.lastname)
    .bind(&dto.admin.email)
    .bind(password_hash)
    .bind(serde_json::to_value(&roles).unwrap())
    .execute(&mut *tx)
    .await
    .map_err(|e| agrocore_infrastructure::PostgresDb::map_db_error(e))?;

    tx.commit()
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

    // 4 Publish TenantCreated event after successful commit
    let _ = state
        .messaging
        .publish(
            "system.tenant.created",
            &agrocore_messaging::Event::new(
                tenant.id.to_string(),
                agrocore_messaging::GlobalEvent::TenantCreated(tenant.clone()),
            ),
        )
        .await;

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
