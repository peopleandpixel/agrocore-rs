//! Tenant-isolation tests for the repositories (tasks.md B1, J21).
//!
//! `kelter_delivery.rs` accepted `tid: TenantId` in all seven methods and used
//! it in none of the queries: `find_all` returned every tenant's rows and
//! `delete` removed any tenant's row. The signature looked correct, so nothing
//! flagged it — only the missing `tenant_id` column in
//! `test_tenant_scoped_tables_have_tenant_id` did.
//!
//! These tests exercise the SQL shapes directly against a real database. They
//! require `DATABASE_URL` and the migrations to have been applied:
//!
//! ```bash
//! DATABASE_URL=postgresql://agrocore:agrocore@localhost:5432/agrocore \
//!   cargo test -p agrocore-infrastructure --test tenant_isolation_tests -- --ignored
//! ```

use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

/// Two tenants, two sites, two vineyards, two deliveries — nothing shared.
struct Fixture {
    pool: sqlx::PgPool,
    tenant_a: Uuid,
    tenant_b: Uuid,
    delivery_a: Uuid,
    vineyard_a: Uuid,
    vineyard_b: Uuid,
}

impl Fixture {
    async fn new() -> Option<Self> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .ok()?;

        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let site_a = Uuid::new_v4();
        let site_b = Uuid::new_v4();
        let vineyard_a = Uuid::new_v4();
        let vineyard_b = Uuid::new_v4();
        let delivery_a = Uuid::new_v4();
        let delivery_b = Uuid::new_v4();

        for (id, name, slug) in [
            (tenant_a, "Iso A", &format!("iso-a-{}", tenant_a.simple())),
            (tenant_b, "Iso B", &format!("iso-b-{}", tenant_b.simple())),
        ] {
            sqlx::query(
                "INSERT INTO tenants (id, name, slug, is_active) VALUES ($1, $2, $3, true)",
            )
            .bind(id)
            .bind(name)
            .bind(slug)
            .execute(&pool)
            .await
            .ok()?;
        }

        for (id, tenant, label) in [
            (site_a, tenant_a, "Iso site A"),
            (site_b, tenant_b, "Iso site B"),
        ] {
            sqlx::query(
                "INSERT INTO sites (id, tenant_id, label, is_active, plots) \
                 VALUES ($1, $2, $3, true, '[]')",
            )
            .bind(id)
            .bind(tenant)
            .bind(label)
            .execute(&pool)
            .await
            .ok()?;
        }

        for (id, tenant, site) in [
            (vineyard_a, tenant_a, site_a),
            (vineyard_b, tenant_b, site_b),
        ] {
            sqlx::query(
                "INSERT INTO vineyards (id, tenant_id, site_id, vintage, created_at, updated_at) \
                 VALUES ($1, $2, $3, 2024, NOW(), NOW())",
            )
            .bind(id)
            .bind(tenant)
            .bind(site)
            .execute(&pool)
            .await
            .ok()?;
        }

        for (id, tenant, vineyard, lot) in [
            (delivery_a, tenant_a, vineyard_a, "LOT-A"),
            (delivery_b, tenant_b, vineyard_b, "LOT-B"),
        ] {
            sqlx::query(
                "INSERT INTO kelter_deliveries \
                 (id, tenant_id, vineyard_id, delivery_date, gross_weight_kg, net_weight_kg, \
                  lot_number, kelter_name, created_at, updated_at) \
                 VALUES ($1, $2, $3, NOW(), 100, 90, $4, 'Kelter', NOW(), NOW())",
            )
            .bind(id)
            .bind(tenant)
            .bind(vineyard)
            .bind(lot)
            .execute(&pool)
            .await
            .ok()?;
        }

        Some(Self {
            pool,
            tenant_a,
            tenant_b,
            delivery_a,
            vineyard_a,
            vineyard_b,
        })
    }

    /// Clean up in reverse dependency order.
    async fn cleanup(&self) {
        for q in [
            "DELETE FROM kelter_deliveries WHERE tenant_id = $1",
            "DELETE FROM vineyards WHERE tenant_id = $1",
        ] {
            let _ = sqlx::query(q).bind(self.tenant_a).execute(&self.pool).await;
            let _ = sqlx::query(q).bind(self.tenant_b).execute(&self.pool).await;
        }
        let _ = sqlx::query("DELETE FROM sites WHERE tenant_id = $1 OR tenant_id = $2")
            .bind(self.tenant_a)
            .bind(self.tenant_b)
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("DELETE FROM tenants WHERE id = $1 OR id = $2")
            .bind(self.tenant_a)
            .bind(self.tenant_b)
            .execute(&self.pool)
            .await;
    }
}

async fn fixture() -> Option<Fixture> {
    match Fixture::new().await {
        Some(f) => Some(f),
        None => {
            eprintln!("skipping: DATABASE_URL unreachable");
            None
        }
    }
}

