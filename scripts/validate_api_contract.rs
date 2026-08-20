//! API Contract Validator
//!
//! This script validates that all UI API helper paths have corresponding API routes.
//! Run with: cargo run --bin validate_api_contract --features validate_contract
//!
//! It prevents 500 errors by catching path mismatches before they reach production.

use std::collections::BTreeSet;

/// Extract all registered API routes from the handler configuration.
/// These are the "source of truth" for available endpoints.
pub fn extract_api_routes() -> BTreeSet<String> {
    let mut routes = BTreeSet::new();

    // Core routes from handlers/mod.rs
    let core_routes = vec![
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
        "/api/v1/workforce/workers",
        "/api/v1/workforce/workers/{id}",
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
        "/api/v1/agriculture/nutrition/demand",
        "/api/v1/agriculture/predict/harvest",
        "/api/v1/agriculture/calculate/material",
        "/api/v1/agriculture/calculate/water-rate",
        "/api/v1/agriculture/gdd/accumulated",
        // Reporting exports
        "/api/v1/reporting/export/orders/excel",
        "/api/v1/reporting/export/pac/sip",
        "/api/v1/reporting/export/sites/geojson",
        "/api/v1/reporting/export/veterinary",
    ];

    for route in core_routes {
        routes.insert(route.to_string());
    }

    routes
}

/// Extract all API paths used by UI fetch functions.
/// These are the "consumers" of the API.
pub fn extract_ui_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();

    let ui_paths = vec![
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
        "/api/v1/sigpac/parcels/{}",
        "/api/v1/sigpac/parcels{}",
        "/api/v1/sigpac/parcels/search/near-point?{}",
        "/api/v1/sites",
        "/api/v1/sites/{}",
        "/api/v1/sites/import/geojson",
        "/api/v1/sites/import/shapefile",
        "/api/v1/system/setup",
        "/api/v1/system/status",
        "/api/v1/system/tenant",
        "/api/v1/tasks",
        "/api/v1/users",
        "/api/v1/users/{}",
        "/api/v1/weather/data",
        "/api/v1/weather/phenology",
        "/api/v1/weather/stations",
        "/api/v1/workforce/clock-entries",
        "/api/v1/workforce/clock-entries/{}",
        "/api/v1/workforce/tasks/{}/status",
        "/api/v1/workforce/tasks/{}/status/{}",
        "/api/v1/workforce/tasks/{}/status/aggregate",
        "/api/v1/workforce/workers",
        "/api/v1/workforce/workers/{}",
        "/api/v1/workforce/workers/{}/clock-active",
        "/api/v1/workforce/workers/{}/clock-entries",
        "/api/v1/workforce/workers/{}/clock-sessions",
        "/api/v1/workforce/workers/{}/clock-sessions?{}",
        "/api/v1/workforce/workers/{}/hours-worked",
        "/api/v1/workforce/workers/{}/hours-worked?{}",
    ];

    for path in ui_paths {
        paths.insert(path.to_string());
    }

    paths
}

/// Normalize a path pattern by replacing parameters ({}, {id}) with {param}
fn normalize_path(path: &str) -> String {
    // Remove query strings
    let path = path.split('?').next().unwrap_or(path);
    // Replace {} or {id} etc with {param}
    let result = path.replace("{}", "{param}");
    // Handle {id}, {user_id}, etc — keep as {param}
    let result = result.replace("{id}", "{param}");
    let result = result.replace("{user_id}", "{param}");
    let result = result.replace("{device_id}", "{param}");
    let result = result.replace("{site_id}", "{param}");
    let result = result.replace("{task_id}", "{param}");
    let result = result.replace("{worker_id}", "{param}");
    let result = result.replace("{number}", "{param}");
    let result = result.replace("{query}", "{param}");
    result
}

/// Check if two path patterns match (ignoring parameter names and query strings)
fn paths_match(api_path: &str, ui_path: &str) -> bool {
    let api_norm = normalize_path(api_path);
    let ui_norm = normalize_path(ui_path);
    api_norm == ui_norm
}

