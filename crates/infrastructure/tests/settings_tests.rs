//! Settings persistence and tenant scoping (tasks.md F1/H1).
//!
//! The company profile used to live in the browser's localStorage, so settings
//! were per-device and invisible to the backend. These tests cover the part that
//! makes that impossible: a write is visible to the server, is scoped to one
//! tenant, and inherits the system default until it is overridden.
//!
//! Need a migrated database with the shipped defaults:
//!
//! ```text
//! DATABASE_URL=... cargo test -p agrocore-infrastructure \
//!   --test settings_tests -- --ignored --test-threads=1
//! ```

use agrocore_domain::entities::setting::{SettingValueType, UpdateSetting};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_infrastructure::postgres::setting::PgSettingsRepo;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// The app role, so the tenant policies apply exactly as they do in production.
async fn app_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE agrocore_app")
                    .execute(&mut *conn)
                    .await
                    .map(|_| ())
                    .map_err(|e| sqlx::Error::Configuration(Box::new(e)))
            })
        })
        .connect(&url)
        .await
        .expect("connect")
}

/// A tenant plus the cleanup that removes it again.
struct Fixture {
    pool: PgPool,
    tenant: TenantId,
    user: Uuid,
    repo: PgSettingsRepo,
}

impl Fixture {
    async fn new() -> Self {
        let pool = app_pool().await;
        let admin = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("admin pool");

        let tenant = Uuid::new_v4();
        let user = Uuid::new_v4();
        sqlx::query("INSERT INTO tenants (id, name, slug, is_active) VALUES ($1, $2, $3, true)")
            .bind(tenant)
            .bind("Settings Test")
            .bind(format!("settings-test-{tenant}"))
            .execute(&admin)
            .await
            .expect("insert tenant");

        // `system_settings.updated_by` references `users`, so the fixture needs
        // a real user. Any hash works: this test never authenticates.
        sqlx::query(
            "INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash,
                                language, color, is_active, roles, created_at, updated_at)
             VALUES ($1, $2, 'Settings', 'Test', $3, 'x', 'de', '#000000', true, '[]'::jsonb,
                     NOW(), NOW())",
        )
        .bind(user)
        .bind(tenant)
        .bind(format!("settings-{user}@test.local"))
        .execute(&admin)
        .await
        .expect("insert user");

        let repo = PgSettingsRepo::new(pool.clone());
        Self {
            pool,
            tenant: TenantId(tenant),
            user,
            repo,
        }
    }

    async fn cleanup(&self) {
        // RLS blocks a tenant-scoped delete from an unpinned connection, so the
        // fixture's own tenant is removed through the pinned path the app uses.
        let _ = self.repo.reset(self.tenant, "company.name").await;
        // Remove the tenant itself as superuser.
        let admin = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("admin pool");
        // users cascades from tenants, so removing the tenant removes both.
        let _ = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(self.tenant.0)
            .execute(&admin)
            .await;
    }
}

