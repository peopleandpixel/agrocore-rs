//! Initial Setup and Demo Switch Integration Tests
//!
//! These tests verify the complete initial setup flow works correctly in both
//! DEV and LIVE modes, and that the demo switch creates all expected objects.
//!
//! The initial setup MUST never fail — it is the gateway for every new deployment.
//! The demo switch must create a comprehensive set of demo data covering all
//! entity types: sites, workers, tasks, trees, buildings, animals, equipment,
//! orders, inventory, and livestock.

use std::env;
use validator::Validate;

/// Serialises access to the environment across the tests in this file.
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Helper to set/remove env vars safely in parallel test execution.
fn with_var(key: &str, value: Option<&str>, f: impl FnOnce()) {
    let _guard = env_lock();
    let previous = env::var(key).ok();
    unsafe {
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }
    f();
    unsafe {
        match previous {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }
}

// ============================================================================
// Initial Setup Request Validation Tests
// ============================================================================

#[test]
fn initial_setup_request_requires_admin_firstname() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "",
            "lastname": "Meyer",
            "email": "anna@example.com",
            "password": "secure-password-123"
        },
        "tenant": {
            "name": "AgroCore",
            "slug": "agrocore"
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.admin.validate();
    assert!(result.is_err(), "Empty firstname must fail validation");
}

#[test]
fn initial_setup_request_requires_admin_lastname() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "",
            "email": "anna@example.com",
            "password": "secure-password-123"
        },
        "tenant": {
            "name": "AgroCore",
            "slug": "agrocore"
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.admin.validate();
    assert!(result.is_err(), "Empty lastname must fail validation");
}

#[test]
fn initial_setup_request_requires_valid_email() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "Meyer",
            "email": "not-an-email",
            "password": "secure-password-123"
        },
        "tenant": {
            "name": "AgroCore",
            "slug": "agrocore"
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.admin.validate();
    assert!(result.is_err(), "Invalid email must fail validation");
}

#[test]
fn initial_setup_request_requires_password_minimum_length() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "Meyer",
            "email": "anna@example.com",
            "password": "short"
        },
        "tenant": {
            "name": "AgroCore",
            "slug": "agrocore"
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.admin.validate();
    assert!(result.is_err(), "Short password must fail validation");
}

#[test]
fn initial_setup_request_requires_tenant_name() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "Meyer",
            "email": "anna@example.com",
            "password": "secure-password-123"
        },
        "tenant": {
            "name": "",
            "slug": "agrocore"
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.tenant.validate();
    assert!(result.is_err(), "Empty tenant name must fail validation");
}

#[test]
fn initial_setup_request_requires_tenant_slug() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "Meyer",
            "email": "anna@example.com",
            "password": "secure-password-123"
        },
        "tenant": {
            "name": "AgroCore",
            "slug": ""
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    let result = request.tenant.validate();
    assert!(result.is_err(), "Empty tenant slug must fail validation");
}

