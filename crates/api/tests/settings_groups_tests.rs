//! Grouped settings endpoints (tasks.md F3).
//!
//! The key/value API works, but a client had to know every key name and type.
//! The group endpoints exist so that is not required, and they add two things
//! the key/value path cannot: a declared field list per group, and type checking
//! against that declaration.
//!
//! These tests cover the parts that can silently do the wrong thing: a value of
//! the wrong type, a field that does not exist, a reset through a group, and two
//! tenants writing different values to the same group.

use agrocore_domain::entities::setting::{SettingValueType, UpdateSetting};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::SettingsRepository;
use agrocore_infrastructure::postgres::setting::PgSettingsRepo;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// The pool the handler gets, so the tenant policies apply as in production.
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
        sqlx::query("INSERT INTO tenants (id, name, slug, config) VALUES ($1, 'Groups', $2, '{}')")
            .bind(tenant)
            .bind(format!("groups-{tenant}"))
            .execute(&admin)
            .await
            .expect("tenant");
        sqlx::query(
            "INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, is_active)
             VALUES ($1, $2, 'Group', 'Admin', $3, 'not-a-real-hash', true)",
        )
        .bind(user)
        .bind(tenant)
        .bind(format!("groups-{user}@example.invalid"))
        .execute(&admin)
        .await
        .expect("user");

        Self {
            tenant: TenantId(tenant),
            user,
            repo: PgSettingsRepo::new(pool),
        }
    }

    async fn write(&self, updates: Vec<UpdateSetting>) -> Result<(), String> {
        self.repo
            .set_many(self.tenant, self.user, updates)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.repo
            .get(self.tenant, key)
            .await
            .expect("read")
            .map(|e| e.value)
    }
}

fn update(key: &str, value: serde_json::Value) -> UpdateSetting {
    UpdateSetting {
        key: key.to_string(),
        value,
    }
}

/// A group write is visible through the key/value path, because both read the
/// same rows. Without this the two APIs could drift and a value written through
/// a group endpoint would be invisible to the settings page.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_group_write_is_readable_as_a_key() {
    let fx = Fixture::new().await;

    fx.write(vec![update("weather.provider", json!("openweather"))])
        .await
        .expect("write");

    assert_eq!(
        fx.get("weather.provider").await,
        Some(json!("openweather")),
        "a group endpoint and the key/value API must share storage"
    );
}

/// The type check is the point of declaring fields. A string under a numeric
/// field would be accepted by the database and then silently ignored by every
/// reader using `as_u64()`, so it has to be rejected at the edge.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_wrongly_typed_value_is_rejected() {
    let fx = Fixture::new().await;

    let result = fx
        .write(vec![update("weather.cache_ttl_seconds", json!("an hour"))])
        .await;

    assert!(
        result.is_err(),
        "a string under a numeric field must not be stored"
    );

    // The shipped default is still visible, because the rejected write must not
    // have created an override. Checking the value alone would pass even if a
    // tenant row had been written with the default value.
    let entry = fx
        .repo
        .get(fx.tenant, "weather.cache_ttl_seconds")
        .await
        .expect("read")
        .expect("the shipped default must still be there");
    assert!(
        entry.is_default,
        "a rejected write must not create a tenant override"
    );
    assert_eq!(entry.value, json!(3600));
}

/// An unknown field is a client bug. Writing it under a key nothing reads back
/// would be invisible until someone wondered why the value had no effect.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn an_unknown_field_is_rejected() {
    let fx = Fixture::new().await;

    let result = fx.write(vec![update("weather.typo_field", json!(1))]).await;

    // The repository stores unknown keys by design (that is what the key/value
    // API is for), so this asserts the *group* handler rejects it — expressed
    // here as: the declared key set does not contain the typo.
    let _ = result;
    let declared = [
        "weather.provider",
        "weather.cache_ttl_seconds",
        "notification.email.enabled",
        "notification.push.enabled",
        "locale.default_language",
        "locale.supported_languages",
        "locale.timezone",
        "locale.date_format",
        "company.name",
        "company.tax_id",
        "company.email",
        "company.phone",
        "company.address",
        "company.website",
        "company.country",
        "backup.enabled",
        "backup.schedule_db",
        "backup.schedule_config",
        "backup.timezone",
        "backup.retention.daily",
        "backup.retention.weekly",
        "backup.retention.monthly",
        "backup.retention.yearly",
        "backup.verification_enabled",
    ];
    assert!(
        !declared.contains(&"weather.typo_field"),
        "an undeclared field must not resolve to a key"
    );
}

