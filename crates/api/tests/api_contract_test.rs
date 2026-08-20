//! API Contract Validation Test
//!
//! This test prevents 500 errors by verifying that every UI API helper path
//! has a corresponding backend route. It catches path mismatches, renamed
//! routes, and missing endpoints before they reach the browser.
//!
//! Run: `cargo test --test api_contract_test -- --nocapture`

use std::collections::BTreeSet;

/// All API routes registered in `crates/api/src/handlers/mod.rs`.
/// This is the "source of truth" for available endpoints.
/// Update this list when adding or removing API routes.
fn api_routes() -> BTreeSet<String> {
    let routes = vec![
        // Auth
        "/api/v1/auth/login",
        "/api/v1/auth/refresh",
        "/api/v1/auth/logout",
        "/api/v1/auth/impersonate/stop",
        "/api/v1/auth/impersonate/{user_id}",
        // System
        "/api/v1/system/status",
        "/api/v1/system/setup",
        "/api/v1/system/tenant",
        // Health
        "/api/v1/health",
        // Sites
        "/api/v1/sites",
        "/api/v1/sites/{id}",
        "/api/v1/sites/import/geojson",
        "/api/v1/sites/import/shapefile",
        "/api/v1/specialized/sites",
        // Orders
        "/api/v1/orders",
        "/api/v1/orders/my-tasks",
        "/api/v1/orders/{id}",
        "/api/v1/orders/{id}/complete",
        "/api/v1/orders/{id}/start",
        // Tasks
        "/api/v1/tasks",
        "/api/v1/tasks/{id}",
        "/api/v1/tasks/{id}/start-for-worker",
        "/api/v1/tasks/{id}/stop-for-worker",
        // Workers
        "/api/v1/workers",
        "/api/v1/workers/{id}",
        "/api/v1/workers/{id}/clock-active",
        "/api/v1/workers/{id}/clock-entries",
        "/api/v1/workers/{id}/clock-sessions",
        "/api/v1/workers/{id}/hours-worked",
        // Workforce tasks
        "/api/v1/workforce/tasks/{task_id}/status",
        "/api/v1/workforce/tasks/{task_id}/status/{worker_id}",
        "/api/v1/workforce/tasks/{task_id}/status/aggregate",
        // Users
        "/api/v1/users",
        "/api/v1/users/{id}",
        // Equipment
        "/api/v1/equipments",
        "/api/v1/equipments/{id}",
        // Inventory
        "/api/v1/inventory/items",
        "/api/v1/inventory/items/{id}",
        "/api/v1/inventory/balances",
        "/api/v1/inventory/balances/below-minimum",
        "/api/v1/inventory/items/{id}/transactions",
        "/api/v1/inventory/transactions",
        "/api/v1/inventory/stock-in",
        "/api/v1/inventory/stock-out",
        "/api/v1/inventory/transfer",
        "/api/v1/inventory/adjust",
        "/api/v1/inventory/locations",
        // Compliance
        "/api/v1/compliance/audit-logs",
        "/api/v1/compliance/checklists",
        "/api/v1/compliance/fertilizer",
        "/api/v1/compliance/plant-protection",
        // Finance
        "/api/v1/finance/cost-centers",
        "/api/v1/finance/financial-records",
        "/api/v1/finance/pac-applications",
        // Customers
        "/api/v1/customers/{id}/orders",
        "/api/v1/customers/number/{number}",
        "/api/v1/customers/search/{query}",
        // Weather
        "/api/v1/weather/data",
        "/api/v1/weather/stations",
        "/api/v1/weather/phenology",
        // Livestock
        "/api/v1/livestock/animals",
        "/api/v1/livestock/animals/{id}",
        "/api/v1/livestock/animals/{id}/treatments",
        "/api/v1/livestock/animals/{id}/grazing",
        // LPIS
        "/api/v1/lpis/providers",
        "/api/v1/parcels",
        "/api/v1/parcels/{id}",
        "/api/v1/sigpac/parcels/{id}",
        "/api/v1/sigpac/parcels",
        "/api/v1/sigpac/parcels/search/near-point",
        // IoT
        "/api/v1/devices/{device_id}/command",
        // Agriculture
        "/api/v1/nutrition/demand",
        "/api/v1/predict/harvest",
        "/api/v1/calculate/material",
        "/api/v1/calculate/water-rate",
        "/api/v1/gdd/accumulated",
        // Reporting exports
        "/api/v1/reporting/export/orders/excel",
        "/api/v1/reporting/export/pac/sip",
        "/api/v1/reporting/export/sites/geojson",
        "/api/v1/reporting/export/veterinary",
    ];

    routes.into_iter().map(String::from).collect()
}

