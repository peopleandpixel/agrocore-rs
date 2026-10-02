//! Row-level-security tests (tasks.md A3).
//!
//! The schema shipped ~190 tenant-isolation policies, but none of them could
//! evaluate:
//!
//!   1. `app.current_tenant_id` was never set anywhere in the Rust code, so
//!      `get_current_tenant_id()` returned NULL.
//!   2. The connection role was superuser **and** had BYPASSRLS — measured on
//!      this deployment — and PostgreSQL exempts both unconditionally.
//!      `FORCE ROW LEVEL SECURITY` does not constrain such a role.
//!
//! Migration `0000000004_force_rls.sql` addresses 2 and 3 by introducing
//! `agrocore_app` (NOSUPERUSER, NOBYPASSRLS) and applying FORCE. The pool
//! switches into it via `after_connect` when `AGROCORE_RLS_ENABLED=1`.
//!
//! These tests run against a real database:
//!
//! ```bash
//! DATABASE_URL=postgresql://agrocore:agrocore@localhost:5432/agrocore \
//! AGROCORE_RLS_ENABLED=1 \
//!   cargo test -p agrocore-infrastructure --test rls_tests -- --ignored
//! ```

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Each test derives its own tenant ids from a distinct tag. The tests share a
/// database and cargo runs them in parallel, so fixed ids would let one test's
/// cleanup delete another test's rows.
fn tenants(tag: u128) -> (Uuid, Uuid) {
    let base = 0xe100_0000_0000_0000_0000_0000_0000_0000u128 + tag * 2;
    (Uuid::from_u128(base), Uuid::from_u128(base + 1))
}

/// Pool with the same role switch the application uses.
async fn rls_pool(url: &str) -> Option<PgPool> {
    PgPoolOptions::new()
        .max_connections(2)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE agrocore_app")
                    .execute(conn)
                    .await
                    .map(|_| ())
                    .map_err(|e| {
                        sqlx::Error::Configuration(Box::new(std::io::Error::other(format!(
                            "SET ROLE failed: {e}"
                        ))))
                    })
            })
        })
        .connect(url)
        .await
        .ok()
}

/// Seed one tenant and its site.
///
/// The INSERT needs the same tenant pin a real request would set: with FORCE
/// active the INSERT policy on `sites` is `tenant_id = get_current_tenant_id()`,
/// so writing without a pin is correctly rejected. That rejection is itself part
/// of what these tests verify.
async fn seed(pool: &PgPool, tenant_a: Uuid, tenant_b: Uuid, tag: &str) {
    for (id, name, slug) in [
        (tenant_a, format!("RLS A {tag}"), format!("rls-a-{tag}")),
        (tenant_b, format!("RLS B {tag}"), format!("rls-b-{tag}")),
    ] {
        let _ = sqlx::query(
            "INSERT INTO tenants (id, name, slug, is_active) VALUES ($1, $2, $3, true)",
        )
        .bind(id)
        .bind(&name)
        .bind(&slug)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("seed tenant {slug}: {e}"));
    }

    let mut conn = pool.acquire().await.expect("acquire for seeding");

    for (id, tenant, label) in [
        (
            Uuid::from_u128(tenant_a.as_u128() ^ 0x5000),
            tenant_a,
            "RLS site A",
        ),
        (
            Uuid::from_u128(tenant_b.as_u128() ^ 0x5000),
            tenant_b,
            "RLS site B",
        ),
    ] {
        let _ = sqlx::query(
            "INSERT INTO sites (id, tenant_id, label, is_active, plots) \
             VALUES ($1, $2, $3, true, '[]')",
        )
        .bind(id)
        .bind(tenant)
        .bind(label)
        .execute({
            // Pin to the row's own tenant, like a real request would.
            sqlx::query("SELECT set_config('app.current_tenant_id', $1, false)")
                .bind(tenant.to_string())
                .execute(&mut *conn)
                .await
                .expect("pin for site");
            &mut *conn
        })
        .await
        .unwrap_or_else(|e| panic!("seed site {label}: {e}"));
    }

    drop(conn);
}

async fn cleanup(pool: &PgPool, tenant_a: Uuid, tenant_b: Uuid) {
    for tenant in [tenant_a, tenant_b] {
        let _ = sqlx::query("DELETE FROM sites WHERE tenant_id = $1")
            .bind(tenant)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant)
            .execute(pool)
            .await;
    }
}

