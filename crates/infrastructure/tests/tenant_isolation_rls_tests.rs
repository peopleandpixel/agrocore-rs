//! Tenant isolation under the role the application actually uses (tasks.md J21).
//!
//! # What was missing
//!
//! `tenant_isolation_tests.rs` already checks that repository queries carry a
//! `tenant_id = $2` predicate. That is a statement about the SQL text, and it is
//! worth having — but it connects as the migration role, which is a superuser,
//! and a superuser bypasses row-level security entirely. Every assertion in that
//! file therefore passes whether or not RLS is active, which means it verifies the
//! query and not the isolation.
//!
//! This file checks the thing that actually protects one tenant's data from
//! another: the policy, under the role the server uses.
//!
//! # The distinction the tests turn on
//!
//! Two connections, same database, same rows:
//!
//!   * `super_pool` — the migration role, which can see and write everything;
//!   * `rls_pool` — the same pool with `SET ROLE agrocore_app`, which is subject to
//!     the policies and needs `app.current_tenant_id` pinned per connection.
//!
//! An isolation test is only meaningful if the RLS pool is *provably* subject to
//! RLS. `rls_is_actually_active` establishes that first: it writes a row as tenant
//! A, then checks that a plain `SELECT` under the app role without a tenant pin
//! returns nothing. If that ever returns the row, the isolation assertions below
//! prove nothing, so the test fails there instead of passing quietly.
//!
//! Run against a real database:
//!
//! ```bash
//! DATABASE_URL=postgresql://agrocore:***@localhost:5432/agrocore \
//! AGROCORE_RLS_ENABLED=1 \
//!   cargo test -p agrocore-infrastructure --test tenant_isolation_rls_tests -- --ignored
//! ```

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Unique tenant ids per tag.
///
/// The tests share a database and cargo runs them in parallel, so fixed ids would
/// let one test's cleanup delete another test's rows. The run timestamp keeps a
/// run that left rows behind from colliding on the primary key next time; with
/// FORCE row-level security active, `ON CONFLICT DO NOTHING` is not a usable
/// workaround because it triggers a policy check the plain INSERT does not.
fn tenants(tag: u128) -> (Uuid, Uuid) {
    let run = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let base = 0xe200_0000_0000_0000_0000_0000_0000_0000u128 + tag * 2 + (run % 0x1000) * 2;
    (Uuid::from_u128(base), Uuid::from_u128(base + 1))
}

/// The connection the migration role uses: no policy applies.
async fn super_pool(url: &str) -> Option<PgPool> {
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await
        .ok()
}

/// The connection the application uses: `agrocore_app`, subject to RLS.
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

/// Pin the tenant for the current transaction.
///
/// `set_config(..., false)` is session-scoped, so a pool that returns this
/// connection to a later request would carry the previous tenant's pin. That is
/// why the isolation tests below pin and read on the *same* acquired connection
/// instead of going through the pool.
async fn pin(conn: &mut sqlx::PgConnection, tenant: Uuid) {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, false)")
        .bind(tenant.to_string())
        .execute(&mut *conn)
        .await
        .expect("pin tenant");
}

/// Seed two tenants and one site each.
///
/// The sites are written through the migration role, which is why the seed does
/// not need a pin; the read paths under test do.
async fn seed(pool: &PgPool, tenant_a: Uuid, tenant_b: Uuid, tag: &str) {
    let suffix = tenant_a.as_u128() % 0xffff_ffff;
    for (id, name, slug) in [
        (
            tenant_a,
            format!("Iso A {tag}"),
            format!("iso-a-{tag}-{suffix:x}"),
        ),
        (
            tenant_b,
            format!("Iso B {tag}"),
            format!("iso-b-{tag}-{suffix:x}"),
        ),
    ] {
        sqlx::query("INSERT INTO tenants (id, name, slug, is_active) VALUES ($1, $2, $3, true)")
            .bind(id)
            .bind(&name)
            .bind(&slug)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("seed tenant {slug}: {e}"));
    }

    for (id, tenant, label) in [
        (
            Uuid::from_u128(tenant_a.as_u128() ^ 0x6000),
            tenant_a,
            format!("Iso site A {tag}"),
        ),
        (
            Uuid::from_u128(tenant_b.as_u128() ^ 0x6000),
            tenant_b,
            format!("Iso site B {tag}"),
        ),
    ] {
        sqlx::query(
            "INSERT INTO sites (id, tenant_id, label, is_active, plots) \
             VALUES ($1, $2, $3, true, '[]')",
        )
        .bind(id)
        .bind(tenant)
        .bind(&label)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("seed site {label}: {e}"));
    }
}