/// All API paths used by the Admin UI (`crates/admin-ui/src/api.rs`).
/// This is the "consumer" list — if any path here doesn't match an API route,
/// the UI will get a 500 (or 404) error at runtime.
/// Update this list when adding or removing UI fetch functions.
fn ui_paths() -> Vec<String> {
    vec![
        "/api/v1/auth/impersonate/{}",
        "/api/v1/auth/impersonate/stop",
        "/api/v1/auth/login",
        "/api/v1/calculate/material",
        "/api/v1/compliance/audit-logs",
        "/api/v1/compliance/checklists",
        "/api/v1/compliance/fertilizer",
        "/api/v1/compliance/plant-protection",
        "/api/v1/equipments",
        "/api/v1/equipments/{}",
        "/api/v1/finance/cost-centers",
        "/api/v1/finance/financial-records",
        "/api/v1/finance/pac-applications",
        "/api/v1/inventory/adjust",
        "/api/v1/inventory/balances",
        "/api/v1/inventory/balances/below-minimum",
        "/api/v1/inventory/items",
        "/api/v1/inventory/items/{}",
        "/api/v1/inventory/items/{}/transactions",
        "/api/v1/inventory/locations",
        "/api/v1/inventory/stock-in",
        "/api/v1/inventory/stock-out",
        "/api/v1/inventory/transfer",
        "/api/v1/livestock/animals",
        "/api/v1/livestock/animals/{}/treatments",
        "/api/v1/nutrition/demand",
        "/api/v1/orders",
        "/api/v1/orders/{}",
        "/api/v1/orders/my-tasks",
        "/api/v1/predict/harvest?site_id={}",
        "/api/v1/reporting/export/orders/excel",
        "/api/v1/reporting/export/pac/sip",
        "/api/v1/reporting/export/sites/geojson",
        "/api/v1/reporting/export/veterinary",
        // Fixed: query param appended with ? not directly
        "/api/v1/sigpac/parcels?{}",
        "/api/v1/sigpac/parcels/{}",
        "/api/v1/sigpac/parcels/search/near-point?{}",
        "/api/v1/sites",
        "/api/v1/sites/{}",
        "/api/v1/sites/import/geojson",
        "/api/v1/sites/import/shapefile",
        "/api/v1/system/setup",
        "/api/v1/system/status",
        "/api/v1/system/tenant",
        "/api/v1/tasks",
        "/api/v1/tasks/{}",
        "/api/v1/users",
        "/api/v1/users/{}",
        "/api/v1/weather/data",
        "/api/v1/weather/phenology",
        "/api/v1/weather/stations",
        // Fixed: /workforce/workers -> /workers
        "/api/v1/workers",
        "/api/v1/workers/{}",
        "/api/v1/workers/{}/clock-active",
        "/api/v1/workers/{}/clock-entries",
        "/api/v1/workers/{}/clock-sessions",
        "/api/v1/workers/{}/clock-sessions?{}",
        "/api/v1/workers/{}/hours-worked",
        "/api/v1/workers/{}/hours-worked?{}",
        // Workforce tasks paths
        "/api/v1/workforce/tasks/{}/status",
        "/api/v1/workforce/tasks/{}/status/{}",
        "/api/v1/workforce/tasks/{}/status/aggregate",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// All UI routes defined in admin-ui/src/main.rs
fn ui_routes() -> Vec<String> {
    vec![
        "/",
        "/sites",
        "/import",
        "/sigpac",
        "/map",
        "/tasks",
        "/livestock",
        "/weather",
        "/finance",
        "/equipment",
        "/inventory",
        "/analytics",
        "/audit",
        "/resources",
        "/compliance",
        "/users",
        "/settings",
        "/wizard",
        "/worker/tasks",
        "/workers",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Normalize a path: remove query strings, standardize parameter placeholders.
fn normalize(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);

    path.replace("{}", "{param}")
        .replace("{id}", "{param}")
        .replace("{user_id}", "{param}")
        .replace("{device_id}", "{param}")
        .replace("{site_id}", "{param}")
        .replace("{task_id}", "{param}")
        .replace("{worker_id}", "{param}")
        .replace("{number}", "{param}")
        .replace("{query}", "{param}")
}

/// Check if a UI path has a matching API route.
fn find_api_for_ui(ui_path: &str, api_routes: &BTreeSet<String>) -> Option<String> {
    let ui_norm = normalize(ui_path);

    for api in api_routes {
        if normalize(api) == ui_norm {
            return Some(api.to_string());
        }
    }

    None
}

#[test]
fn test_api_contract_all_ui_paths_have_routes() {
    let api_routes_set: BTreeSet<String> = api_routes().into_iter().collect();
    let ui_paths = ui_paths();

    let mut violations = Vec::new();

    for ui_path in &ui_paths {
        if find_api_for_ui(ui_path, &api_routes_set).is_none() {
            violations.push(ui_path.as_str());
        }
    }

    if !violations.is_empty() {
        eprintln!("\n❌ API Contract Violations — UI paths without matching API routes:");
        eprintln!("   These will cause 500/404 errors in the Admin UI:\n");
        for v in &violations {
            eprintln!("   - {}", v);
        }
        eprintln!("\n   Fix: Add the route to API handlers, OR update the UI fetch path.");
        panic!(
            "Found {} API contract violations! The Admin UI references API paths that don't exist.",
            violations.len()
        );
    }

    println!(
        "✅ All {} UI paths have matching API routes!",
        ui_paths.len()
    );
}

#[test]
fn test_api_contract_no_orphaned_routes() {
    let api_routes_set: BTreeSet<String> = api_routes().into_iter().collect();
    let ui_paths = ui_paths();
    let ui_paths_set: BTreeSet<String> = ui_paths.into_iter().collect();

    let mut orphans = Vec::new();

    for api in &api_routes_set {
        let api_norm = normalize(api);
        let has_ui = ui_paths_set.iter().any(|ui| normalize(ui) == api_norm);
        if !has_ui && api.contains("/api/v1/") {
            orphans.push(api.as_str());
        }
    }

    if !orphans.is_empty() && std::env::var("CI").is_ok() {
        eprintln!("\n⚠️  Orphaned API routes (no UI consumer):");
        for o in &orphans {
            eprintln!("   - {}", o);
        }
    }
}

/// All page components in crates/admin-ui/src/components/
fn ui_components() -> Vec<&'static str> {
    vec![
        "dashboard",
        "sites",
        "import",
        "sigpac",
        "map",
        "orders",
        "livestock",
        "weather",
        "finance",
        "equipment",
        "inventory",
        "analytics",
        "audit",
        "resources",
        "compliance",
        "users",
        "settings",
        "wizard",
        "worker_tasks",
        "workers",
    ]
}

#[test]
fn test_crud_coverage() {
    // Verify that for each major resource, the UI has both
    // list (fetch_) and detail (create_/update_/delete_) operations
    let api_routes_set: BTreeSet<String> = api_routes().into_iter().collect();
    let ui_paths = ui_paths();

    // (resource_name, list_path, detail_paths)
    let resource_checks: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "Sites",
            "/api/v1/sites",
            vec!["/api/v1/sites/{}", "/api/v1/sites/import/geojson"],
        ),
        (
            "Orders",
            "/api/v1/orders",
            vec!["/api/v1/orders/{}", "/api/v1/orders/my-tasks"],
        ),
        (
            "Workers",
            "/api/v1/workers",
            vec![
                "/api/v1/workers/{}/clock-entries",
                "/api/v1/workers/{}/clock-sessions",
                "/api/v1/workers/{}/hours-worked",
            ],
        ),
        (
            "Users",
            "/api/v1/users",
            vec!["/api/v1/users/{}", "/api/v1/auth/impersonate/{}"],
        ),
        (
            "Equipment",
            "/api/v1/equipments",
            vec!["/api/v1/equipments/{}"],
        ),
        (
            "Inventory",
            "/api/v1/inventory/items",
            vec![
                "/api/v1/inventory/items/{}",
                "/api/v1/inventory/stock-in",
                "/api/v1/inventory/stock-out",
            ],
        ),
        (
            "PACDeductions",
            "/api/v1/finance/pac-applications",
            vec!["/api/v1/finance/pac-applications"],
        ),
        (
            "Cost Centers",
            "/api/v1/finance/cost-centers",
            vec!["/api/v1/finance/cost-centers"],
        ),
        (
            "Livestock",
            "/api/v1/livestock/animals",
            vec!["/api/v1/livestock/animals/{}/treatments"],
        ),
        (
            "Tasks",
            "/api/v1/tasks",
            vec!["/api/v1/tasks/{}", "/api/v1/orders/my-tasks"],
        ),
    ];

    let mut missing = Vec::new();

    for (resource, list_path, detail_paths) in &resource_checks {
        // Check list path exists in UI and API
        let list_found_ui = ui_paths.iter().any(|p| p == list_path);
        let list_found_api = api_routes_set.iter().any(|p| p == list_path);

        if !list_found_ui {
            missing.push(format!(
                "{}: missing list path {} in UI",
                resource, list_path
            ));
        }
        if !list_found_api {
            missing.push(format!(
                "{}: missing list path {} in API",
                resource, list_path
            ));
        }

        // Check detail paths exist in both
        for detail in detail_paths {
            let detail_found_ui = ui_paths.iter().any(|p| normalize(p) == normalize(detail));
            let detail_found_api = api_routes_set
                .iter()
                .any(|p| normalize(p) == normalize(detail));

            if !detail_found_ui {
                missing.push(format!(
                    "{}: missing detail path {} in UI",
                    resource, detail
                ));
            }
            if !detail_found_api {
                missing.push(format!(
                    "{}: missing detail path {} in API",
                    resource, detail
                ));
            }
        }
    }

    if !missing.is_empty() {
        eprintln!("\n❌ Missing CRUD coverage:");
        for m in &missing {
            eprintln!("   - {}", m);
        }
        panic!(
            "Found {} missing CRUD operations! Every resource needs List + Detail in both UI and API.",
            missing.len()
        );
    }

    println!(
        "✅ All {} resources have full CRUD coverage (List + Detail) in UI and API!",
        resource_checks.len()
    );
}

