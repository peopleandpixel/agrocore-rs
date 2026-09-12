//! Demo API Handlers

use crate::AppState;
use crate::dto::demo::{DemoDataSummary, DemoSeedRequest, DemoSeedResponse};
use crate::error::ApiError;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::user::UserRole;
use agrocore_shared::SharedError;
use serde_json::json;
use sqlx::query;
use uuid::Uuid;
use validator::Validate;

/// Configure demo routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/demo")
            .service(web::resource("/seed").route(web::post().to(seed_demo)))
            .service(web::resource("/reset").route(web::post().to(reset_demo)))
            .service(web::resource("/summary").route(web::get().to(demo_summary))),
    );
}

/// Seed demo data
#[utoipa::path(
    post,
    path = "/api/v1/demo/seed",
    request_body = DemoSeedRequest,
    responses(
        (status = 201, description = "Demo data seeded", body = DemoSeedResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Demo data already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "demo"
)]
pub async fn seed_demo(
    state: web::Data<AppState>,
    dto: web::Json<DemoSeedRequest>,
) -> Result<HttpResponse, ApiError> {
    let req = dto.into_inner();
    req.validate()
        .map_err(|e| ApiError::validation(e.to_string()))?;

    let pool = state.db.pool();

    // Check if tenant already exists
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM tenants WHERE slug = $1 AND is_active = true")
            .bind(&req.tenant)
            .fetch_optional(pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

    if existing.is_some() && !req.reset {
        return Err(ApiError::validation(
            "Demo tenant already exists. Use reset=true to overwrite.",
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

    let tenant_id = if let Some(existing_id) = existing {
        // Delete existing demo data (cascades to related tables)
        query("DELETE FROM tenants WHERE id = $1")
            .bind(existing_id)
            .execute(&mut *tx)
            .await
            .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;
        existing_id
    } else {
        Uuid::new_v4()
    };

    // Create Tenant
    let tenant = sqlx::query_as::<_, agrocore_domain::entities::tenant::Tenant>(
        r#"INSERT INTO tenants (id, name, slug, config, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, true, NOW(), NOW())
           RETURNING *"#,
    )
    .bind(tenant_id)
    .bind(&req.tenant)
    .bind(&req.tenant)
    .bind(serde_json::json!({
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
            "Reports",
            "Inventory",
            "Livestock",
            "Equipment"
        ],
        "custom_field_schemas": {}
    }))
    .fetch_one(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create Admin User
    let user_id = Uuid::new_v4();
    use argon2::PasswordHasher;
    use password_hash::SaltString;
    use rand::thread_rng;

    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = argon2::Argon2::default()
        .hash_password(b"demo123", &salt)
        .map_err(|e| SharedError::Internal(format!("Hashing error: {}", e)))?
        .to_string();

    let roles = vec![UserRole::Admin];

    sqlx::query(
        r#"INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())"#
    )
    .bind(user_id)
    .bind(tenant.id)
    .bind("Demo")
    .bind("Admin")
    .bind(&req.user)
    .bind(password_hash)
    .bind(serde_json::to_value(&roles).unwrap())
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo sites
    let site1_id = Uuid::new_v4();
    let site2_id = Uuid::new_v4();
    let site3_id = Uuid::new_v4();

    query(
        r#"INSERT INTO sites (id, tenant_id, label, code, description, site_type, crop_type, variety, area, center_lng, center_lat, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())"#
    )
    .bind(site1_id)
    .bind(tenant.id)
    .bind("Hof Nord - Weizen")
    .bind("HN-001")
    .bind("Hauptfeld im Norden, Weizenanbau")
    .bind(json!({"type": "field", "category": "arable"}))
    .bind(json!({"crop": "wheat", "category": "cereals"}))
    .bind("Akteur")
    .bind(12.5)
    .bind(-8.61308)
    .bind(41.14961)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO sites (id, tenant_id, label, code, description, site_type, crop_type, variety, area, center_lng, center_lat, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())"#
    )
    .bind(site2_id)
    .bind(tenant.id)
    .bind("Hof Süd - Mais")
    .bind("HS-002")
    .bind("Südfeld, Maissilage für Vieh")
    .bind(json!({"type": "field", "category": "arable"}))
    .bind(json!({"crop": "maize", "category": "cereals"}))
    .bind("Ronaldinio")
    .bind(8.3)
    .bind(-8.60854)
    .bind(41.14233)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO sites (id, tenant_id, label, code, description, site_type, crop_type, variety, area, center_lng, center_lat, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())"#
    )
    .bind(site3_id)
    .bind(tenant.id)
    .bind("Weide Ost - Rinder")
    .bind("WO-003")
    .bind("Weidefläche für Milchviehhaltung")
    .bind(json!({"type": "pasture", "category": "grassland"}))
    .bind(json!({"crop": "grass", "category": "grassland"}))
    .bind("")
    .bind(15.0)
    .bind(-8.59912)
    .bind(41.15124)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo equipment
    let eq1_id = Uuid::new_v4();
    let eq2_id = Uuid::new_v4();
    let eq3_id = Uuid::new_v4();

    query(
        r#"INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, false, true, NOW(), NOW())"#
    )
    .bind(eq1_id)
    .bind(tenant.id)
    .bind("John Deere 6R 180")
    .bind("JD-6R180")
    .bind(json!({"category": "tractor", "brand": "John Deere", "model": "6R 180", "year": 2022, "power_kw": 132}))
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, false, true, NOW(), NOW())"#
    )
    .bind(eq2_id)
    .bind(tenant.id)
    .bind("Claas Jaguar 960")
    .bind("CJ-960")
    .bind(json!({"category": "harvester", "brand": "Claas", "model": "Jaguar 960", "year": 2021, "power_kw": 375}))
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO equipment (id, tenant_id, label, code, equipment_type, in_usage, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, false, true, NOW(), NOW())"#
    )
    .bind(eq3_id)
    .bind(tenant.id)
    .bind("Amazone ZA-M 1501")
    .bind("AZ-1501")
    .bind(json!({"category": "sprayer", "brand": "Amazone", "model": "ZA-M 1501", "year": 2023, "tank_liters": 1500}))
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo workers
    let w1_id = Uuid::new_v4();
    let w2_id = Uuid::new_v4();

    query(
        r#"INSERT INTO workers (id, tenant_id, firstname, lastname, email, phone, role, hourly_rate, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())"#
    )
    .bind(w1_id)
    .bind(tenant.id)
    .bind("Hans")
    .bind("Müller")
    .bind("hans.mueller@demo.local")
    .bind("+49 151 1234567")
    .bind("operator")
    .bind(22.50)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO workers (id, tenant_id, firstname, lastname, email, phone, role, hourly_rate, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())"#
    )
    .bind(w2_id)
    .bind(tenant.id)
    .bind("Maria")
    .bind("Santos")
    .bind("maria.santos@demo.local")
    .bind("+351 912 345678")
    .bind("operator")
    .bind(18.00)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo orders
    let order1_id = Uuid::new_v4();
    let order2_id = Uuid::new_v4();

    query(
        r#"INSERT INTO orders (id, tenant_id, label, title, description, priority, status, order_type, assigned_to, assigned_worker_ids, site_ids, scheduled_start, scheduled_end, planned_date, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $9, $10, $11, $12, $13, true, NOW(), NOW())"#
    )
    .bind(order1_id)
    .bind(tenant.id)
    .bind("Aussaat Weizen HN-001")
    .bind("Weizen Aussaat auf Hof Nord")
    .bind("Aussaat Winterweizen Sorte Akteur, 180 kg/ha")
    .bind(5)
    .bind(json!({"state": "planned", "progress": 0}))
    .bind("field_work")
    .bind(user_id)
    .bind(serde_json::to_value(vec![w1_id]).unwrap())
    .bind(serde_json::to_value(vec![site1_id]).unwrap())
    .bind(chrono::Utc::now() + chrono::Duration::days(1))
    .bind(chrono::Utc::now() + chrono::Duration::days(2))
    .bind(chrono::Utc::now().date_naive() + chrono::Duration::days(1))
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO orders (id, tenant_id, label, title, description, priority, status, order_type, assigned_to, assigned_worker_ids, site_ids, scheduled_start, scheduled_end, planned_date, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, true, NOW(), NOW())"#
    )
    .bind(order2_id)
    .bind(tenant.id)
    .bind("Gülle Ausbringung WO-003")
    .bind("Gülleausbringung auf Weide Ost")
    .bind("Organische Düngung der Weidefläche, 30 m³/ha")
    .bind(3)
    .bind(json!({"state": "planned", "progress": 0}))
    .bind("fertilization")
    .bind(user_id)
    .bind(serde_json::to_value(vec![w2_id]).unwrap())
    .bind(serde_json::to_value(vec![site3_id]).unwrap())
    .bind(chrono::Utc::now() + chrono::Duration::days(3))
    .bind(chrono::Utc::now() + chrono::Duration::days(3))
    .bind(chrono::Utc::now().date_naive() + chrono::Duration::days(3))
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo inventory items
    let inv1_id = Uuid::new_v4();
    let inv2_id = Uuid::new_v4();
    let inv3_id = Uuid::new_v4();

    query(
        r#"INSERT INTO inventory_items (id, tenant_id, name, sku, category, unit, current_stock, min_stock, max_stock, unit_cost, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())"#
    )
    .bind(inv1_id)
    .bind(tenant.id)
    .bind("Weizensaatgut Akteur")
    .bind("SEED-WHT-AKT")
    .bind("seeds")
    .bind("kg")
    .bind(2500.0)
    .bind(500.0)
    .bind(5000.0)
    .bind(0.85)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO inventory_items (id, tenant_id, name, sku, category, unit, current_stock, min_stock, max_stock, unit_cost, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())"#
    )
    .bind(inv2_id)
    .bind(tenant.id)
    .bind("NPK 15-15-15 Dünger")
    .bind("FERT-NPK-15")
    .bind("fertilizer")
    .bind("kg")
    .bind(3000.0)
    .bind(1000.0)
    .bind(10000.0)
    .bind(0.65)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO inventory_items (id, tenant_id, name, sku, category, unit, current_stock, min_stock, max_stock, unit_cost, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())"#
    )
    .bind(inv3_id)
    .bind(tenant.id)
    .bind("Rindergülle")
    .bind("MANU-COW")
    .bind("manure")
    .bind("m3")
    .bind(150.0)
    .bind(20.0)
    .bind(500.0)
    .bind(0.0)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo inventory location
    let loc_id = Uuid::new_v4();
    query(
        r#"INSERT INTO inventory_locations (id, tenant_id, name, code, location_type, address, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())"#
    )
    .bind(loc_id)
    .bind(tenant.id)
    .bind("Hauptlager Scheune")
    .bind("HL-001")
    .bind("warehouse")
    .bind("Hof Nord, Scheune 1")
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo inventory balances
    query(
        r#"INSERT INTO inventory_transactions (id, tenant_id, item_id, location_id, transaction_type, quantity, unit_cost, reference_type, reference_id, notes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#
    )
    .bind(Uuid::new_v4())
    .bind(tenant.id)
    .bind(inv1_id)
    .bind(loc_id)
    .bind("stock_in")
    .bind(2500.0)
    .bind(0.85)
    .bind("initial")
    .bind(tenant.id)
    .bind("Initial demo stock")
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO inventory_transactions (id, tenant_id, item_id, location_id, transaction_type, quantity, unit_cost, reference_type, reference_id, notes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#
    )
    .bind(Uuid::new_v4())
    .bind(tenant.id)
    .bind(inv2_id)
    .bind(loc_id)
    .bind("stock_in")
    .bind(3000.0)
    .bind(0.65)
    .bind("initial")
    .bind(tenant.id)
    .bind("Initial demo stock")
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO inventory_transactions (id, tenant_id, item_id, location_id, transaction_type, quantity, unit_cost, reference_type, reference_id, notes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#
    )
    .bind(Uuid::new_v4())
    .bind(tenant.id)
    .bind(inv3_id)
    .bind(loc_id)
    .bind("stock_in")
    .bind(150.0)
    .bind(0.0)
    .bind("initial")
    .bind(tenant.id)
    .bind("Initial demo stock")
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    // Create demo animals
    let animal1_id = Uuid::new_v4();
    let animal2_id = Uuid::new_v4();
    let animal3_id = Uuid::new_v4();

    query(
        r#"INSERT INTO animals (id, tenant_id, tag_number, name, species, breed, sex, birth_date, mother_id, father_id, status, location_id, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())"#
    )
    .bind(animal1_id)
    .bind(tenant.id)
    .bind("DE 09 12345678")
    .bind("Bella")
    .bind("cattle")
    .bind("Holstein")
    .bind("female")
    .bind("2022-03-15")
    .bind(Option::<Uuid>::None)
    .bind(Option::<Uuid>::None)
    .bind("active")
    .bind(site3_id)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO animals (id, tenant_id, tag_number, name, species, breed, sex, birth_date, mother_id, father_id, status, location_id, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())"#
    )
    .bind(animal2_id)
    .bind(tenant.id)
    .bind("DE 09 12345679")
    .bind("Lotte")
    .bind("cattle")
    .bind("Holstein")
    .bind("female")
    .bind("2021-11-22")
    .bind(Option::<Uuid>::None)
    .bind(Option::<Uuid>::None)
    .bind("active")
    .bind(site3_id)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    query(
        r#"INSERT INTO animals (id, tenant_id, tag_number, name, species, breed, sex, birth_date, mother_id, father_id, status, location_id, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())"#
    )
    .bind(animal3_id)
    .bind(tenant.id)
    .bind("DE 09 12345680")
    .bind("Bruno")
    .bind("cattle")
    .bind("Fleckvieh")
    .bind("male")
    .bind("2023-01-10")
    .bind(Option::<Uuid>::None)
    .bind(Option::<Uuid>::None)
    .bind("active")
    .bind(site3_id)
    .execute(&mut *tx)
    .await
    .map_err(agrocore_infrastructure::PostgresDb::map_db_error)?;

    tx.commit()
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

    // Publish TenantCreated event
    let _ = state
        .messaging
        .publish(
            "system.tenant.created".to_string(),
            &agrocore_messaging::Event::new(
                tenant.id.to_string(),
                agrocore_messaging::GlobalEvent::TenantCreated(tenant.clone()),
            ),
        )
        .await;

    Ok(HttpResponse::Created().json(DemoSeedResponse {
        success: true,
        message: format!("Demo data seeded for tenant '{}'", req.tenant),
        tenant_id: Some(tenant.id),
        user_id: Some(user_id),
    }))
}

