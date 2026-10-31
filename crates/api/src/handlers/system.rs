use crate::AppState;
use crate::dto::user::CreateUserDto;
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::tenant::{CompanyProfile, CreateTenantDto, GeoCoordinates};
use agrocore_logging::{error, info, warn};
use agrocore_shared::SharedError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, ToSchema)]
pub struct SystemStatusResponse {
    pub initialized: bool,
    pub demo_mode: bool,
    pub demo_readonly: bool,
    pub uptime_seconds: u64,
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
        demo_mode: state.is_demo_mode,
        demo_readonly: state.demo_readonly,
        uptime_seconds: state.startup_time.elapsed().as_secs(),
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
    info!("Starting initial system setup");

    dto.admin.validate().map_err(|e| {
        warn!("Admin validation failed: {}", e);
        SharedError::Validation(e.to_string())
    })?;
    dto.tenant.validate().map_err(|e| {
        warn!("Tenant validation failed: {}", e);
        SharedError::Validation(e.to_string())
    })?;

    // The setup endpoint runs *before* any tenant exists, so no tenant can be
    // pinned. The nil tenant matches no tenant: the policies deny data access,
    // while `tenants_insert` still allows this one setup INSERT. That is the
    // intent - setup may create the first tenant, but must not read or write
    // tenant data.
    // Use raw connection to SET ROLE before transaction (SET ROLE not allowed in transaction block)
    let mut conn = state.db.pool().acquire().await.map_err(|e| {
        error!("Failed to acquire connection: {}", e);
        SharedError::Database(e.to_string())
    })?;
    // SET ROLE must be outside transaction block
    sqlx::query("SET ROLE agrocore_app")
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            error!("Failed to SET ROLE: {}", e);
            SharedError::Database(e.to_string())
        })?;
    // SET LOCAL configs before transaction
    sqlx::query(
        "SELECT set_config('app.current_tenant_id', $1, true), \
                set_config('app.is_superadmin', $2, true)",
    )
    .bind(Uuid::nil().to_string())
    .bind("true")
    .execute(&mut *conn)
    .await
    .map_err(|e| {
        error!("Failed to set tenant config: {}", e);
        SharedError::Database(e.to_string())
    })?;

    // 1. Check if already initialized (outside transaction, on raw connection)
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| {
            error!("Failed to check initialization status: {}", e);
            SharedError::Database(e.to_string())
        })?;

    if count > 0 {
        warn!("Setup attempted on already initialized system");
        return Err(SharedError::Validation("System is already initialized".into()).into());
    }

    // 2. Hash password
    let password_hash =
        agrocore_infrastructure::password::hash_password(dto.admin.password.clone())
            .await
            .map_err(|e| {
                error!("Password hashing failed: {}", e);
                SharedError::Internal("Hashing error".to_string())
            })?;

    // 3. Call SECURITY DEFINER function OUTSIDE transaction
    // The function runs as table owner (agrocore) and handles FORCE RLS internally.
    // It must be called outside any transaction block for ALTER TABLE to work.
    let tenant_id: Uuid =
        sqlx::query_scalar("SELECT public.initial_system_setup($1, $2, $3, $4, $5, $6, $7)")
            .bind(&dto.tenant.name)
            .bind(&dto.tenant.slug)
            .bind(serde_json::to_value(dto.tenant.config.clone().unwrap_or_default()).unwrap())
            .bind(&dto.admin.firstname)
            .bind(&dto.admin.lastname)
            .bind(&dto.admin.email)
            .bind(&password_hash)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to create tenant via setup function: {}", e);
                agrocore_infrastructure::PostgresDb::map_db_error(e)
            })?;

    // 4. Fetch the created tenant (still outside transaction)
    let tenant = sqlx::query_as::<_, agrocore_domain::entities::tenant::Tenant>(
        r#"SELECT * FROM tenants WHERE id = $1"#,
    )
    .bind(tenant_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| {
        error!("Failed to fetch created tenant: {}", e);
        agrocore_infrastructure::PostgresDb::map_db_error(e)
    })?;

    // Release connection back to pool
    drop(conn);

    // 5. Publish TenantCreated event after successful setup
    info!("Publishing TenantCreated event for tenant: {}", tenant.id);
    let tenant_event = agrocore_messaging::Event::new(
        tenant.id.to_string(),
        agrocore_messaging::GlobalEvent::TenantCreated(tenant.clone()),
    );
    let _ = crate::publish_event(
        state.messaging.as_ref(),
        "system.tenant.created",
        &tenant_event,
    )
    .await;

    info!("Initial setup completed successfully");
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

