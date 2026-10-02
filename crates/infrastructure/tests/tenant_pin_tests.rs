//! Tenant-pin tests for `TenantPool` (tasks.md A3, pin integration).
//!
//! `TenantPool` exists so a repository cannot obtain a connection without
//! `app.current_tenant_id` being set. These tests check that the pin is applied
//! to the connection that actually runs the query, that it survives a
//! transaction, and that a pool checkout cannot inherit a previous tenant.
//!
//! All tests need a database with the full migration set and
//! `AGROCORE_RLS_ENABLED=1`, so they are `#[ignore]`d and run explicitly:
//!
//! ```text
//! DATABASE_URL=... AGROCORE_RLS_ENABLED=1 \
//!   cargo test -p agrocore-infrastructure --test tenant_pin_tests -- --ignored
//! ```

use agrocore_infrastructure::TenantPool;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Namespace per test, so each gets its own tenants and rows and cannot collide
/// with another test's fixture when the suite runs.
///
/// `run_nonce` offsets the namespace per process. Without it a run that left
/// rows behind — the tests share one database, and a failed cleanup leaves the
/// fixture in place — collides on tenants_pkey and tenants_slug_key on the next
/// run. `ON CONFLICT DO NOTHING` is not a workaround: the ids are what the
/// assertions read, so reusing a stale row would test the wrong data.
fn run_nonce() -> u128 {
    // Computed once per process: `tenant()` is called from both the fixture and
    // the assertion, and a nonce that changed between the two calls would give
    // them different ids — the fixture would seed one tenant while the test read
    // another.
    static NONCE: std::sync::OnceLock<u128> = std::sync::OnceLock::new();
    *NONCE.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
            % 0x1000
    })
}

fn tenant(slot: u128, which: u128) -> Uuid {
    Uuid::from_u128(0x7000_0000_0000_4000_8000_0000_0000_0000 | ((run_nonce() ^ slot) << 8) | which)
}

fn site_id(slot: u128, which: u128) -> Uuid {
    Uuid::from_u128(0x7100_0000_0000_4000_8000_0000_0000_0000 | ((run_nonce() ^ slot) << 8) | which)
}

/// Build the pool exactly the way the service does, so the test exercises the
/// real `SET ROLE` path rather than a privileged superuser connection. A direct
/// `PgPool::connect` bypasses `after_connect`, and `agrocore` is superuser, so
/// the policies would never be evaluated and every isolation assertion here
/// would pass vacuously.
async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE agrocore_app")
                    .execute(conn)
                    .await
                    .map(|_| ())
                    .map_err(|e| sqlx::Error::Configuration(Box::new(e)))
            })
        })
        .connect(&url)
        .await
        .expect("connect")
}

/// A privileged pool, for fixture writes that must bypass the policies.
/// Fixtures run as superuser so they can create tenants and rows regardless of
/// the pin; the assertions then read back through the pinned pool.
fn admin_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect_lazy(&url).expect("lazy admin pool")
}

async fn seed(slot: u128, which: u128, label: &'static str) {
    let pool = &admin_pool();
    let tid = tenant(slot, which);
    // The slug is UNIQUE and the label is fixed per test, so it gets the same
    // per-run suffix as the id.
    let slug = format!("{label}-{:x}", tid.as_u128() % 0xffff_ffff);
    sqlx::query("INSERT INTO tenants (id, name, slug, is_active) VALUES ($1, $2, $3, true)")
        .bind(tid)
        .bind(label)
        .bind(&slug)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("insert tenant {label}: {e}"));

    sqlx::query("INSERT INTO sites (id, tenant_id, label, is_active) VALUES ($1, $2, $3, true)")
        .bind(site_id(slot, which))
        .bind(tid)
        .bind(label)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("insert site {label}: {e}"));
}

async fn labels(pool: &TenantPool) -> Vec<String> {
    sqlx::query_scalar("SELECT label FROM sites ORDER BY label")
        .fetch_all(pool)
        .await
        .expect("labels through pinned pool")
}

/// The pin must be visible to the very next query on that connection.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn pin_is_visible_to_the_query_it_protects() {
    let raw = pool().await;
    seed(1, 1, "pin-a").await;
    let a = tenant(1, 1);

    let pinned = TenantPool::new(&raw, a);
    let rows: Vec<String> = sqlx::query_scalar("SELECT label FROM sites WHERE tenant_id = $1")
        .bind(a)
        .fetch_all(&pinned)
        .await
        .expect("query through pinned pool");

    assert_eq!(rows, vec!["pin-a"], "pinned tenant must see its own row");
}

/// A pooled connection that previously served tenant A must not leak that
/// state to tenant B. This is the whole reason the pin is re-applied per query.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn checkout_does_not_inherit_previous_tenant() {
    let raw = pool().await;
    seed(2, 1, "pin-b-a").await;
    seed(2, 2, "pin-b-b").await;

    let a = TenantPool::new(&raw, tenant(2, 1));
    let b = TenantPool::new(&raw, tenant(2, 2));

    // Interleave so both are likely to land on the same pooled connection.
    let a1 = labels(&a).await;
    let b1 = labels(&b).await;
    let a2 = labels(&a).await;

    assert_eq!(a1, vec!["pin-b-a"], "tenant A sees only A");
    assert_eq!(b1, vec!["pin-b-b"], "tenant B sees only B");
    assert_eq!(
        a2, a1,
        "reusing a connection previously pinned to B must not give B's rows to A"
    );
}