/// The exact statement `find_all` issues.
#[tokio::test]
#[ignore = "requires DATABASE_URL"]
async fn find_all_returns_only_own_tenant() {
    let Some(f) = fixture().await else {
        panic!("DATABASE_URL must be reachable");
    };

    let a: Vec<(String,)> = sqlx::query_as(
        "SELECT lot_number FROM kelter_deliveries WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
    )
    .bind(f.tenant_a)
    .bind(10_i32)
    .bind(0_i32)
    .fetch_all(&f.pool)
    .await
    .expect("query");

    assert_eq!(a.len(), 1, "tenant A must see exactly its own delivery");
    assert_eq!(a[0].0, "LOT-A");

    let b: Vec<(String,)> = sqlx::query_as(
        "SELECT lot_number FROM kelter_deliveries WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
    )
    .bind(f.tenant_b)
    .bind(10_i32)
    .bind(0_i32)
    .fetch_all(&f.pool)
    .await
    .expect("query");

    assert_eq!(b[0].0, "LOT-B");
    assert!(
        !b.iter().any(|(lot,)| lot == "LOT-A"),
        "tenant B must never see tenant A's rows"
    );

    f.cleanup().await;
}

/// The exact statement `find_by_id` issues.
#[tokio::test]
#[ignore = "requires DATABASE_URL"]
async fn find_by_id_rejects_foreign_tenant() {
    let Some(f) = fixture().await else {
        panic!("DATABASE_URL must be reachable");
    };

    // Owning tenant finds it.
    let own: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM kelter_deliveries WHERE id = $1 AND tenant_id = $2")
            .bind(f.delivery_a)
            .bind(f.tenant_a)
            .fetch_optional(&f.pool)
            .await
            .expect("query");
    assert!(own.is_some(), "owner must find its own delivery");

    // Other tenant must not, even with the correct UUID.
    let foreign: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM kelter_deliveries WHERE id = $1 AND tenant_id = $2")
            .bind(f.delivery_a)
            .bind(f.tenant_b)
            .fetch_optional(&f.pool)
            .await
            .expect("query");
    assert!(
        foreign.is_none(),
        "a guessed UUID must not be readable across tenants"
    );

    f.cleanup().await;
}

/// The exact statement `delete` issues.
#[tokio::test]
#[ignore = "requires DATABASE_URL"]
async fn delete_cannot_remove_foreign_rows() {
    let Some(f) = fixture().await else {
        panic!("DATABASE_URL must be reachable");
    };

    let result = sqlx::query("DELETE FROM kelter_deliveries WHERE id = $1 AND tenant_id = $2")
        .bind(f.delivery_a)
        .bind(f.tenant_b)
        .execute(&f.pool)
        .await
        .expect("query");

    assert_eq!(
        result.rows_affected(),
        0,
        "a cross-tenant delete must affect no rows"
    );

    // The row is still there.
    let still_there: i64 =
        sqlx::query_scalar("SELECT count(*) FROM kelter_deliveries WHERE id = $1")
            .bind(f.delivery_a)
            .fetch_one(&f.pool)
            .await
            .expect("query");
    assert_eq!(still_there, 1, "the row must survive a cross-tenant delete");

    f.cleanup().await;
}

/// The exact statement `find_by_vineyard` issues.
#[tokio::test]
#[ignore = "requires DATABASE_URL"]
async fn find_by_vineyard_is_tenant_scoped() {
    let Some(f) = fixture().await else {
        panic!("DATABASE_URL must be reachable");
    };

    let own: Vec<(String,)> = sqlx::query_as(
        "SELECT lot_number FROM kelter_deliveries WHERE vineyard_id = $1 AND tenant_id = $2 \
         LIMIT $3 OFFSET $4",
    )
    .bind(f.vineyard_a)
    .bind(f.tenant_a)
    .bind(10_i32)
    .bind(0_i32)
    .fetch_all(&f.pool)
    .await
    .expect("query");
    assert_eq!(own.len(), 1);
    assert_eq!(own[0].0, "LOT-A");

    // Tenant B asking for tenant A's vineyard gets nothing.
    let foreign: Vec<(String,)> = sqlx::query_as(
        "SELECT lot_number FROM kelter_deliveries WHERE vineyard_id = $1 AND tenant_id = $2 \
         LIMIT $3 OFFSET $4",
    )
    .bind(f.vineyard_a)
    .bind(f.tenant_b)
    .bind(10_i32)
    .bind(0_i32)
    .fetch_all(&f.pool)
    .await
    .expect("query");
    assert!(
        foreign.is_empty(),
        "another tenant's vineyard must not return rows"
    );

    let _ = f.vineyard_b;
    f.cleanup().await;
}
