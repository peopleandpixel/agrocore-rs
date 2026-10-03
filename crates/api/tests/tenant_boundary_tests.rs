//! Tenant-boundary regressions for restore and IoT devices (tasks.md B2, B3).
//!
//! Two handler bugs shared one root cause: `is_admin()` was treated as a
//! superadmin flag that could widen the tenant boundary. It is a role *within* a
//! tenant.
//!
//!   * `delete_device` checked `device.tenant_id != auth.0.tenant_id &&
//!     !auth.is_admin()` after `require_admin()`, so the condition was
//!     constant-false — and the DELETE behind it had no tenant condition of its
//!     own.
//!   * `RestoreRequest.target_database` came from the request and was passed
//!     straight to `pg_restore`, which runs with `--clean` and therefore empties
//!     whatever it is pointed at.
//!
//! # Why this file checks source text
//!
//! Both bugs are about *which code path exists*, not about a value computed at
//! runtime. A behavioural test would need a request whose target database is a
//! second real database, and asserting that the second database was not touched
//! proves only that the fixture was wired correctly — the same class of test that
//! let the bug through in the first place, since the old code had no reachable
//! path a test could drive without a live second database.
//!
//! So the assertions are on the source: the dangerous construct must not appear,
//! and the safe one must. That is a blunt instrument, and it is deliberately
//! blunt: a regression reintroduces the exact line these tests forbid.
//!
//! What this cannot catch: a future author who reintroduces the same capability
//! under different wording. That is what the code review and the DB-level
//! tenant-isolation tests are for.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/api -> repo root")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Every `&& !auth.is_admin()` style guard is a bug, wherever it appears.
///
/// The condition is only ever evaluated after a `require_*` call, so the
/// `!auth.is_admin()` branch is always false and the guard cannot reject.
#[test]
fn no_guard_uses_is_admin_to_widen_the_tenant_boundary() {
    let handlers = repo_root().join("crates/api/src/handlers");
    let mut offenders = Vec::new();

    for entry in std::fs::read_dir(&handlers).expect("read handlers dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read handler");
        for (number, line) in text.lines().enumerate() {
            let code = line.trim();
            if code.starts_with("//") || code.starts_with("///") {
                continue;
            }
            if code.contains("&& !auth.is_admin()") || code.contains("|| auth.is_admin()") {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.file_name().unwrap().to_string_lossy(),
                    number + 1,
                    code
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "`is_admin()` is a role within a tenant and must not widen the tenant \
         boundary. Put the tenant predicate in the SQL instead:\n{}",
        offenders.join("\n")
    );
}

/// `delete_device` must filter on the tenant in the statement.
///
/// A check in the handler body does not protect the row: anything that reaches
/// the DELETE without passing the check deletes across tenants.
#[test]
fn the_iot_delete_filters_on_the_tenant() {
    let iot = read("crates/api/src/handlers/iot.rs");

    let delete_line = iot
        .lines()
        .find(|l| l.contains("DELETE FROM iot_devices"))
        .expect("delete_device must issue a DELETE");

    assert!(
        delete_line.contains("tenant_id"),
        "the DELETE must carry a tenant condition, otherwise any tenant admin \
         can delete another tenant's device: {delete_line}"
    );
}

/// The production restore path must not take a database name from anywhere.
///
/// `pg_restore --clean` drops the objects in the target before writing, so a
/// caller-chosen target is a data-destruction primitive.
#[test]
fn the_restore_path_has_no_caller_supplied_target() {
    let service = read("crates/backup-service/src/service.rs");

    let signature = service
        .lines()
        .find(|l| l.contains("pub async fn restore("))
        .expect("BackupService::restore signature");

    assert!(
        !signature.contains("target_db") && !signature.contains("Option<String>"),
        "BackupService::restore must not accept a database name: {signature}"
    );
}

/// The escape hatch for verification — which restores into a throwaway database
/// it created itself — must exist and must be named so its narrow purpose is
/// visible at the call site.
#[test]
fn the_named_restore_exists_only_for_verification() {
    let pg_dump = read("crates/backup-service/src/pg_dump.rs");

    assert!(
        pg_dump.contains("pub async fn restore_into_named_database"),
        "verification restores into a throwaway database and needs a way to say so"
    );

    // The default entry point must delegate to the configured URL, not to the
    // named variant.
    let default_entry = pg_dump
        .split("pub async fn restore_from_storage")
        .nth(1)
        .expect("restore_from_storage body")
        .chars()
        .take(400)
        .collect::<String>();

    assert!(
        default_entry.contains("self.database_url"),
        "restore_from_storage must use the configured database URL"
    );
}

/// The API request must not carry the field at all.
///
/// Leaving it in the DTO would keep it in the OpenAPI schema, so a client would
/// still send it and a later refactor could reintroduce the wiring.
#[test]
fn the_restore_dto_has_no_target_database_field() {
    let dto = read("crates/api/src/dto/backup.rs");

    let body = dto
        .split("pub struct RestoreRequest")
        .nth(1)
        .expect("RestoreRequest definition")
        .split('}')
        .next()
        .expect("struct body");

    // Only the field lines, not the explanatory comment that follows the struct:
    // the comment mentions the field by name precisely to say it is absent.
    let fields: Vec<&str> = body
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pub "))
        .collect();

    assert!(
        !fields.iter().any(|f| f.contains("target_database")),
        "RestoreRequest must not expose a target database, found: {fields:?}"
    );
    assert!(
        fields.iter().any(|f| f.contains("dry_run")),
        "sanity: the body was parsed at all, found: {fields:?}"
    );
}

/// The handler must not read a target from the request.
#[test]
fn the_restore_handler_does_not_read_a_target() {
    let backup = read("crates/api/src/handlers/backup.rs");
    assert!(
        !backup.contains("target_database"),
        "the restore handler must not read a target database from the request"
    );
}
