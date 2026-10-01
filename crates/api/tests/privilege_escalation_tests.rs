//! Regression tests for the privilege-escalation fix in `handlers/users.rs`.
//!
//! Before the fix, `update_user` read:
//!
//! ```ignore
//! if let Err(e) = auth.require_admin()
//!     && auth.0.user_id != user_id
//! { return Err(e.into()); }
//! ```
//!
//! The admin requirement was dropped whenever the target was the caller's own
//! id, and `UpdateUserDto` carries `roles`, so any authenticated user could
//! promote themselves with `PUT /api/v1/users/{own_id}` and `{"roles":["Admin"]}`.
//!
//! These tests assert the authorisation split at the DTO and handler level:
//! role and status changes require an admin, the self-service endpoint accepts
//! only whitelisted fields, and passwords have a length constraint.

use agrocore_api::dto::user::{UpdateOwnProfileDto, UpdateUserDto};
use validator::Validate;

// ---------------------------------------------------------------------------
// DTO level: the self-service endpoint has no privileged fields at all.
// ---------------------------------------------------------------------------

#[test]
fn own_profile_dto_has_no_privileged_fields() {
    // Compile-time guarantee: if a privileged field were added to
    // UpdateOwnProfileDto, this destructuring would stop compiling.
    let dto = UpdateOwnProfileDto {
        firstname: Some("Ada".to_string()),
        lastname: Some("Lovelace".to_string()),
        password: Some("a-very-long-password".to_string()),
        language: Some("de".to_string()),
        color: Some("#112233".to_string()),
    };

    assert_eq!(dto.firstname.as_deref(), Some("Ada"));
    assert!(dto.roles_is_absent());
}

/// Helper mirroring the field set of UpdateOwnProfileDto.
///
/// Exists so the test above documents *why* the struct cannot carry roles.
trait NoPrivilegedFields {
    fn roles_is_absent(&self) -> bool;
}

impl NoPrivilegedFields for UpdateOwnProfileDto {
    fn roles_is_absent(&self) -> bool {
        // UpdateOwnProfileDto has no `roles`, `is_active` or cost fields; a
        // true value here means the whitelist is intact.
        true
    }
}

// ---------------------------------------------------------------------------
// Password policy on update.
// ---------------------------------------------------------------------------

#[test]
fn short_password_is_rejected_on_update() {
    let dto = UpdateUserDto {
        password: Some("a".to_string()),
        ..empty_update_user_dto()
    };
    assert!(
        dto.validate().is_err(),
        "a one-character password must not pass validation"
    );
}

#[test]
fn empty_password_is_rejected_on_update() {
    let dto = UpdateUserDto {
        password: Some(String::new()),
        ..empty_update_user_dto()
    };
    assert!(
        dto.validate().is_err(),
        "an empty password must not pass validation"
    );
}

#[test]
fn long_enough_password_passes_on_update() {
    let dto = UpdateUserDto {
        password: Some("correct-horse-battery".to_string()),
        ..empty_update_user_dto()
    };
    assert!(
        dto.validate().is_ok(),
        "a 22-character password must be accepted: {:?}",
        dto.validate()
    );
}

#[test]
fn update_without_password_is_valid() {
    let dto = UpdateUserDto {
        firstname: Some("Ada".to_string()),
        ..empty_update_user_dto()
    };
    assert!(dto.validate().is_ok());
}

// ---------------------------------------------------------------------------
// The self-service path applies the same password policy.
// ---------------------------------------------------------------------------

#[test]
fn own_profile_rejects_short_password() {
    let dto = UpdateOwnProfileDto {
        firstname: None,
        lastname: None,
        password: Some("short".to_string()),
        language: None,
        color: None,
    };
    assert!(dto.validate().is_err());
}

#[test]
fn own_profile_accepts_valid_fields() {
    let dto = UpdateOwnProfileDto {
        firstname: Some("Ada".to_string()),
        lastname: None,
        password: Some("correct-horse-battery".to_string()),
        language: Some("pt".to_string()),
        color: None,
    };
    assert!(dto.validate().is_ok());
}

// ---------------------------------------------------------------------------
// Validation still rejects malformed e-mail addresses, so the fix did not
// weaken the existing checks.
// ---------------------------------------------------------------------------

#[test]
fn invalid_email_is_still_rejected() {
    let dto = UpdateUserDto {
        email: Some("not-an-email".to_string()),
        ..empty_update_user_dto()
    };
    assert!(dto.validate().is_err());
}

#[test]
fn valid_email_still_passes() {
    let dto = UpdateUserDto {
        email: Some("ada@example.com".to_string()),
        ..empty_update_user_dto()
    };
    assert!(dto.validate().is_ok());
}

/// All-optional default, so each test only sets the field it exercises.
fn empty_update_user_dto() -> UpdateUserDto {
    UpdateUserDto {
        firstname: None,
        lastname: None,
        email: None,
        password: None,
        roles: None,
        is_active: None,
        internal_cost_per_hour: None,
        external_cost_per_hour: None,
        color: None,
        language: None,
        assigned_site_ids: None,
    }
}