#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_fresh_tenant_sees_the_shipped_defaults() {
    let fx = Fixture::new().await;

    let rows = fx.repo.list_effective(fx.tenant).await.expect("list");
    assert!(!rows.is_empty(), "shipped defaults must be visible");

    let company_name = rows
        .iter()
        .find(|r| r.entry.key == "company.name")
        .expect("company.name is a known key");
    assert!(
        company_name.entry.is_default,
        "a value nobody overrode must be reported as inherited"
    );
    assert!(company_name.entry.value.is_string());

    fx.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_written_setting_persists_and_is_not_a_default() {
    let fx = Fixture::new().await;
    let value = format!("Farm {}", &Uuid::new_v4().to_string()[..8]);

    fx.repo
        .set(
            fx.tenant,
            fx.user,
            UpdateSetting {
                key: "company.name".into(),
                value: json!(value),
            },
        )
        .await
        .expect("set");

    // Read back through a fresh pool: the point is that this is on the server,
    // not in browser storage.
    let other_repo = PgSettingsRepo::new(fx.pool.clone());
    let entry = other_repo
        .get(fx.tenant, "company.name")
        .await
        .expect("get")
        .expect("the setting exists");

    assert_eq!(entry.value, json!(value));
    assert!(!entry.is_default, "an override must not report as default");

    fx.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn reset_falls_back_to_the_default() {
    let fx = Fixture::new().await;
    let value = format!("Override {}", &Uuid::new_v4().to_string()[..8]);

    fx.repo
        .set(
            fx.tenant,
            fx.user,
            UpdateSetting {
                key: "company.name".into(),
                value: json!(value),
            },
        )
        .await
        .expect("set");

    assert!(
        fx.repo
            .reset(fx.tenant, "company.name")
            .await
            .expect("reset")
    );

    let entry = fx
        .repo
        .get(fx.tenant, "company.name")
        .await
        .expect("get")
        .expect("default still exists");
    assert!(entry.is_default, "after reset the default applies again");

    // A second reset has nothing to remove and must say so, so the UI can tell
    // "reset" from "there was nothing to reset".
    assert!(
        !fx.repo
            .reset(fx.tenant, "company.name")
            .await
            .expect("reset again")
    );

    fx.cleanup().await;
}

/// The core isolation property: one tenant's settings must never appear in
/// another's list.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn one_tenant_cannot_see_another_tenants_override() {
    let a = Fixture::new().await;
    let b = Fixture::new().await;

    let marker = format!("Secret {}", &Uuid::new_v4().to_string()[..8]);
    a.repo
        .set(
            a.tenant,
            a.user,
            UpdateSetting {
                key: "company.name".into(),
                value: json!(marker),
            },
        )
        .await
        .expect("set for A");

    let b_rows = b.repo.list_effective(b.tenant).await.expect("B lists");
    assert!(
        !b_rows.iter().any(|r| r.entry.value == json!(marker)),
        "tenant B must not see tenant A's override"
    );

    // B still gets the default for that key, not A's value.
    let b_name = b
        .repo
        .get(b.tenant, "company.name")
        .await
        .expect("B get")
        .expect("B sees the default");
    assert!(b_name.is_default);
    assert_ne!(b_name.value, json!(marker));

    a.cleanup().await;
    b.cleanup().await;
}

/// A declared `boolean` must not accept a string, or the setting would only fail
/// later at the point where it is read as a bool.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_boolean_rejects_a_string_value() {
    let fx = Fixture::new().await;

    let result = fx
        .repo
        .set(
            fx.tenant,
            fx.user,
            UpdateSetting {
                key: "backup.enabled".into(),
                value: json!("yes please"),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "backup.enabled is declared boolean and must reject a string"
    );

    // And the accepted form still works.
    fx.repo
        .set(
            fx.tenant,
            fx.user,
            UpdateSetting {
                key: "backup.enabled".into(),
                value: json!(false),
            },
        )
        .await
        .expect("boolean accepted");

    let entry = fx
        .repo
        .get(fx.tenant, "backup.enabled")
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(entry.value, json!(false));
    assert_eq!(entry.value_type, SettingValueType::Boolean);

    fx.cleanup().await;
}

/// An unknown key is accepted: settings are meant to be extensible without a
/// code change. The type is inferred from the value.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn an_unknown_key_is_stored_with_an_inferred_type() {
    let fx = Fixture::new().await;
    let key = format!("custom.{}", &Uuid::new_v4().to_string()[..8]);

    fx.repo
        .set(
            fx.tenant,
            fx.user,
            UpdateSetting {
                key: key.clone(),
                value: json!(42),
            },
        )
        .await
        .expect("unknown key accepted");

    let entry = fx
        .repo
        .get(fx.tenant, &key)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(entry.value_type, SettingValueType::Number);

    fx.cleanup().await;
}