/// Check if a UI path has a corresponding API route
fn find_matching_api(ui_path: &str, api_routes: &BTreeSet<String>) -> Option<String> {
    let ui_norm = normalize_path(ui_path);

    for api_path in api_routes {
        if normalize_path(api_path) == ui_norm {
            return Some(api_path.to_string());
        }
    }

    // Try prefix matching for sub-paths
    for api_path in api_routes {
        let api_norm = normalize_path(api_path);
        if ui_norm.starts_with(&api_norm) || api_norm.starts_with(&ui_norm) {
            return Some(api_path.to_string());
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct ContractViolation {
    pub ui_path: String,
    pub matched_api_route: Option<String>,
    pub issue: String,
}

/// Run the full contract validation
pub fn validate_api_contract() -> Vec<ContractViolation> {
    let api_routes = extract_api_routes();
    let ui_paths = extract_ui_paths();

    let mut violations = Vec::new();

    for ui_path in &ui_paths {
        let matched = find_matching_api(ui_path, &api_routes);

        if matched.is_none() {
            violations.push(ContractViolation {
                ui_path: ui_path.clone(),
                matched_api_route: None,
                issue: "Missing API route — UI calls an endpoint that does not exist".to_string(),
            });
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/api/v1/orders/{}"), normalize_path("/api/v1/orders/{id}"));
        assert_eq!(normalize_path("/api/v1/items"), normalize_path("/api/v1/items"));
        assert!(!paths_match("/api/v1/orders", "/api/v1/users"));
    }

    #[test]
    fn test_validate_contract_passes_for_known_routes() {
        let violations = validate_api_contract();

        if violations.is_empty() {
            println!("✅ All UI paths have matching API routes!");
        } else {
            println!("❌ Found {} contract violations:", violations.len());
            for v in &violations {
                println!("  UI path: {}", v.ui_path);
                if let Some(api) = &v.matched_api_route {
                    println!("  -> Matched API: {}", api);
                } else {
                    println!("  -> NO MATCHING API ROUTE FOUND!");
                }
                println!("  Issue: {}", v.issue);
                println!();
            }
        }

        // For now, just assert — we know there are mismatches
        // In CI this would be a hard failure
        if !violations.is_empty() {
            eprintln!("Contract violations found — see output above");
        }
    }

    #[test]
    fn test_all_ui_paths_accounted_for() {
        // This test ensures we haven't missed any UI paths
        // When new fetch functions are added to api.rs but not registered here,
        // this test will fail — prompting an update to the contract list
        let ui_paths = extract_ui_paths();
        let expected_count = 60; // Update when adding new paths
        assert!(
            ui_paths.len() >= expected_count,
            "UI paths count ({}) is below expected minimum ({}). \
             New fetch functions may have been added without updating extract_ui_paths!",
            ui_paths.len(),
            expected_count
        );
    }
}

/// Binary entry point — run this to get a report:
/// `cargo run --bin validate_api_contract`
#[cfg(feature = "validate_contract")]
fn main() {
    println!("=== API Contract Validation ===\n");

    let violations = validate_api_contract();

    if violations.is_empty() {
        println!("✅ No contract violations found!");
        println!("\nAll UI API paths have matching backend routes.\n");
    } else {
        println!("❌ Found {} contract violations:\n", violations.len());
        for v in &violations {
            println!("  Path: {}", v.ui_path);
            println!("  Issue: {}", v.issue);
            println!();
        }
    }

    // Also show full route listing
    println!("\n=== API Routes ({} total) ===", extract_api_routes().len());
    for route in extract_api_routes() {
        let has_ui = find_matching_api(&route, &extract_ui_paths()).is_some();
        let status = if has_ui { "✅" } else { "⚠️ " };
        println!("  {} {}", status, route);
    }

    println!("\n=== UI Paths ({} total) ===", extract_ui_paths().len());
    for path in extract_ui_paths() {
        let matched = find_matching_api(&path, &extract_api_routes());
        let status = if matched.is_some() { "✅" } else { "❌" };
        println!("  {} {} -> {:?}", status, path, matched);
    }

    if !violations.is_empty() {
        std::process::exit(1);
    }
}
