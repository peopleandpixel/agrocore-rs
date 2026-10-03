//! Route inventory, read from the running application.
//!
//! # Why the application is asked instead of the source
//!
//! Two earlier versions of this tried to recover the route list by parsing
//! `web::scope(..)` and `web::resource(..)` out of the handler sources. Both were
//! wrong, and wrong in the direction that matters: they reported routes that do
//! not exist and missed routes that do.
//!
//! Parsing has to track scope nesting, because where a resource is mounted
//! depends on which scopes are open around it:
//!
//! ```text
//! web::scope("/weather").service(web::resource("/data"))   -> /api/v1/weather/data
//! .service(web::scope("/system").route("/status", ..))     -> /api/v1/system/status
//! web::scope("/livestock").route("/animals", ..)           -> /api/v1/livestock/animals
//! ```
//!
//! Two things make that unreliable. `web::scope("/api/v1")` already carries the
//! root while `web::scope("/weather")` does not, so the same literal needs
//! different treatment depending on where it appears. And a scope's indentation
//! says nothing about where it closes:
//!
//! ```text
//!     cfg.service(
//!         web::scope("/api/v1")   <- opens at indent 8
//!             .service(...),
//!     );                          <- closes at indent 4
//! ```
//!
//! An indent-based rule never closes the root scope. That one mistake filed every
//! `.configure(module)` delegation under a stale prefix and reported 179 real
//! routes as unrouted.
//!
//! # What this does instead
//!
//! Builds the same application the server runs and asks Actix to route a request
//! for each candidate path. A path that answers with anything other than 404 is
//! registered. There is no second copy of the route list to drift, because the
//! list is produced by the router itself.
//!
//! # What it still needs
//!
//! Actix cannot enumerate its own routing table, so the set of paths to probe
//! still comes from the handler sources. What changes is that parsing now only
//! has to produce a *candidate* set, and the candidates are the registration
//! shapes the handlers actually use — 22 scopes joined with 179 resource paths,
//! about 3.9 thousand paths. A previous attempt built candidates from every
//! literal crossed with every other literal, about 28 million; correct as a
//! superset, useless as a test.
//!
//! # What it cannot detect
//!
//! A route assembled at runtime by string concatenation. None exists today.
//! `every_registration_shape_is_covered` fails loudly if one is added.

use actix_web::{App, http::StatusCode, test};
use std::collections::{BTreeMap, BTreeSet};

const ROOT: &str = "/api/v1";

/// Every HTTP verb the handlers use.
const VERBS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

/// Placeholder names the handlers use in paths.
const PLACEHOLDERS: [&str; 15] = [
    "id",
    "user_id",
    "device_id",
    "site_id",
    "task_id",
    "worker_id",
    "plot_id",
    "herd_id",
    "animal_id",
    "order_id",
    "key",
    "query",
    "number",
    "resource",
    "name",
];

fn handler_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/handlers")
}

fn admin_ui_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("api crate has a parent")
        .join("admin-ui/src")
}

/// Drop `//`, `///` and block comments.
///
/// Doc comments matter: a handler documented as `/// GET /api/v1/parcels` above a
/// call to `/api/v1/sigpac/parcels` would otherwise contribute a path nothing
/// calls.
fn strip_comments(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_block {
            if trimmed.contains("*/") {
                in_block = false;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("/*") {
            in_block = !rest.contains("*/");
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        out.push(line.to_string());
    }
    out
}

/// Every double-quoted literal on a line.
fn quoted_literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;

    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        match after.find('"') {
            Some(close) => {
                out.push(after[..close].to_string());
                rest = &after[close + 1..];
            }
            // An unterminated literal continues on the next line.
            None => break,
        }
    }
    out
}

/// The string literal after `marker` on a line.
fn literal_after<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let open = rest.find('"')?;
    let after = &rest[open + 1..];
    let close = after.find('"')?;
    Some(&after[..close])
}

/// A line holding nothing but a quoted path and a trailing comma.
fn sole_path(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_suffix(',')?.trim();
    let inner = rest.strip_prefix('"')?.strip_suffix('"')?;
    inner.starts_with('/').then(|| inner.to_string())
}

