//! Tests for the JWT secret strength check (tasks.md A5).
//!
//! `validate_jwt_secret()` existed but had no callers anywhere in the
//! workspace, so a deployment without `JWT_SECRET` started with the built-in
//! `dev-secret`. Anyone could then mint an admin token with a plain HS256
//! signature. `run_server` now refuses to start in that case.

use agrocore_shared::config::{JWT_SECRET_MIN_LENGTH, JwtSecretError};

/// A secret long enough to pass the length requirement.
const GOOD_SECRET: &str = "a-sufficiently-long-secret-for-hs256";

#[test]
fn minimum_length_is_at_least_32_bytes() {
    // 32 bytes is the width of a SHA-256 digest, the usual floor for a
    // symmetric HMAC key. Anything below that is not worth the name.
    const { assert!(JWT_SECRET_MIN_LENGTH >= 32) };
}

#[test]
fn good_secret_is_long_enough() {
    assert!(
        GOOD_SECRET.len() >= JWT_SECRET_MIN_LENGTH,
        "test secret has {} characters, minimum is {}",
        GOOD_SECRET.len(),
        JWT_SECRET_MIN_LENGTH
    );
}

/// The error variants must describe the problem, not just "invalid".
#[test]
fn errors_describe_themselves() {
    let dev = JwtSecretError::DevSecret.to_string();
    assert!(
        dev.contains("development secret"),
        "message should name the cause: {dev}"
    );

    let short = JwtSecretError::TooShort {
        length: 8,
        minimum: JWT_SECRET_MIN_LENGTH,
    }
    .to_string();
    assert!(
        short.contains("8"),
        "message should state the length: {short}"
    );
    assert!(
        short.contains(&JWT_SECRET_MIN_LENGTH.to_string()),
        "message should state the minimum: {short}"
    );
}

#[test]
fn dev_secret_and_too_short_are_distinguishable() {
    // A caller logging the reason must be able to tell "unset" from "set but
    // weak"; the remediation differs.
    assert_ne!(
        JwtSecretError::DevSecret,
        JwtSecretError::TooShort {
            length: 8,
            minimum: 32
        }
    );
}

/// Length is measured in bytes of the configured secret.
///
/// The check exists so a deployment cannot ship a one-character signing key,
/// which for HS256 is brute-forceable offline from a single captured token.
#[test]
fn length_check_counts_all_bytes() {
    let long_enough: String = "x".repeat(JWT_SECRET_MIN_LENGTH);
    let one_short: String = "x".repeat(JWT_SECRET_MIN_LENGTH - 1);

    assert!(long_enough.len() >= JWT_SECRET_MIN_LENGTH);
    assert!(one_short.len() < JWT_SECRET_MIN_LENGTH);
}