/// Reset demo data (delete and reseed)
#[utoipa::path(
    post,
    path = "/api/v1/demo/reset",
    request_body = DemoSeedRequest,
    responses(
        (status = 200, description = "Demo data reset and reseeded", body = DemoSeedResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "demo"
)]
pub async fn reset_demo(
    state: web::Data<AppState>,
    dto: web::Json<DemoSeedRequest>,
) -> Result<HttpResponse, ApiError> {
    let mut req = dto.into_inner();
    req.reset = true;
    seed_demo(state, web::Json(req)).await
}

/// Get demo data summary
#[utoipa::path(
    get,
    path = "/api/v1/demo/summary",
    responses(
        (status = 200, description = "Demo data summary", body = DemoDataSummary),
        (status = 404, description = "Demo tenant not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "demo"
)]
pub async fn demo_summary(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let pool = state.db.pool();

    // Find demo tenant
    let tenant: Option<agrocore_domain::entities::tenant::Tenant> =
        sqlx::query_as("SELECT * FROM tenants WHERE slug = 'demo' AND is_active = true LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

    let tenant = tenant.ok_or_else(|| ApiError::not_found("Demo tenant not found"))?;

    // Count demo data
    let sites_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sites WHERE tenant_id = $1 AND is_active = true")
            .bind(tenant.id)
            .fetch_one(pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

    let equipment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM equipment WHERE tenant_id = $1 AND is_active = true",
    )
    .bind(tenant.id)
    .fetch_one(pool)
    .await
    .map_err(|e| SharedError::Database(e.to_string()))?;

    let orders_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE tenant_id = $1 AND is_active = true")
            .bind(tenant.id)
            .fetch_one(pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

    let workers_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM workers WHERE tenant_id = $1 AND is_active = true",
    )
    .bind(tenant.id)
    .fetch_one(pool)
    .await
    .map_err(|e| SharedError::Database(e.to_string()))?;

    let inventory_items_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inventory_items WHERE tenant_id = $1 AND is_active = true",
    )
    .bind(tenant.id)
    .fetch_one(pool)
    .await
    .map_err(|e| SharedError::Database(e.to_string()))?;

    let animals_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM animals WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant.id)
    .fetch_one(pool)
    .await
    .map_err(|e| SharedError::Database(e.to_string()))?;

    Ok(HttpResponse::Ok().json(DemoDataSummary {
        tenant_id: tenant.id,
        tenant_name: tenant.name,
        sites_count: sites_count as usize,
        equipment_count: equipment_count as usize,
        orders_count: orders_count as usize,
        workers_count: workers_count as usize,
        inventory_items_count: inventory_items_count as usize,
        animals_count: animals_count as usize,
    }))
}