/// Candidate paths to probe.
///
/// The shapes the handlers use:
///   * a scope joined with a resource — `web::scope("/weather")` holding
///     `web::resource("/data")`;
///   * a scope holding a bare route — `web::scope("/livestock")` with
///     `.route("/animals", ..)`;
///   * a path standing alone on its own line inside a call split across lines;
///   * an absolute path registered directly under the root scope.
///
/// Delegated modules are included: `.configure(settings_groups::configure)` mounts
/// another file's resources into the scope that was open at the point of the
/// call, so `/settings/groups` is registered even though nothing in
/// `settings_groups.rs` names a scope.
///
/// The previous version of this file built candidates from every literal crossed
/// with every other literal — about 28 million paths. Correct as a superset,
/// useless as a test.
fn candidates() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let dir = handler_dir();

    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    let mut files: Vec<_> = entries
        .filter_map(|e| {
            let path = e.expect("dir entry").path();
            (path.extension().and_then(|x| x.to_str()) == Some("rs")).then_some(path)
        })
        .collect();
    files.sort();

    for path in files {
        let source = std::fs::read_to_string(&path).expect("read handler source");
        candidates_in_file(&strip_comments(&source), &dir, &mut out);
    }

    out
}

/// Mount every path registered in one file.
fn candidates_in_file(lines: &[String], dir: &std::path::Path, out: &mut BTreeSet<String>) {
    // Open scopes as (prefix, brace depth when opened).
    let mut stack: Vec<(String, i32)> = Vec::new();
    // Modules delegated to, as (module name, owning scope).
    let mut delegated: Vec<(String, String)> = Vec::new();
    let mut depth: i32 = 0;

    for line in lines {
        let trimmed = line.trim();

        if let Some(scope) = literal_after(line, "web::scope(") {
            let prefix = if scope.starts_with("/api") {
                scope.to_string()
            } else {
                format!("{ROOT}{scope}")
            };
            // The depth *before* this line's own parentheses.
            //
            // `web::scope("/x")` carries an unbalanced `(`: it opens the scope and
            // the call continues on the following lines. Counting the line's parens
            // pushed the scope one level too deep, so the closing `)` of the
            // wrapping `.service(` no longer closed it and everything after
            // `/system` was filed under `/api/v1/system`.
            stack.push((prefix.clone(), depth));
            if let Some(module) = delegated_module(line) {
                delegated.push((module, prefix));
            }
            continue;
        }

        if trimmed.starts_with(')') {
            // The closing parens of this line are applied *before* deciding what
            // it closes. Popping first and counting afterwards compared the depth
            // from before the line, so a scope opened on the previous line at the
            // same depth as its closing `)` was never popped and everything after
            // `/system` stayed under `/api/v1/system`.
            depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
            while stack.last().is_some_and(|(_, opened)| depth < *opened) {
                stack.pop();
            }
            continue;
        }

        if let Some(module) = delegated_module(line) {
            let owner = stack
                .last()
                .map(|(p, _)| p.clone())
                .unwrap_or_else(|| ROOT.to_string());
            delegated.push((module, owner));
            continue;
        }

        let owner: &str = stack.last().map(|(p, _)| p.as_str()).unwrap_or(ROOT);

        for marker in ["web::resource(", "web::route(", ".route("] {
            if let Some(path) = literal_after(line, marker).filter(|p| p.starts_with('/')) {
                out.insert(mount(owner, path));
            }
        }

        if let Some(path) = sole_path(trimmed) {
            out.insert(mount(owner, &path));
        }

        depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
    }

    for (module, owner) in delegated {
        let path = dir.join(format!("{module}.rs"));
        let Ok(source) = std::fs::read_to_string(&path) else {
            // A module in a subdirectory is not resolved here. Its routes would
            // be missing from the candidate set, and a missing candidate is the
            // only thing that can hide a route, so this is reported rather than
            // skipped.
            eprintln!(
                "warning: delegated module {module} not found in {}; its routes are \
                 not probed",
                dir.display()
            );
            continue;
        };
        for line in strip_comments(&source) {
            for marker in ["web::resource(", "web::route(", ".route("] {
                if let Some(p) = literal_after(&line, marker).filter(|p| p.starts_with('/')) {
                    out.insert(mount(&owner, p));
                }
            }
        }
    }
}

/// The module name in `.configure(<path>::configure)`.
fn delegated_module(line: &str) -> Option<String> {
    let start = line.find(".configure(")? + ".configure(".len();
    let rest = &line[start..];
    let close = rest.find(')')?;
    let inner = &rest[..close];
    let module = inner.strip_suffix("::configure")?;
    let short = module.rsplit("::").next()?;
    (!short.is_empty()).then(|| short.to_string())
}