/// DTO for updating the company profile of the current tenant.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCompanyProfileDto {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub fax: Option<String>,
    pub website: Option<String>,
    pub tax_id: Option<String>,
    pub vat_id: Option<String>,
    pub registration_number: Option<String>,
    /// Latitude of the company location (from map picker)
    pub latitude: Option<f64>,
    /// Longitude of the company location (from map picker)
    pub longitude: Option<f64>,
}

#[utoipa::path(
    put,
    path = "/api/v1/system/company-profile",
    request_body = UpdateCompanyProfileDto,
    responses(
        (status = 200, description = "Company profile updated", body = CompanyProfile),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Tenant not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "system"
)]
pub async fn update_company_profile(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
    dto: web::Json<UpdateCompanyProfileDto>,
) -> Result<HttpResponse, ApiError> {
    let tenant_id = auth.0.tenant_id;
    info!("Updating company profile for tenant {}", tenant_id);

    // Fetch current tenant
    let tenant = state
        .db
        .tenant_repo()
        .find_by_id(tenant_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Tenant not found".into()))?;

    // Build updated company profile from current config + DTO overrides
    let current_profile = tenant.config.company_profile.clone();
    let profile = match current_profile {
        Some(mut profile) => {
            if let Some(name) = &dto.name {
                profile.name = name.clone();
            }
            if let Some(address) = &dto.address {
                profile.address = address.clone();
            }
            if let Some(city) = &dto.city {
                profile.city = Some(city.clone());
            }
            if let Some(postal_code) = &dto.postal_code {
                profile.postal_code = Some(postal_code.clone());
            }
            if let Some(state) = &dto.state {
                profile.state = Some(state.clone());
            }
            if let Some(country) = &dto.country {
                profile.country = country.clone();
            }
            if let Some(email) = &dto.email {
                profile.email = email.clone();
            }
            if let Some(phone) = &dto.phone {
                profile.phone = phone.clone();
            }
            if let Some(mobile) = &dto.mobile {
                profile.mobile = Some(mobile.clone());
            }
            if let Some(fax) = &dto.fax {
                profile.fax = Some(fax.clone());
            }
            if let Some(website) = &dto.website {
                profile.website = Some(website.clone());
            }
            if let Some(tax_id) = &dto.tax_id {
                profile.tax_id = Some(tax_id.clone());
            }
            if let Some(vat_id) = &dto.vat_id {
                profile.vat_id = Some(vat_id.clone());
            }
            if let Some(registration_number) = &dto.registration_number {
                profile.registration_number = Some(registration_number.clone());
            }
            // Update coordinates from map picker
            if dto.latitude.is_some() || dto.longitude.is_some() {
                profile.coordinates = Some(GeoCoordinates {
                    lat: dto.latitude.unwrap_or(profile.coordinates.as_ref().map(|c| c.lat).unwrap_or(0.0)),
                    lng: dto.longitude.unwrap_or(profile.coordinates.as_ref().map(|c| c.lng).unwrap_or(0.0)),
                });
            }
            profile
        }
        None => {
            // Create new profile if none exists
            CompanyProfile {
                name: dto.name.clone().unwrap_or_default(),
                address: dto.address.clone().unwrap_or_default(),
                city: dto.city.clone(),
                postal_code: dto.postal_code.clone(),
                state: dto.state.clone(),
                country: dto.country.clone().unwrap_or_default(),
                email: dto.email.clone().unwrap_or_default(),
                phone: dto.phone.clone().unwrap_or_default(),
                mobile: dto.mobile.clone(),
                fax: dto.fax.clone(),
                website: dto.website.clone(),
                tax_id: dto.tax_id.clone(),
                vat_id: dto.vat_id.clone(),
                registration_number: dto.registration_number.clone(),
                coordinates: if dto.latitude.is_some() && dto.longitude.is_some() {
                    Some(GeoCoordinates {
                        lat: dto.latitude.unwrap_or(0.0),
                        lng: dto.longitude.unwrap_or(0.0),
                    })
                } else {
                    None
                },
            }
        }
    };

    // Update tenant config with new company profile
    let mut config = tenant.config.clone();
    config.company_profile = Some(profile.clone());

    // Persist via SQL (RLS-aware)
    sqlx::query(
        "UPDATE tenants SET config = jsonb_set(config, '{company_profile}', $1) WHERE id = $2"
    )
    .bind(serde_json::to_value(&profile).map_err(|e| {
        error!("Failed to serialize company profile: {}", e);
        SharedError::Internal("Failed to serialize company profile".to_string())
    })?)
    .bind(tenant_id)
    .execute(&*state.db.pool())
    .await
    .map_err(|e| {
        error!("Failed to update company profile: {}", e);
        agrocore_infrastructure::PostgresDb::map_db_error(e)
    })?;

    info!("Company profile updated successfully for tenant {}", tenant_id);
    Ok(HttpResponse::Ok().json(profile))
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