/// Remove the seeded rows.
///
/// The tenants table has FORCE row-level security, so a cleanup that silently
/// fails leaves rows behind and the next run collides on the slug. Deleting the
/// sites first keeps that from happening; the tenants are removed through the
/// migration role for the same reason.
async fn cleanup(pool: &PgPool, tenant_a: Uuid, tenant_b: Uuid) {
    for tenant in [tenant_a, tenant_b] {
        let _ = sqlx::query("DELETE FROM sites WHERE tenant_id = $1")
            .bind(tenant)
            .execute(pool)
            .await;
    }
    for tenant in [tenant_a, tenant_b] {
        let deleted = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant)
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);
        assert_eq!(
            deleted, 1,
            "tenant {tenant} was not deleted; a leftover row will collide with the \
             next run on tenants_slug_key"
        );
    }
}

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok().filter(|u| !u.is_empty())
}

/// The RLS pool has to be subject to RLS for any assertion about isolation to
/// mean anything.
///
/// This is the load-bearing check of the whole file. It writes a row as tenant A
/// and then reads it back through the app role with no tenant pinned. If the row
/// comes back, the policies are not being enforced for this role and every other
/// test here would pass while isolation was broken.
#[tokio::test]
#[ignore = "requires a database with migrations applied"]
async fn rls_is_actually_active_for_the_app_role() {
    let Some(url) = database_url() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    let (Some(admin), Some(app)) = (super_pool(&url).await, rls_pool(&url).await) else {
        eprintln!("cannot connect; skipping");
        return;
    };

    let (tenant_a, tenant_b) = tenants(1);
    seed(&admin, tenant_a, tenant_b, "active").await;

    let mut conn = app.acquire().await.expect("acquire app connection");

    // No pin: the session is not acting as any tenant.
    let leaked: i64 = sqlx::query_scalar("SELECT count(*) FROM sites WHERE tenant_id = $1")
        .bind(tenant_a)
        .fetch_one(&mut *conn)
        .await
        .expect("unpinned read");

    assert_eq!(
        leaked, 0,
        "a SELECT under agrocore_app with no tenant pin returned {leaked} of tenant A's \
         rows. Row-level security is not being enforced for this role, so the \
         isolation assertions in this file would pass while isolation was broken."
    );

    // With a pin the same query must see the row — otherwise the check above
    // would also pass on a table with no policy at all.
    pin(&mut conn, tenant_a).await;
    let visible: i64 = sqlx::query_scalar("SELECT count(*) FROM sites WHERE tenant_id = $1")
        .bind(tenant_a)
        .fetch_one(&mut *conn)
        .await
        .expect("pinned read");

    assert_eq!(
        visible, 1,
        "with tenant A pinned, tenant A's own site should be visible; getting {visible} \
         means the policy is filtering too much rather than filtering correctly"
    );

    drop(conn);
    cleanup(&admin, tenant_a, tenant_b).await;
}

/// Tenant A sees its own rows and none of tenant B's.
#[tokio::test]
#[ignore = "requires a database with migrations applied"]
async fn a_pinned_tenant_sees_only_its_own_rows() {
    let Some(url) = database_url() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    let (Some(admin), Some(app)) = (super_pool(&url).await, rls_pool(&url).await) else {
        eprintln!("cannot connect; skipping");
        return;
    };

    let (tenant_a, tenant_b) = tenants(2);
    seed(&admin, tenant_a, tenant_b, "pin").await;

    let mut conn = app.acquire().await.expect("acquire app connection");
    pin(&mut conn, tenant_a).await;

    // A full listing, not a filtered one. If the policy on `sites` is right, this
    // returns exactly tenant A's rows; a query with a WHERE clause would pass
    // even with no policy, because the predicate does the filtering itself.
    let all_labels: Vec<String> = sqlx::query_scalar("SELECT label FROM sites ORDER BY label")
        .fetch_all(&mut *conn)
        .await
        .expect("pinned listing");

    assert!(
        all_labels.iter().any(|l| l.contains("Iso site A")),
        "tenant A pinned but its own site is missing from the listing: {all_labels:?}"
    );
    assert!(
        !all_labels.iter().any(|l| l.contains("Iso site B")),
        "tenant A sees tenant B's rows: {all_labels:?}"
    );

    // And the other direction, on a separate connection so the pin is not reused.
    let mut other = app.acquire().await.expect("acquire second connection");
    pin(&mut other, tenant_b).await;
    let b_labels: Vec<String> = sqlx::query_scalar("SELECT label FROM sites ORDER BY label")
        .fetch_all(&mut *other)
        .await
        .expect("pinned listing for B");
    assert!(
        !b_labels.iter().any(|l| l.contains("Iso site A")),
        "tenant B sees tenant A's rows: {b_labels:?}"
    );
    assert!(
        b_labels.iter().any(|l| l.contains("Iso site B")),
        "tenant B pinned but its own site is missing: {b_labels:?}"
    );

    drop(conn);
    drop(other);
    cleanup(&admin, tenant_a, tenant_b).await;
}