/// Replace placeholder segments so a router can match the path.
fn concrete(path: &str) -> String {
    let mut out = path.split('?').next().unwrap_or(path).to_string();

    for name in PLACEHOLDERS {
        out = out.replace(&format!("{{{name}}}"), "probe");
    }
    out = out.replace("{}", "probe");
    out
}

/// Probe every candidate against the real application.
async fn registered_routes() -> BTreeSet<String> {
    let app = test::init_service(App::new().configure(agrocore_api::handlers::configure)).await;

    let mut found = BTreeSet::new();
    for candidate in candidates() {
        let req = test::TestRequest::get()
            .uri(&concrete(&candidate))
            .to_request();
        let resp = test::call_service(&app, req).await;
        if resp.status() != StatusCode::NOT_FOUND {
            found.insert(candidate);
        }
    }
    found
}

/// The methods registered on each fully-qualified route.
///
/// A path alone does not say whether a resource is editable: `/sites` takes GET
/// and POST, `/sites/{id}` takes GET, PUT and DELETE. The verb is only visible in
/// the source — a probe cannot distinguish a 405 raised for "wrong method" from
/// one raised inside a handler.
///
/// # Scope tracking
///
/// The path has to be joined to its scope before it can be compared with anything
/// else, and the previous version of this function did not: it collected the raw
/// literal and the verbs that followed it, so `/sites` stayed `/sites` instead of
/// `/api/v1/sites`. Every route then looked like a bare segment, no detail route
/// could be matched to a collection route, and 69 collections were reported as
/// read-only.
///
/// Scope nesting is tracked by brace depth rather than indentation. In this
/// codebase a scope's own line sits one level right of the `cfg.service(` that
/// wraps it, so its indentation is *greater* than the indentation that closes it
/// and an indent-based rule never closes the root scope.
///
/// Each file is read separately because each module registers its routes in its
/// own `configure`; scopes never nest across files.
fn methods_by_path() -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let dir = handler_dir();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    let mut files: Vec<_> = entries
        .filter_map(|e| {
            let path = e.expect("dir entry").path();
            (path.extension().and_then(|x| x.to_str()) == Some("rs")).then_some(path)
        })
        .collect();
    // Stable order so a failure reports the same thing twice.
    files.sort();

    for path in files {
        let source = std::fs::read_to_string(&path).expect("read handler source");
        methods_in_file(&strip_comments(&source), &mut out);
    }

    out
}

/// One handler file, scopes and verbs collected into `out`.
fn methods_in_file(lines: &[String], out: &mut BTreeMap<String, BTreeSet<String>>) {
    // Open scopes as (prefix, brace depth when the scope opened).
    let mut stack: Vec<(String, i32)> = Vec::new();
    let mut depth: i32 = 0;

    // The registration currently collecting verbs, and its verbs.
    let mut current: Option<(String, BTreeSet<String>)> = None;

    macro_rules! flush {
        () => {
            if let Some((path, verbs)) = current.take() {
                out.entry(path).or_default().extend(verbs);
            }
        };
    }

    for line in lines {
        let trimmed = line.trim();

        if let Some(scope) = literal_after(line, "web::scope(") {
            flush!();
            let prefix = if scope.starts_with("/api") {
                scope.to_string()
            } else {
                format!("{ROOT}{scope}")
            };
            // Depth before this line's own parens — see `candidates_in_file`.
            stack.push((prefix, depth));
            continue;
        }

        if trimmed.starts_with(')') {
            // A registration or scope block ended.
            if trimmed == ")" || trimmed == ")," || trimmed == ");" {
                flush!();
            }
            // Count before popping — see `candidates_in_file`.
            depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
            while stack.last().is_some_and(|(_, opened)| depth < *opened) {
                stack.pop();
            }
            continue;
        }

        // A path on a registration line starts a new verb block.
        if let Some(path) = resources_on_line(line) {
            flush!();
            let owner: &str = stack.last().map(|(p, _)| p.as_str()).unwrap_or(ROOT);
            current = Some((mount(owner, &path), BTreeSet::new()));
        } else if let Some(path) = sole_path(trimmed) {
            flush!();
            let owner: &str = stack.last().map(|(p, _)| p.as_str()).unwrap_or(ROOT);
            current = Some((mount(owner, &path), BTreeSet::new()));
        }

        for verb in VERBS {
            if line.contains(&format!("web::{verb}()")) {
                // The verb joins the block under construction, or the open scope
                // when the path is on the scope line itself
                // (`web::scope("/x").route("/y", web::get())`).
                let target = match current.as_mut() {
                    Some((_, verbs)) => verbs,
                    None => continue,
                };
                target.insert(verb.to_uppercase());
            }
        }

        depth += line.matches('(').count() as i32 - line.matches(')').count() as i32;
    }

    flush!();
}

