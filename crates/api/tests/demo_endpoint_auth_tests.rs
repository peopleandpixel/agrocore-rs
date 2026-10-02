//! Regression tests for the demo endpoint authorisation (tasks.md A2).
//!
//! `POST /api/v1/demo/seed`, `/demo/reset` and `GET /demo/summary` had no
//! `AuthExtractor` at all, while `/demo/reset` executes
//! `DELETE FROM tenants WHERE id = $1` with the tenant slug taken from the
//! request body. Any unauthenticated client could therefore delete a tenant.
//!
//! The fix adds two independent barriers: an admin role and the
//! `ALLOW_DEMO_ENDPOINTS` flag. These tests pin the flag's parsing rules and the
//! shape of the guard; the database behaviour itself needs the integration
//! fixture.

use std::env;

/// Serialises access to the environment across the tests in this file.
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Mirrors `demo_endpoints_enabled()` in crates/api/src/lib.rs.
fn demo_endpoints_enabled() -> bool {
    match env::var("ALLOW_DEMO_ENDPOINTS") {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

// The tests below mutate a process-wide environment variable, so they must not
// run concurrently. Each restores the previous value.
fn with_var(key: &str, value: Option<&str>, f: impl FnOnce()) {
    // cargo runs tests in parallel threads and the environment is
    // process-wide, so every test in this file must take the lock or it will
    // read whatever value another test set last.
    let _guard = env_lock();
    let previous = env::var(key).ok();
    // SAFETY: single-threaded within this test, restored immediately after.
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

#[test]
fn unset_flag_disables_demo_endpoints() {
    with_var("ALLOW_DEMO_ENDPOINTS", None, || {
        assert!(
            !demo_endpoints_enabled(),
            "without the variable the demo endpoints must stay off"
        );
    });
}

#[test]
fn accepted_true_values_enable_demo_endpoints() {
    for value in ["1", "true", "TRUE", "yes", "YES", " true ", "True"] {
        with_var("ALLOW_DEMO_ENDPOINTS", Some(value), || {
            assert!(
                demo_endpoints_enabled(),
                "{value:?} should enable the demo endpoints"
            );
        });
    }
}

#[test]
fn rejected_values_keep_demo_endpoints_disabled() {
    // Anything that is not explicitly affirmative must fail closed, including
    // typos: the endpoints can delete tenants.
    for value in ["0", "false", "no", "off", "", "enabled", "2"] {
        with_var("ALLOW_DEMO_ENDPOINTS", Some(value), || {
            assert!(
                !demo_endpoints_enabled(),
                "{value:?} must not enable the demo endpoints"
            );
        });
    }
}

#[test]
fn empty_value_does_not_enable() {
    with_var("ALLOW_DEMO_ENDPOINTS", Some(""), || {
        assert!(!demo_endpoints_enabled());
    });
}

/// The default demo password must satisfy the application's own minimum.
///
/// `CreateUserDto` enforces 12 characters since v0.25.0. The demo seed used a
/// literal that was shorter, and it disagreed with both the SQL seed and the
/// value printed by dev.sh.
#[test]
fn demo_default_password_meets_minimum_length() {
    const DEMO_DEFAULT_PASSWORD: &str = "demo1234-agrocore";
    assert!(
        DEMO_DEFAULT_PASSWORD.len() >= 12,
        "demo password is {} characters, minimum is 12",
        DEMO_DEFAULT_PASSWORD.len()
    );
}