/// A write against another tenant's row is rejected, not silently ignored.
#[tokio::test]
#[ignore = "requires a database with migrations applied"]
async fn a_pinned_tenant_cannot_write_another_tenants_row() {
    let Some(url) = database_url() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    let (Some(admin), Some(app)) = (super_pool(&url).await, rls_pool(&url).await) else {
        eprintln!("cannot connect; skipping");
        return;
    };

    let (tenant_a, tenant_b) = tenants(3);
    seed(&admin, tenant_a, tenant_b, "write").await;

    let site_b = Uuid::from_u128(tenant_b.as_u128() ^ 0x6000);
    let mut conn = app.acquire().await.expect("acquire app connection");
    pin(&mut conn, tenant_a).await;

    // An UPDATE that names tenant B's row directly. The WHERE clause is the one a
    // careless handler would write; the policy is the one that has to stop it.
    let updated = sqlx::query("UPDATE sites SET label = 'hijacked' WHERE id = $1")
        .bind(site_b)
        .execute(&mut *conn)
        .await;

    // Either outcome is correct: a permission error, or zero rows because the
    // policy made the row invisible. What must not happen is a successful write.
    if let Ok(result) = updated {
        assert_eq!(
            result.rows_affected(),
            0,
            "updating tenant B's site as tenant A reported {} rows affected; the policy \
             should have made the row invisible",
            result.rows_affected()
        );
    }

    // The row is unchanged, whichever way the write was rejected.
    let label: String = sqlx::query_scalar("SELECT label FROM sites WHERE id = $1")
        .bind(site_b)
        .fetch_one(&admin)
        .await
        .expect("read back tenant B's site");
    assert!(
        !label.contains("hijacked"),
        "tenant B's site was modified by a request pinned as tenant A: {label}"
    );

    // And tenant A can still write its own row, or the policy is filtering too
    // much rather than correctly.
    let site_a = Uuid::from_u128(tenant_a.as_u128() ^ 0x6000);
    let own = sqlx::query("UPDATE sites SET label = 'mine' WHERE id = $1")
        .bind(site_a)
        .execute(&mut *conn)
        .await
        .expect("a tenant must be able to write its own row");
    assert_eq!(
        own.rows_affected(),
        1,
        "tenant A could not write its own row"
    );

    drop(conn);
    cleanup(&admin, tenant_a, tenant_b).await;
}

/// A delete aimed at another tenant's rows affects nothing.
#[tokio::test]
#[ignore = "requires a database with migrations applied"]
async fn a_pinned_tenant_cannot_delete_another_tenants_row() {
    let Some(url) = database_url() else {
        eprintln!("DATABASE_URL not set; skipping");
        return;
    };
    let (Some(admin), Some(app)) = (super_pool(&url).await, rls_pool(&url).await) else {
        eprintln!("cannot connect; skipping");
        return;
    };

    let (tenant_a, tenant_b) = tenants(4);
    seed(&admin, tenant_a, tenant_b, "delete").await;

    let mut conn = app.acquire().await.expect("acquire app connection");
    pin(&mut conn, tenant_a).await;

    let _ = sqlx::query("DELETE FROM sites WHERE tenant_id = $1")
        .bind(tenant_b)
        .execute(&mut *conn)
        .await;

    // Read through the migration role, which is not filtered.
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM sites WHERE tenant_id = $1")
        .bind(tenant_b)
        .fetch_one(&admin)
        .await
        .expect("count tenant B's sites");
    assert_eq!(
        remaining, 1,
        "tenant B's site was deleted by a request pinned as tenant A"
    );

    drop(conn);
    cleanup(&admin, tenant_a, tenant_b).await;
}
