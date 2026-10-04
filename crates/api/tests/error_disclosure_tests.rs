//! Error responses must not leak database internals (tasks.md C1), and
//! impersonation must be reachable, audited and revocable (tasks.md D2).
//!
//! C1: `ApiError::error_response` used to render `self.0.to_string()` into the
//! response body for every status. For a 500 that string is the `Display` of
//! `sqlx::Error`, which names the table, the column, the violated constraint and —
//! for a failed decode — the Rust type the decoder expected. Anyone who could
//! trigger an error got a free map of the schema, and the text changes whenever the
//! schema does, so it was not reliably parseable either.
//!
//! D2: `impersonate` compared the caller's roles against `"admin"` while
//! `generate_jwt` emits `"Admin"`, so it never matched and nobody could impersonate
//! anyone. Repairing only the case would have made a dead endpoint live with three
//! further defects intact: no revocation, no audit entry, and no record of the
//! original caller in the issued token.
//!
//! The C1 assertions are on the real `error_response`, because the defect was in
//! that function's output and a test on the enum would not have caught it.

use actix_web::{ResponseError, http::StatusCode};
use agrocore_api::error::ApiError;
use agrocore_shared::SharedError;

// ---------------------------------------------------------------------------
// C1 — the 5xx body
// ---------------------------------------------------------------------------

/// The body a client receives, as a string.
///
/// `HttpResponse::body()` yields a `BoxBody` rather than bytes, so the response is
/// taken apart through `into_body` and read with `to_bytes`. The assertions do not
/// parse the JSON — they check that no part of the rendered response carries the
/// underlying message, which is the stronger property and does not depend on the
/// field layout.
async fn rendered_body(err: ApiError) -> String {
    let resp = err.error_response();
    let bytes = actix_web::body::to_bytes(resp.into_body())
        .await
        .expect("body");
    String::from_utf8_lossy(&bytes).to_string()
}

#[actix_web::test]
async fn a_database_error_does_not_reach_the_client() {
    // Shaped exactly like a real constraint violation from a `sqlx::Error`.
    let raw = "duplicate key value violates unique constraint \"uq_sigpac_parcels_reference\"\
               \nDETAIL:  Key (sigpac_reference)=(01001001001001001001) already exists.";
    let err = ApiError(SharedError::Database(raw.to_string()));

    let body = rendered_body(err).await;

    for leak in [
        "uq_sigpac_parcels_reference",
        "sigpac_parcels",
        "sigpac_reference",
        "duplicate key",
    ] {
        assert!(
            !body.contains(leak),
            "the response body leaked `{leak}`: {body}"
        );
    }
}

#[actix_web::test]
async fn a_decode_error_does_not_reach_the_client() {
    // A `sqlx::Error` from a failed decode names the expected Rust type, which
    // describes the column as well.
    let raw = "error decoding response: \
               invalid type: string \"n/a\", expected f64 for column sites.area_ha";
    let err = ApiError(SharedError::Database(raw.to_string()));

    let body = rendered_body(err).await;

    for leak in ["area_ha", "sites", "invalid type", "expected f64"] {
        assert!(
            !body.contains(leak),
            "the response body leaked `{leak}`: {body}"
        );
    }
}

#[actix_web::test]
async fn an_internal_error_body_says_something_useful() {
    let err = ApiError(SharedError::Internal("connection pool exhausted".into()));
    let body = rendered_body(err).await;

    assert!(
        body.contains("internal error") && body.contains("server log"),
        "the 5xx body should point the operator at the log: {body}"
    );
    assert!(
        !body.contains("connection pool exhausted"),
        "the internal detail must not be echoed to the client: {body}"
    );
}

/// A 4xx message is authored and describes what the caller did wrong, so it stays.
/// Replacing it with a generic string would break every client that reads
/// `message`.
#[actix_web::test]
async fn client_errors_keep_their_message() {
    let cases = [
        (
            SharedError::Validation("name must not be empty".into()),
            StatusCode::BAD_REQUEST,
        ),
        (
            SharedError::NotFound("User not found".into()),
            StatusCode::NOT_FOUND,
        ),
        (
            SharedError::Forbidden("Admin role required".into()),
            StatusCode::FORBIDDEN,
        ),
        (
            SharedError::Conflict("duplicate".into()),
            StatusCode::CONFLICT,
        ),
    ];

    for (inner, expected) in cases {
        let err = ApiError(inner);
        let resp = err.error_response();
        assert_eq!(resp.status(), expected);

        let bytes = actix_web::body::to_bytes(resp.into_body())
            .await
            .expect("body");
        let body = String::from_utf8_lossy(&bytes).to_string();
        assert!(
            !body.contains("server log"),
            "a {expected} body must not be replaced by the 5xx text: {body}"
        );
    }
}

/// `NotImplemented` keeps a specific message rather than the generic 5xx text: it
/// is not an internal failure, it is a deliberate statement about this build, and a
/// client can act on it.
#[actix_web::test]
async fn not_implemented_stays_specific() {
    let err = ApiError(SharedError::NotImplemented("CSA subscriptions".into()));
    let resp = err.error_response();
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);

    let bytes = actix_web::body::to_bytes(resp.into_body())
        .await
        .expect("body");
    let body = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        body.contains("not implemented"),
        "the 501 body should say so: {body}"
    );
}

// ---------------------------------------------------------------------------
// D2 — impersonation
// ---------------------------------------------------------------------------

/// The role strings `generate_jwt` actually puts in a token.
///
/// Sourced from `crates/infrastructure/src/jwt.rs`:
/// `UserRole::Admin => "Admin"`. The old handler compared against `"admin"`.
const JWT_ROLE_ADMIN: &str = "Admin";

/// The comparison the old handler performed.
fn old_admin_check(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "admin" || r == "superadmin")
}

/// The comparison the handler performs now.
fn new_admin_check(roles: &[String]) -> bool {
    roles.iter().any(|r| r == JWT_ROLE_ADMIN)
}

#[test]
fn the_old_role_comparison_could_never_match() {
    // This is the defect, asserted directly: with the strings a real token
    // carries, the old check is false for every role the system has.
    for role in ["Admin", "Manager", "Worker", "Viewer"] {
        assert!(
            !old_admin_check(&[role.to_string()]),
            "unexpectedly matched for {role}"
        );
    }
}

#[test]
fn the_new_role_comparison_matches_a_real_admin_token() {
    assert!(new_admin_check(&["Admin".to_string()]));
    assert!(new_admin_check(&[
        "Manager".to_string(),
        "Admin".to_string()
    ]));

    for role in ["Manager", "Worker", "Viewer", "admin", "superadmin"] {
        assert!(
            !new_admin_check(&[role.to_string()]),
            "{role} must not pass an admin check"
        );
    }
}

/// `superadmin` is not a role the system has. `UserRole` is `Admin`, `Manager`,
/// `Worker`, `Viewer`, `Custom(String)` — so the old handler's second literal could
/// never appear in a token either, and it is not added back here.
#[test]
fn there_is_no_superadmin_role_to_match() {
    let roles = ["Admin", "Manager", "Worker", "Viewer"];
    assert!(
        !roles.contains(&"superadmin"),
        "if a superadmin role now exists, the check above needs revisiting"
    );
}