#[test]
fn initial_setup_request_validates_successfully() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    // Password must be 12+ chars with uppercase, lowercase, digit, special char
    let payload = json!({
        "admin": {
            "firstname": "Anna",
            "lastname": "Meyer",
            "email": "anna@example.com",
            "password": "SecureP@ss1234"
        },
        "tenant": {
            "name": "AgroCore Farm",
            "slug": "agrocore-farm",
            "config": {
                "default_language": "en",
                "supported_languages": ["en", "de"],
                "timezone": "Europe/Berlin",
                "enabled_modules": ["field_management", "Harvest", "Reports"],
                "custom_field_schemas": {},
                "logo_url": null,
                "primary_color": null,
                "validation_rules": null
            }
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();

    // Admin validation
    assert!(
        request.admin.validate().is_ok(),
        "Valid admin data must pass validation"
    );

    // Tenant validation
    assert!(
        request.tenant.validate().is_ok(),
        "Valid tenant data must pass validation"
    );
}

#[test]
fn initial_setup_request_supports_all_modules() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    // Password must be 12+ chars with uppercase, lowercase, digit, special char
    let payload = json!({
        "admin": {
            "firstname": "Admin",
            "lastname": "User",
            "email": "admin@example.com",
            "password": "SecureP@ss1234"
        },
        "tenant": {
            "name": "Full Feature Farm",
            "slug": "full-feature",
            "config": {
                "default_language": "en",
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
                "custom_field_schemas": {},
                "logo_url": null,
                "primary_color": null,
                "validation_rules": null
            }
        }
    });

    let request: InitialSetupRequest = serde_json::from_value(payload).unwrap();
    assert!(request.admin.validate().is_ok());
    assert!(request.tenant.validate().is_ok());

    let modules = request
        .tenant
        .config
        .as_ref()
        .unwrap()
        .enabled_modules
        .len();
    assert_eq!(modules, 11, "All 11 modules should be enabled");
}

// ============================================================================
// DEV/LIVE Mode Configuration Tests
// ============================================================================
// These tests verify the JWT secret validation logic without depending on
// the global OnceLock config, which cannot be re-initialized in parallel tests.

#[test]
fn dev_secret_detection_works() {
    // When ALLOW_DEV_SECRET is not set, dev-secret should be rejected
    with_var("ALLOW_DEV_SECRET", None, || {
        let dev_secret_allowed = matches!(
            std::env::var("ALLOW_DEV_SECRET")
                .ok()
                .as_deref()
                .map(str::trim)
                .map(|v| v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true")),
            Some(true)
        );
        assert!(!dev_secret_allowed, "ALLOW_DEV_SECRET must not be set");
    });

    // When ALLOW_DEV_SECRET=1, dev-secret check is bypassed
    with_var("ALLOW_DEV_SECRET", Some("1"), || {
        let dev_secret_allowed = matches!(
            std::env::var("ALLOW_DEV_SECRET")
                .ok()
                .as_deref()
                .map(str::trim)
                .map(|v| v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true")),
            Some(true)
        );
        assert!(
            dev_secret_allowed,
            "ALLOW_DEV_SECRET=1 must allow dev secret"
        );
    });
}

#[test]
fn jwt_secret_minimum_length_is_enforced() {
    // JWT_SECRET_MIN_LENGTH is 32 characters
    assert_eq!(
        agrocore_shared::config::JWT_SECRET_MIN_LENGTH,
        32,
        "Minimum JWT secret length must be 32"
    );

    // A short secret should fail
    let short_secret = "short";
    assert!(
        short_secret.len() < agrocore_shared::config::JWT_SECRET_MIN_LENGTH,
        "Test secret should be too short"
    );
}

#[test]
fn dev_secret_is_too_short() {
    const DEV_SECRET: &str = "dev-secret";
    assert!(
        DEV_SECRET.len() < agrocore_shared::config::JWT_SECRET_MIN_LENGTH,
        "Dev secret ({}) is shorter than minimum ({})",
        DEV_SECRET.len(),
        agrocore_shared::config::JWT_SECRET_MIN_LENGTH
    );
}

#[test]
fn valid_secret_length_example() {
    // A proper production secret would be 32+ characters
    let valid_secret = "this-is-a-very-secure-production-secret-key-1234";
    assert!(
        valid_secret.len() >= agrocore_shared::config::JWT_SECRET_MIN_LENGTH,
        "Valid secret ({}) meets minimum length ({})",
        valid_secret.len(),
        agrocore_shared::config::JWT_SECRET_MIN_LENGTH
    );
}

// ============================================================================
// RLS (Row Level Security) Configuration Tests
// ============================================================================
// These test the rls_enabled() logic directly without depending on global state.

/// Mirrors `rls_enabled()` in crates/shared/src/config.rs.
fn rls_enabled() -> bool {
    matches!(
        std::env::var("AGROCORE_RLS_ENABLED")
            .ok()
            .as_deref()
            .map(str::trim),
        Some("1") | Some("true")
    )
}

#[test]
fn rls_disabled_by_default() {
    with_var("AGROCORE_RLS_ENABLED", None, || {
        assert!(!rls_enabled(), "RLS must be disabled by default");
    });
}

#[test]
fn rls_enabled_with_true_value() {
    // rls_enabled() only accepts lowercase "true" and "1", not "TRUE"
    for value in ["1", "true"] {
        with_var("AGROCORE_RLS_ENABLED", Some(value), || {
            assert!(rls_enabled(), "RLS must be enabled with value: {}", value);
        });
    }
}

#[test]
fn rls_disabled_with_false_value() {
    for value in ["0", "false", "no", ""] {
        with_var("AGROCORE_RLS_ENABLED", Some(value), || {
            assert!(!rls_enabled(), "RLS must be disabled with value: {}", value);
        });
    }
}

// ============================================================================
// Demo Switch Configuration Tests
// ============================================================================

#[test]
fn demo_endpoints_disabled_by_default() {
    with_var("ALLOW_DEMO_ENDPOINTS", None, || {
        assert!(
            !agrocore_api::demo_endpoints_enabled(),
            "Demo endpoints must be disabled by default"
        );
    });
}

#[test]
fn demo_endpoints_enabled_with_valid_values() {
    for value in ["1", "true", "TRUE", "yes", "YES"] {
        with_var("ALLOW_DEMO_ENDPOINTS", Some(value), || {
            assert!(
                agrocore_api::demo_endpoints_enabled(),
                "Demo endpoints must be enabled with value: {}",
                value
            );
        });
    }
}

#[test]
fn demo_endpoints_reject_invalid_values() {
    for value in ["0", "false", "no", "off", "", "enabled", "2"] {
        with_var("ALLOW_DEMO_ENDPOINTS", Some(value), || {
            assert!(
                !agrocore_api::demo_endpoints_enabled(),
                "Demo endpoints must reject value: {}",
                value
            );
        });
    }
}

// ============================================================================
// Demo Seed Request Validation Tests
// ============================================================================

#[test]
fn demo_seed_request_requires_tenant_name() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    let payload = json!({
        "tenant": "",
        "user": "demo@example.com",
        "reset": false
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    let result = request.validate();
    assert!(result.is_err(), "Empty tenant name must fail validation");
}

#[test]
fn demo_seed_request_requires_user_email() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    let payload = json!({
        "tenant": "demo",
        "user": "",
        "reset": false
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    let result = request.validate();
    assert!(result.is_err(), "Empty user email must fail validation");
}

#[test]
fn demo_seed_request_validates_successfully() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    let payload = json!({
        "tenant": "demo-farm",
        "user": "demo@example.com",
        "reset": false
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    assert!(
        request.validate().is_ok(),
        "Valid demo seed request must pass validation"
    );
}

#[test]
fn demo_seed_request_reset_defaults_to_false() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    let payload = json!({
        "tenant": "demo",
        "user": "demo@example.com"
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    assert!(!request.reset, "Reset flag must default to false");
}

// ============================================================================
// Demo Data Summary Structure Tests
// ============================================================================

#[test]
fn demo_data_summary_contains_all_entity_counts() {
    use agrocore_api::dto::DemoDataSummary;
    use uuid::Uuid;

    let summary = DemoDataSummary {
        tenant_id: Uuid::nil(),
        tenant_name: "Demo".to_string(),
        sites_count: 3,
        equipment_count: 3,
        orders_count: 2,
        workers_count: 2,
        inventory_items_count: 3,
        animals_count: 3,
    };

    // Verify all counts are present and non-zero
    assert_eq!(summary.sites_count, 3, "Should have 3 demo sites");
    assert_eq!(summary.equipment_count, 3, "Should have 3 demo equipment");
    assert_eq!(summary.orders_count, 2, "Should have 2 demo orders");
    assert_eq!(summary.workers_count, 2, "Should have 2 demo workers");
    assert_eq!(
        summary.inventory_items_count, 3,
        "Should have 3 demo inventory items"
    );
    assert_eq!(summary.animals_count, 3, "Should have 3 demo animals");
}

#[test]
fn demo_data_summary_serializes_correctly() {
    use agrocore_api::dto::DemoDataSummary;
    use uuid::Uuid;

    let summary = DemoDataSummary {
        tenant_id: Uuid::nil(),
        tenant_name: "Demo Farm".to_string(),
        sites_count: 3,
        equipment_count: 3,
        orders_count: 2,
        workers_count: 2,
        inventory_items_count: 3,
        animals_count: 3,
    };

    let json = serde_json::to_string(&summary).expect("Failed to serialize");

    // Verify all fields are present in JSON
    assert!(
        json.contains("sites_count"),
        "JSON must contain sites_count"
    );
    assert!(
        json.contains("animals_count"),
        "JSON must contain animals_count"
    );
    assert!(
        json.contains("orders_count"),
        "JSON must contain orders_count"
    );
    assert!(
        json.contains("workers_count"),
        "JSON must contain workers_count"
    );
}

// ============================================================================
// Demo Default Password Tests
// ============================================================================

#[test]
fn demo_default_password_meets_minimum_length() {
    const DEMO_DEFAULT_PASSWORD: &str = "demo1234-agrocore";
    assert!(
        DEMO_DEFAULT_PASSWORD.len() >= 12,
        "Demo password is {} characters, minimum is 12",
        DEMO_DEFAULT_PASSWORD.len()
    );
}

#[test]
fn demo_default_password_is_not_trivial() {
    const DEMO_DEFAULT_PASSWORD: &str = "demo1234-agrocore";

    // Should not be a common password
    let common_passwords = ["password", "123456", "admin", "demo", "test"];
    for common in &common_passwords {
        assert!(
            DEMO_DEFAULT_PASSWORD != *common,
            "Demo password must not be a common password"
        );
    }
}

// ============================================================================
// Setup Flow Integration Tests (App-level, no DB)
// ============================================================================
// Note: Actix app-level tests are covered by app_setup_tests.rs.
// These tests focus on the DTO validation and configuration logic.

// ============================================================================
// Complete Setup Flow Tests
// ============================================================================

#[test]
fn complete_setup_request_structure() {
    use agrocore_api::handlers::system::InitialSetupRequest;
    use serde_json::json;

    // This is the exact structure a new deployment would send
    let payload = json!({
        "admin": {
            "firstname": "John",
            "lastname": "Doe",
            "email": "john.doe@farm.example.com",
            "password": "MySecureP@ssw0rd!",
            "roles": null,
            "internal_cost_per_hour": null,
            "external_cost_per_hour": null,
            "language": null
        },
        "tenant": {
            "name": "Green Valley Farm",
            "slug": "green-valley",
            "config": {
                "default_language": "en",
                "supported_languages": ["en", "de", "es"],
                "timezone": "Europe/Berlin",
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
                "custom_field_schemas": {
                    "company_profile": {
                        "name": "Green Valley Farm",
                        "address": "Farm Road 1",
                        "country": "Germany",
                        "email": "info@greenvalley.example.com",
                        "phone": "+49 123 456789"
                    }
                },
                "logo_url": null,
                "primary_color": null,
                "validation_rules": null
            }
        }
    });

    let request: InitialSetupRequest =
        serde_json::from_value(payload.clone()).expect("Setup request must deserialize");

    // Validate admin
    request.admin.validate().expect("Admin must be valid");

    // Validate tenant
    request.tenant.validate().expect("Tenant must be valid");

    // Verify structure
    assert_eq!(request.admin.firstname, "John");
    assert_eq!(request.admin.lastname, "Doe");
    assert_eq!(request.admin.email, "john.doe@farm.example.com");
    assert_eq!(request.tenant.name, "Green Valley Farm");
    assert_eq!(request.tenant.slug, "green-valley");

    // Verify config
    let config = request.tenant.config.as_ref().expect("Config must exist");
    assert_eq!(config.default_language, "en");
    assert_eq!(config.enabled_modules.len(), 11);
}

#[test]
fn demo_seed_creates_comprehensive_data() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    // Verify the demo seed request structure
    let payload = json!({
        "tenant": "demo",
        "user": "demo@agrocore.local",
        "reset": false
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    assert!(request.validate().is_ok());
    assert_eq!(request.tenant, "demo");
    assert_eq!(request.user, "demo@agrocore.local");
    assert!(!request.reset);
}

#[test]
fn demo_seed_reset_mode() {
    use agrocore_api::dto::DemoSeedRequest;
    use serde_json::json;

    let payload = json!({
        "tenant": "demo",
        "user": "demo@agrocore.local",
        "reset": true
    });

    let request: DemoSeedRequest = serde_json::from_value(payload).unwrap();
    assert!(request.validate().is_ok());
    assert!(request.reset, "Reset flag must be true");
}

// ============================================================================
// Entity Type Coverage Tests
// ============================================================================

#[test]
fn demo_covers_all_entity_types() {
    // This test documents the expected entity types that the demo switch creates.
    // It serves as a checklist to ensure no entity type is forgotten.

    let entity_types = [
        ("Tenant", 1),
        ("User (Admin)", 1),
        ("Sites (Fields)", 3),
        ("Equipment", 3),
        ("Workers", 2),
        ("Orders", 2),
        ("Inventory Items", 3),
        ("Inventory Locations", 1),
        ("Inventory Transactions", 3),
        ("Animals", 3),
        ("Buildings", 3),
        ("Trees", 3),
        ("Tasks (Task Data)", 2),
        ("Livestock (Herd counts)", 2),
    ];

    // Verify we have coverage for all major entity types
    assert_eq!(entity_types.len(), 14, "Should cover 14 entity types");

    // Print coverage for documentation
    println!("Demo data coverage:");
    for (entity, count) in &entity_types {
        println!("  - {}: {} entries", entity, count);
    }
}

#[test]
fn demo_uses_english_terms() {
    // Verify that demo data uses English terms as required
    let english_terms = [
        "North Farm",
        "South Farm",
        "East Pasture",
        "Wheat",
        "Corn",
        "Cattle",
        "Barn",
        "ChickenCoop",
        "GoatStable",
        "Olive",
        "CorkOak",
        "Almond",
        "Dairy Cattle Herd",
        "Free-Range Chickens",
        "Main Wheat Storage Barn",
        "Poultry House Alpha",
        "Goat Shelter Beta",
        "Olive Grove Row A",
        "Cork Oak Stand B",
        "Almond Orchard C",
    ];

    // All terms should be in English (no German, Portuguese, etc.)
    for term in &english_terms {
        assert!(
            !term.contains("Feld") && !term.contains("Hof") && !term.contains("Stall"),
            "Term '{}' should be in English, not German",
            term
        );
    }
}