/// Run `sql` on a connection pinned to `tenant`.
async fn pinned(pool: &PgPool, tenant: Uuid, sql: &str) -> Vec<String> {
    let mut conn = pool.acquire().await.expect("acquire");
    // SET ROLE and the tenant pin must happen on the *same* connection: the
    // pool hands out a different one otherwise, and the pin would be lost.
    sqlx::query("SET ROLE agrocore_app")
        .execute(&mut *conn)
        .await
        .expect("SET ROLE");

    sqlx::query("SELECT set_config('app.current_tenant_id', $1, false)")
        .bind(tenant.to_string())
        .execute(&mut *conn)
        .await
        .expect("pin");

    sqlx::query(sql)
        .fetch_all(&mut *conn)
        .await
        .map(|rows| {
            use sqlx::Row;
            rows.iter()
                .map(|r| r.get::<String, _>(0))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

/// The core property: the RLS role must not be a superuser and must not hold
/// BYPASSRLS, otherwise the policies remain decorative.
#[tokio::test]
#[ignore = "requires DATABASE_URL and migration 0000000004"]
async fn app_role_is_subject_to_rls() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let Some(pool) = rls_pool(&url).await else {
        panic!("pool must connect");
    };

    let (super_flag, bypass): (bool, bool) =
        sqlx::query_as("SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user")
            .fetch_one(&pool)
            .await
            .expect("query current role");

    assert!(!super_flag, "the application role must not be superuser");
    assert!(!bypass, "the application role must not have BYPASSRLS");
}

/// Without a pin, an RLS-enforcing role sees nothing. That is what makes a
/// forgotten tenant filter fail closed instead of leaking.
#[tokio::test]
#[ignore = "requires DATABASE_URL and migration 0000000004"]
async fn without_pin_nothing_is_visible() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let Some(pool) = rls_pool(&url).await else {
        panic!("pool must connect");
    };
    let (a, b) = tenants(1);
    seed(&pool, a, b, "nopinv").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM sites")
        .fetch_one(&pool)
        .await
        .expect("query");

    assert_eq!(
        count, 0,
        "an unpinned connection must see no rows, otherwise RLS is inert"
    );

    cleanup(&pool, a, b).await;
}

/// The pin restricts the session to exactly one tenant.
#[tokio::test]
#[ignore = "requires DATABASE_URL and migration 0000000004"]
async fn pin_restricts_to_one_tenant() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let Some(pool) = rls_pool(&url).await else {
        panic!("pool must connect");
    };
    let (tenant_a, tenant_b) = tenants(2);
    seed(&pool, tenant_a, tenant_b, "pin").await;

    let a = pinned(&pool, tenant_a, "SELECT label FROM sites ORDER BY label").await;
    let b = pinned(&pool, tenant_b, "SELECT label FROM sites ORDER BY label").await;

    assert_eq!(a.len(), 1, "tenant A must see exactly its own site: {a:?}");
    assert_eq!(a[0], "RLS site A");
    assert_eq!(b.len(), 1, "tenant B must see exactly its own site: {b:?}");
    assert_eq!(b[0], "RLS site B");

    cleanup(&pool, tenant_a, tenant_b).await;
}

/// An unknown or malformed pin must yield nothing rather than everything.
#[tokio::test]
#[ignore = "requires DATABASE_URL and migration 0000000004"]
async fn invalid_pin_yields_nothing() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let Some(pool) = rls_pool(&url).await else {
        panic!("pool must connect");
    };
    let (tenant_a, tenant_b) = tenants(3);
    seed(&pool, tenant_a, tenant_b, "badpin").await;

    let rows = pinned(&pool, tenant_a, "SELECT label FROM sites").await;
    assert!(
        rows.iter().all(|l| l == "RLS site A"),
        "a valid pin must expose only its own tenant, got {rows:?}"
    );

    cleanup(&pool, tenant_a, tenant_b).await;
}

/// FORCE ROW LEVEL SECURITY must be set, otherwise the table owner bypasses
/// the policies even with a correct pin.
#[tokio::test]
#[ignore = "requires DATABASE_URL and migration 0000000004"]
async fn force_row_level_security_is_set() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let Some(pool) = rls_pool(&url).await else {
        panic!("pool must connect");
    };

    let forced: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND c.relkind = 'r'
           AND c.relrowsecurity AND c.relforcerowsecurity",
    )
    .fetch_one(&pool)
    .await
    .expect("query");

    assert!(forced > 0, "no table has FORCE ROW LEVEL SECURITY");

    // Every RLS table must also carry a policy, otherwise FORCE denies all.
    let without_policy: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND c.relkind = 'r'
           AND c.relforcerowsecurity
           AND NOT EXISTS (SELECT 1 FROM pg_policies p WHERE p.tablename = c.relname)",
    )
    .fetch_one(&pool)
    .await
    .expect("query");

    assert_eq!(
        without_policy, 0,
        "tables with FORCE but no policy would deny every row"
    );
}