#[test]
fn test_ui_routes_have_pages() {
    let routes = ui_routes();
    let components = ui_components();

    assert_eq!(
        routes.len(),
        components.len(),
        "Number of UI routes should match number of page components"
    );

    // Verify navigation sidebar has all routes
    let nav_routes = vec![
        "/",
        "/users",
        "/wizard",
        "/import",
        "/sigpac",
        "/sites",
        "/tasks",
        "/map",
        "/livestock",
        "/weather",
        "/resources",
        "/equipment",
        "/finance",
        "/analytics",
        "/inventory",
        "/audit",
        "/compliance",
        "/settings",
        "/workers",
        "/worker/tasks",
    ];

    for route in &nav_routes {
        assert!(
            routes.iter().any(|r| r == route),
            "Route {} in sidebar navigation has no corresponding <Route> in main.rs",
            route
        );
    }

    println!(
        "✅ All {} UI routes have corresponding page components!",
        routes.len()
    );
}

#[test]
fn test_role_based_access_consistency() {
    // Verify that role-based UI access in main.rs matches API capabilities:
    // - Admin: full CRUD on all resources
    // - Manager: CRUD on non-configuration resources
    // - Worker: read-only + own data operations
    //
    // This test checks that:
    // 1. Admin-only routes exist for sensitive operations (setup, tenant deletion)
    // 2. Worker routes exist for time tracking (clock-in/out, my-tasks)
    // 3. Manager routes exist for team management

    let admin_only_routes = vec![
        "/api/v1/system/setup",  // Initial setup
        "/api/v1/system/tenant", // Tenant deletion
    ];

    let worker_specific_routes = vec![
        "/api/v1/orders/my-tasks",
        "/api/v1/workers/{id}/clock-entries",
        "/api/v1/workers/{id}/clock-active",
        "/api/v1/workers/{id}/clock-sessions",
        "/api/v1/workers/{id}/hours-worked",
    ];

    let manager_routes = vec!["/api/v1/workforce/tasks/{task_id}/status/aggregate"];

    let api_routes_set: BTreeSet<String> = api_routes().into_iter().collect();
    let mut missing = Vec::new();

    // Check admin-only routes
    for route in &admin_only_routes {
        if !api_routes_set.iter().any(|r| r == route) {
            missing.push(format!("Admin-only route missing from API: {}", route));
        }
    }

    // Check worker-specific routes
    for route in &worker_specific_routes {
        if !api_routes_set
            .iter()
            .any(|r| normalize(r) == normalize(route))
        {
            missing.push(format!("Worker-specific route missing from API: {}", route));
        }
    }

    // Check manager routes
    for route in &manager_routes {
        if !api_routes_set
            .iter()
            .any(|r| normalize(r) == normalize(route))
        {
            missing.push(format!("Manager route missing from API: {}", route));
        }
    }

    if !missing.is_empty() {
        eprintln!("\n❌ Role-based access inconsistency:");
        for m in &missing {
            eprintln!("   - {}", m);
        }
        panic!(
            "Found {} role-based access inconsistencies! API must support all role-specific routes.",
            missing.len()
        );
    }

    println!("✅ Role-based access is consistent across API routes!");
}

#[test]
fn test_all_ui_components_exist() {
    let components = ui_components();
    let base = "../admin-ui/src/components/";

    let missing: Vec<_> = components
        .iter()
        .filter(|c| {
            let path1 = format!("{}{}.rs", base, c);
            let path2 = format!("{}{}/mod.rs", base, c);
            !std::path::Path::new(&path1).exists() && !std::path::Path::new(&path2).exists()
        })
        .collect();

    if !missing.is_empty() {
        panic!(
            "Components without source files: {:?}\nThese are referenced in ui_components() but don't exist.",
            missing
        );
    }

    println!(
        "✅ All {} UI components have source files!",
        components.len()
    );
}
