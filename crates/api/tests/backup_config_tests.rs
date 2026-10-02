//! Backup-configuration persistence (tasks.md F2).
//!
//! `update_backup_config` answered `200 {"message": "Backup configuration
//! updated"}` without writing anything, so an admin who turned backups off was
//! told it had worked while the schedule kept running. These tests cover the
//! part that makes that impossible: a write survives, and a partial update
//! leaves the fields it did not mention alone.
//!
//! ```text
//! DATABASE_URL=... cargo test -p agrocore-api --test backup_config_tests \
//!   -- --ignored --test-threads=1
//! ```

use agrocore_domain::entities::setting::{SettingEntry, SettingValueType, UpdateSetting};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_infrastructure::postgres::setting::PgSettingsRepo;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// The pool the handler gets: the tenant policies apply exactly as in production.
async fn app_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
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
        sqlx::query(
            "INSERT INTO tenants (id, name, slug, config) VALUES ($1, 'Backup Config', $2, '{}')",
        )
        .bind(tenant)
        .bind(format!("backup-cfg-{tenant}"))
        .execute(&admin)
        .await
        .expect("insert tenant");

        sqlx::query(
            "INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, is_active)
             VALUES ($1, $2, 'Config', 'Admin', $3, 'not-a-real-hash', true)",
        )
        .bind(user)
        .bind(tenant)
        .bind(format!("cfg-{user}@example.invalid"))
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

    /// Write a setting the way the handler does.
    async fn update(&self, key: &str, value: serde_json::Value) {
        self.repo
            .set_many(
                self.tenant,
                self.user,
                vec![UpdateSetting {
                    key: key.to_string(),
                    value,
                }],
            )
            .await
            .expect("write setting");
    }

    async fn read(&self, key: &str) -> Option<SettingEntry> {
        self.repo.get(self.tenant, key).await.expect("read setting")
    }
}

/// A configuration write must be readable afterwards. This is the assertion
/// that was impossible to make before: the handler never wrote, so there was
/// nothing to read back.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_saved_backup_config_survives() {
    let fx = Fixture::new().await;

    fx.update("backup.schedule_db", json!("30 4 * * *")).await;

    let entry = fx.read("backup.schedule_db").await.expect("entry");
    assert_eq!(entry.value, json!("30 4 * * *"));
    assert_eq!(entry.value_type, SettingValueType::String);
}

/// `is_default` must go false once the tenant overrides, because that is what
/// the UI uses to offer the "Zurücksetzen" button.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_saved_backup_config_is_no_longer_a_default() {
    let fx = Fixture::new().await;

    let before = fx.read("backup.retention.daily").await.expect("default");
    assert!(before.is_default, "shipped default must be inherited");

    fx.update("backup.retention.daily", json!(90)).await;

    let after = fx.read("backup.retention.daily").await.expect("override");
    assert!(!after.is_default);
    assert_eq!(after.value, json!(90));
}

/// A partial update is the documented behaviour: the handler builds its write
/// list from the fields present in the request, so an absent field must keep its
/// stored value rather than reverting to the default.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_partial_update_leaves_the_other_fields_alone() {
    let fx = Fixture::new().await;

    fx.update("backup.schedule_db", json!("15 1 * * *")).await;
    fx.update("backup.timezone", json!("Europe/Lisbon")).await;

    // Only the timezone is written, as a client sending `{"timezone": …}` would.
    fx.update("backup.timezone", json!("Europe/Berlin")).await;

    assert_eq!(
        fx.read("backup.schedule_db").await.expect("schedule").value,
        json!("15 1 * * *"),
        "a field not in the request must not be touched"
    );
    assert_eq!(
        fx.read("backup.timezone").await.expect("timezone").value,
        json!("Europe/Berlin")
    );
}

/// Retention is numeric, so a string write has to be rejected rather than
/// stored: the handler reads these back with `as_u64()` and would silently fall
/// back to the default for a stored string.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_string_retention_value_is_rejected() {
    let fx = Fixture::new().await;

    let result = fx
        .repo
        .set_many(
            fx.tenant,
            fx.user,
            vec![UpdateSetting {
                key: "backup.retention.daily".into(),
                value: json!("seven"),
            }],
        )
        .await;

    assert!(
        result.is_err(),
        "a non-numeric retention must not be stored"
    );
}

/// Two tenants must not see each other's backup configuration.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn backup_config_is_tenant_scoped() {
    let a = Fixture::new().await;
    let b = Fixture::new().await;

    a.update("backup.schedule_db", json!("1 1 * * *")).await;
    b.update("backup.schedule_db", json!("2 2 * * *")).await;

    assert_eq!(
        a.read("backup.schedule_db").await.expect("a").value,
        json!("1 1 * * *")
    );
    assert_eq!(
        b.read("backup.schedule_db").await.expect("b").value,
        json!("2 2 * * *"),
        "tenant B must see its own value, not A's"
    );

    // Resetting A leaves B untouched.
    assert!(
        a.repo
            .reset(a.tenant, "backup.schedule_db")
            .await
            .expect("reset")
    );
    assert_eq!(
        a.read("backup.schedule_db").await.expect("a default").value,
        json!("0 2 * * *"),
        "reset must fall back to the shipped default"
    );
    assert_eq!(
        b.read("backup.schedule_db").await.expect("b").value,
        json!("2 2 * * *")
    );
    let _ = &a.pool;
    let _ = &b.pool;
}