/// `begin()` must pin for the whole transaction. `SET LOCAL` outside a
/// transaction block is a no-op, so this is the regression that would silently
/// leave the pin unset for every statement after the first.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn transaction_stays_pinned_across_statements() {
    let raw = pool().await;
    let a = tenant(3, 1);
    seed(3, 1, "pin-c").await;

    let pinned = TenantPool::new(&raw, a);
    let mut tx = pinned.begin().await.expect("begin");

    // Several statements inside one transaction, each of which must still see
    // the pin.
    for _ in 0..3 {
        let seen: Option<String> =
            sqlx::query_scalar("SELECT current_setting('app.current_tenant_id', true)")
                .fetch_one(&mut *tx)
                .await
                .expect("read setting inside tx");
        assert_eq!(
            seen.as_deref(),
            Some(a.to_string().as_str()),
            "pin must survive every statement in the transaction"
        );

        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites WHERE tenant_id = $1")
            .bind(a)
            .fetch_one(&mut *tx)
            .await
            .expect("count inside tx");
        assert_eq!(n, 1, "pinned transaction must see only its own rows");
    }

    tx.rollback().await.expect("rollback");
}

/// The unscoped handle matches no tenant, so it reads nothing. That is the safe
/// direction for bootstrap work: deny rather than leak.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn unscoped_pool_denies_everything() {
    let raw = pool().await;
    seed(4, 1, "pin-d").await;

    let unscoped = TenantPool::unscoped(&raw);
    let rows: Vec<String> = sqlx::query_scalar("SELECT label FROM sites")
        .fetch_all(&unscoped)
        .await
        .expect("query through unscoped pool");

    assert!(
        rows.is_empty(),
        "unscoped pool must not read tenant rows, got {rows:?}"
    );
}

/// `set_tenant` also clears `app.is_superadmin`, so a connection that served an
/// administrative query cannot carry elevated reads into the next checkout.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn pin_clears_superadmin_flag() {
    let raw = pool().await;
    let mut conn = raw.acquire().await.expect("acquire");

    sqlx::query("SELECT set_config('app.is_superadmin', 'true', false)")
        .execute(&mut *conn)
        .await
        .expect("raise superadmin");

    let before: Option<String> =
        sqlx::query_scalar("SELECT current_setting('app.is_superadmin', true)")
            .fetch_one(&mut *conn)
            .await
            .expect("read flag before pin");
    assert_eq!(
        before.as_deref(),
        Some("true"),
        "precondition: superadmin flag is set on this connection"
    );

    let a = tenant(6, 1);
    agrocore_infrastructure::set_tenant(&mut conn, a)
        .await
        .expect("pin");

    let flag: Option<String> =
        sqlx::query_scalar("SELECT current_setting('app.is_superadmin', true)")
            .fetch_one(&mut *conn)
            .await
            .expect("read flag");
    assert_eq!(flag.as_deref(), Some("false"), "pin must clear superadmin");

    let seen: Option<String> =
        sqlx::query_scalar("SELECT current_setting('app.current_tenant_id', true)")
            .fetch_one(&mut *conn)
            .await
            .expect("read tenant");
    assert_eq!(seen.as_deref(), Some(a.to_string().as_str()));
}

/// A `fetch_optional` that matches nothing must return `None`, not an error -
/// the default `Executor` body derives this from `fetch_many`, and the pin has
/// to work through that path too.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn fetch_optional_returns_none_for_foreign_rows() {
    let raw = pool().await;
    seed(5, 1, "pin-e-a").await;
    seed(5, 2, "pin-e-b").await;

    let pinned = TenantPool::new(&raw, tenant(5, 2));
    let row: Option<String> = sqlx::query_scalar("SELECT label FROM sites WHERE label = 'pin-e-a'")
        .fetch_optional(&pinned)
        .await
        .expect("fetch_optional through pinned pool");

    assert!(
        row.is_none(),
        "tenant B must not read tenant A's row through the pin"
    );
}

/// The unscoped handle must carry the nil tenant, which matches no tenant and
/// therefore denies everything; a pinned handle must report its own tenant.
///
/// `#[ignore]`d like the rest of this file because it needs a real
/// `DATABASE_URL`. Building even a lazy pool touches Tokio internals that a
/// database-less test run does not provide.
#[tokio::test]
#[ignore = "needs DATABASE_URL and AGROCORE_RLS_ENABLED=1"]
async fn tenant_pool_is_exported() {
    let raw = pool().await;
    assert_eq!(
        TenantPool::unscoped(&raw).tenant_id(),
        Uuid::nil(),
        "unscoped matches no tenant"
    );
    assert_eq!(
        TenantPool::new(&raw, tenant(7, 1)).tenant_id(),
        tenant(7, 1),
        "a pinned handle reports its own tenant"
    );
}