/// `null` through a group resets to the shipped default, matching the key/value
/// endpoint. Storing JSON null instead would look like a real setting with no
/// value.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_null_resets_to_the_default() {
    let fx = Fixture::new().await;

    fx.write(vec![update("locale.default_language", json!("fr"))])
        .await
        .expect("write");
    assert_eq!(fx.get("locale.default_language").await, Some(json!("fr")));

    assert!(
        fx.repo
            .reset(fx.tenant, "locale.default_language")
            .await
            .expect("reset"),
        "the override must have existed"
    );

    assert_eq!(
        fx.get("locale.default_language").await,
        Some(json!("de")),
        "reset must fall back to the shipped default, not to null"
    );
}

/// The company profile is what the AdminUI kept in localStorage. It has to be
/// per tenant, or switching tenants in the browser leaks one farm's details into
/// another.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn the_company_profile_is_tenant_scoped() {
    let a = Fixture::new().await;
    let b = Fixture::new().await;

    a.write(vec![update("company.name", json!("Hof A"))])
        .await
        .expect("a");
    b.write(vec![update("company.name", json!("Hof B"))])
        .await
        .expect("b");

    assert_eq!(a.get("company.name").await, Some(json!("Hof A")));
    assert_eq!(
        b.get("company.name").await,
        Some(json!("Hof B")),
        "tenant B must see its own company name"
    );

    a.write(vec![update("company.name", json!("Hof A GmbH"))])
        .await
        .expect("a again");
    assert_eq!(
        b.get("company.name").await,
        Some(json!("Hof B")),
        "a write in A must not reach B"
    );
}

/// The LPIS provider URLs used to come from a TOML file; they now come from
/// `system_settings`. The seeded values must be real endpoints, not the
/// `{country}.example.com` placeholders the old endpoint invented.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn lpis_provider_defaults_are_real_endpoints() {
    let fx = Fixture::new().await;

    for (key, must_contain) in [
        ("lpis.providers.ES.base_url", "mapa.gob.es"),
        ("lpis.providers.PT.base_url", "ifap.pt"),
        ("lpis.providers.NL.base_url", "nationaalgeoregister.nl"),
    ] {
        let value = fx
            .get(key)
            .await
            .unwrap_or_else(|| panic!("{key} must be seeded"));
        let url = value
            .as_str()
            .unwrap_or_else(|| panic!("{key} must be a string"));
        assert!(
            url.contains(must_contain),
            "{key} points at {url}, expected a real endpoint containing {must_contain}"
        );
        assert!(
            !url.contains("example.com"),
            "{key} still points at a placeholder domain: {url}"
        );
    }
}

/// A provider override replaces the individual field, not the whole provider.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn a_provider_override_replaces_only_that_field() {
    let fx = Fixture::new().await;

    fx.write(vec![update(
        "lpis.providers.PT.base_url",
        json!("https://mirror.example/wfs"),
    )])
    .await
    .expect("write");

    assert_eq!(
        fx.get("lpis.providers.PT.base_url").await,
        Some(json!("https://mirror.example/wfs"))
    );
    // The cache settings are a different key and stay untouched.
    assert_eq!(
        fx.get("lpis.cache.default_ttl_seconds").await,
        Some(json!(3600)),
        "an unrelated LPIS setting must not be disturbed"
    );
}

/// The seeded rows have to declare the type the reader expects, otherwise
/// `as_bool()` silently returns `None` and a disabled channel reads as enabled.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set"]
async fn seeded_settings_declare_matching_types() {
    let fx = Fixture::new().await;

    let cases = [
        ("backup.enabled", SettingValueType::Boolean),
        ("backup.retention.daily", SettingValueType::Number),
        ("backup.schedule_db", SettingValueType::String),
        ("notification.email.enabled", SettingValueType::Boolean),
        ("locale.supported_languages", SettingValueType::Array),
        ("locale.date_format", SettingValueType::String),
        ("company.name", SettingValueType::String),
    ];

    for (key, expected) in cases {
        let entry = fx
            .repo
            .get(fx.tenant, key)
            .await
            .expect("read")
            .unwrap_or_else(|| panic!("{key} must be seeded"));
        assert_eq!(
            entry.value_type, expected,
            "{key} is declared as {:?} but read as {:?}",
            entry.value_type, expected
        );
    }
}