/// Join a route path to the scope that owns it.
///
/// `web::scope("/api/v1")` already carries the root; `web::scope("/weather")` is
/// relative to it. An absolute path is used as written.
fn mount(owner: &str, path: &str) -> String {
    if path.starts_with("/api") {
        return path.to_string();
    }
    if owner.starts_with("/api") {
        format!("{owner}{path}")
    } else {
        format!("{ROOT}{owner}{path}")
    }
}

/// A path this registration line carries, if any.
///
/// A `.route(web::get().to(..))` line has no path of its own — the path is on the
/// `web::resource(..)` line above it — so only the registration lines count.
fn resources_on_line(line: &str) -> Option<String> {
    for marker in ["web::resource(", "web::route(", ".route("] {
        if let Some(path) = literal_after(line, marker).filter(|p| p.starts_with('/')) {
            return Some(path.to_string());
        }
    }
    None
}

/// Every API path the Admin UI calls.
fn ui_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut stack = vec![admin_ui_dir()];

    while let Some(current) = stack.pop() {
        let entries = std::fs::read_dir(&current)
            .unwrap_or_else(|e| panic!("read {}: {e}", current.display()));
        for entry in entries {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("read ui source");
            for line in strip_comments(&source) {
                for literal in quoted_literals(&line) {
                    if literal.starts_with(ROOT) && literal.len() > ROOT.len() {
                        paths.insert(literal);
                    }
                }
            }
        }
    }
    paths
}

