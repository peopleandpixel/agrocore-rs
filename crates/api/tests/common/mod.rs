//! Shared `AppState` construction for the integration tests.
//!
//! Thirty `AppState { ... }` literals across ten test files each listed the fields
//! they happened to need, so every time `AppState` gained a field — metrics and the
//! backup service, most recently — each one stopped compiling and had to be patched
//! individually. Nine of these targets did not compile at all for exactly that
//! reason, which meant the `mocks` feature had no coverage at all and every suite
//! that used a repository mock was invisible to `cargo test --workspace`.
//!
//! A field that a test does not care about should not appear in a test. This module
//! sets them once, to values that are inert but real: an empty metrics registry, no
//! backup service, and `messaging: None` — which is what the field became when the
//! broker was made optional, and is also what `new_mock()` returned for years
//! except that it panicked on every call.

#![allow(dead_code)]

use agrocore_api::AppState;
use agrocore_infrastructure::{Database, MockDatabase};
use std::sync::Arc;

/// Builds an `AppState` around a mock database, with the fields no test in this
/// suite exercises filled in.
///
/// `messaging` is `None` rather than a mock: `MessagingClient::new_mock()` is a
/// placeholder that panics on every call, so any test that reached it would fail for
/// a reason unrelated to what it was testing. `AppState::messaging` is an `Option`
/// precisely so a broker outage does not stop the API from starting, and the handlers
/// in this suite ignore it.
pub fn state_with(mock_db: MockDatabase) -> AppState {
    let metrics_registry = prometheus::Registry::new();
    AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: None,
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
        db_metrics: agrocore_api::metrics::DbMetrics::new(&metrics_registry),
        business_metrics: agrocore_api::metrics::BusinessMetrics::new(&metrics_registry),
        metrics_registry: Arc::new(metrics_registry),
        backup_service: None,
        demo_endpoints_enabled: false,
    }
}

/// The token an authenticated request carries.
///
/// `roles` is what `generate_jwt` puts in a token, not the `UserRole` variant name:
/// `UserRole::Admin` serialises as `"Admin"`. The impersonation endpoint was
/// unreachable for months because a handler compared against `"admin"`; these tests
/// use the strings a real token has so that class of mistake cannot hide here.
pub fn signed_token(sub: &str, tenant_id: &str, roles: &[&str]) -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        tenant_id: String,
        roles: Vec<String>,
        exp: usize,
        jti: String,
    }

    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.iter().map(|r| (*r).to_string()).collect(),
            exp: usize::MAX / 2,
            jti: uuid::Uuid::new_v4().to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

/// Wraps a mock database so its repository slots can be filled in the initializer.
///
/// Every integration test in this suite needs exactly this: build a
/// `MockDatabase`, give it one repository, hand it to the app. Written as
///
/// ```ignore
/// let mut mock_db = MockDatabase::default();
/// mock_db.water_source_repo = Some(Arc::new(repo));
/// ```
///
/// that trips clippy's `field_reassign_with_default` forty-one times across ten
/// files — the lint is right, since the value is known before the binding exists.
///
/// The setter is deliberately private to this module's callers: a test that adds a
/// second repository uses the setter again, and the struct-update form
/// (`MockDatabase { a: .., ..Default::default() }`) stays available for the rare
/// test that needs two at once.
pub struct MockDbBuilder(MockDatabase);

impl MockDbBuilder {
    /// Starts a builder with no repositories set.
    pub fn new() -> Self {
        Self(MockDatabase::default())
    }

    /// Sets one repository slot.
    pub fn with(mut self, set: impl FnOnce(&mut MockDatabase)) -> Self {
        set(&mut self.0);
        self
    }

    /// Consumes the builder, producing the database the app state is built around.
    pub fn build(self) -> MockDatabase {
        self.0
    }
}

impl Default for MockDbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a mock database with a single repository configured.
///
/// ```ignore
/// let mock_db = crate::common::db_with(|db| db.water_source_repo = Some(Arc::new(repo)));
/// ```
pub fn db_with(set: impl FnOnce(&mut MockDatabase)) -> MockDatabase {
    MockDbBuilder::new().with(set).build()
}