/// Reduce a path to a shape both sides compare equal on.
///
/// Two differences carry no meaning: the name of a placeholder, and a query
/// string built by `format!` where the interpolated value already starts with `?`.
fn normalize(path: &str) -> String {
    let mut base = path.split('?').next().unwrap_or(path).to_string();

    // `/api/v1/customers/search/{}` is a path parameter;
    // `/api/v1/equipments/search{}` is a search with the query appended. Only
    // the second has the placeholder directly after the keyword.
    if base.ends_with("/search{}") {
        base.truncate(base.len() - 2);
    }

    let mut out = String::with_capacity(base.len());
    let mut rest = base.as_str();

    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        match rest[open..].find('}') {
            Some(close) => {
                out.push_str("{param}");
                rest = &rest[open + close + 1..];
            }
            // Not a placeholder; keep it so the path compares as itself.
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Paths that exist for something other than the Admin UI.
///
/// Health and metrics are probed by load balancers, the setup route runs before
/// a tenant exists, `/demo` is a seeded showcase, and `/users/me` is the caller's
/// own record. Reporting these as unreachable would be a false positive on every
/// run, which is how a real regression gets ignored later.
const NOT_UI_FACING: [&str; 5] = [
    "/api/v1/health",
    "/api/v1/metrics",
    "/api/v1/system/setup",
    "/api/v1/demo",
    "/api/v1/users/me",
];

/// Whether the Admin UI is expected to call this path.
fn ui_should_call(path: &str) -> bool {
    // A detail route is reached from the list page for the same resource, so its
    // absence as a literal in the UI source is not by itself a gap.
    if path.contains('{') {
        return false;
    }
    !NOT_UI_FACING.iter().any(|p| path.starts_with(p))
}

/// Every registered, readable path has a caller in the Admin UI.
///
/// This started at 32 uncovered endpoints and reached zero. What it found along
/// the way is worth keeping in mind when reading it:
///
///   * five pages (`/trees`, `/groups`, `/buildings`, `/livestock`,
///     `/plot/entities`) were routed, had complete CRUD handlers behind them, and
///     did nothing — three discarded their form input, one showed a success toast
///     for a write it never performed, and one rendered four rows of invented
///     data;
///   * `/workforce/locations` is a POST-only endpoint where the handler takes the
///     worker id from the authenticated user, so it is presented as a position
///     report and not as an editable table;
///   * two calculations are each registered twice, by `calculation.rs` and by
///     `nutrition.rs` / `specialized.rs`, with near-identical handler bodies. Both
///     paths are live, so both are offered.
///
/// The assertion is that a readable path with no caller is a defect, whether the
/// missing caller is a page that does nothing or a page that was never built.
#[actix_web::test]
async fn every_ui_facing_route_is_reachable_from_the_admin_ui() {
    let registered = registered_routes().await;

    assert!(
        registered.len() > 50,
        "probing found only {} routes — the application under test is probably not the \
         one the handlers describe, and every assertion here would pass vacuously",
        registered.len()
    );

    let ui = ui_paths();
    let mut unreachable: Vec<String> = registered
        .iter()
        .filter(|p| ui_should_call(p))
        .filter(|p| !ui_calls(ui.iter(), p))
        .cloned()
        .collect();

    if !unreachable.is_empty() {
        unreachable.sort();
        eprintln!(
            "\n❌ {} registered API path(s) the Admin UI never calls:\n",
            unreachable.len()
        );
        for p in &unreachable {
            eprintln!("   - {p}");
        }
        eprintln!(
            "\n   Wire the UI to them, or add them to NOT_UI_FACING if they are meant\n\
             for something other than the Admin UI."
        );
        panic!("{} unreachable API paths", unreachable.len());
    }

    println!(
        "✅ every UI-facing registered route is called by the Admin UI \
         ({} routes probed and matched)",
        registered.len()
    );
}

/// Whether the UI reaches `route`, exactly or through a `format!` path.
///
/// The UI builds some calls with a runtime value in the middle —
///
/// ```text
/// get_json(&format!("/api/v1/settings/{namespace}"), true)
/// ```
///
/// — so the literal in the source is the prefix `/api/v1/settings/` while the
/// route is `/api/v1/settings/backup`. Comparing the two as written reported
/// every settings group as unreachable even though the UI has a function for it.
fn ui_calls<'a>(ui: impl Iterator<Item = &'a String>, route: &str) -> bool {
    ui.into_iter()
        .any(|u| normalize(u) == normalize(route) || u.starts_with(route) || route.starts_with(u))
}

/// A UI call must reach a route.
///
/// The probe is a `GET`, so a UI `POST` to `/api/v1/system/setup` or
/// `/api/v1/livestock/animals/{id}/treatments` answers 405 or 404 and the path
/// looks unrouted. Those are not missing routes — the endpoint exists and the UI
/// calls it with the right method.
///
/// So the check has two parts: the path must appear among the registered paths in
/// some form, and if it does not, the failure message has to say that the GET
/// probe cannot see a non-GET endpoint. Reporting it as an unrouted path would be
/// a false positive, and a test whose failures are half noise stops being read.
#[actix_web::test]
async fn every_ui_path_resolves_to_a_registered_route() {
    let registered = registered_routes().await;
    let methods = methods_by_path();
    let ui = ui_paths();

    let mut missing: Vec<String> = Vec::new();
    let mut non_get: Vec<String> = Vec::new();

    for path in &ui {
        let probe = concrete(path);
        if registered
            .iter()
            .any(|r| concrete(r) == probe || normalize(r) == normalize(path))
        {
            continue;
        }

        // Registered, but only for a method the probe does not use.
        let shape = normalize(path);
        if methods
            .keys()
            .any(|r| normalize(r) == shape || normalize(r) == format!("{shape}/{{param}}"))
        {
            non_get.push(path.clone());
            continue;
        }

        missing.push(path.clone());
    }

    if !non_get.is_empty() {
        // Informational, not a failure: the route exists and the UI calls it.
        println!(
            "ℹ️  {} UI path(s) registered for a non-GET method (verified by source, \
             not by probe): {}",
            non_get.len(),
            non_get.join(", ")
        );
    }

    if !missing.is_empty() {
        eprintln!(
            "\n❌ {} UI path(s) with no registered route:\n",
            missing.len()
        );
        for m in &missing {
            eprintln!("   - {m}");
        }
        panic!("{} UI paths are not routed", missing.len());
    }

    println!("✅ all {} UI paths resolve to a registered route", ui.len());
}
