//! Schema-to-entity conformance for the `orders` table.
//!
//! Found while bringing row-level security live: `GET /api/v1/orders` failed on
//! a freshly migrated database with
//!
//! ```text
//! error occurred while decoding column "recurrence": unexpected null;
//! try decoding as an `Option`
//! ```
//!
//! Every repository query for this entity is `SELECT *`, so any drift between
//! the table definition and the Rust struct makes the whole order list
//! unreachable. These tests read the real table and check the mapping, so the
//! drift cannot come back unnoticed.

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// Column names as PostgreSQL reports them for `SELECT *`.
///
/// Read from `information_schema` rather than from a zero-row result: with
/// `LIMIT 0` there are no rows to inspect the row metadata of.
async fn order_columns(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns \
         WHERE table_name = 'orders' ORDER BY ordinal_position",
    )
    .fetch_all(pool)
    .await
    .expect("describe orders")
}

/// A pool whose every connection is both RLS-enforcing and tenant-pinned.
///
/// The pin must be set in `after_connect`, not once via a separate statement:
/// the pin is session state, so a later query may land on a different pooled
/// connection and see nothing.
async fn pinned_pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(3)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE agrocore_app")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query(
                    "SELECT set_config('app.current_tenant_id', $1, false), \
                            set_config('app.is_superadmin', 'false', false)",
                )
                .bind(DEMO_TENANT)
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

const DEMO_TENANT: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

/// Every JSONB column must be declared with `#[sqlx(json)]`, otherwise sqlx
/// tries to decode the raw JSONB value into the inner type and fails on NULL.
///
/// The seed leaves most of these NULL, which is the normal state for a fresh
/// order, so this is the default path and not an edge case.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set and demo seed"]
async fn jsonb_columns_round_trip_null() {
    let pool = pinned_pool().await;
    let row = sqlx::query("SELECT * FROM orders LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("a seeded order exists for the demo tenant");

    // Decoding every column by name is exactly what `query_as::<_, Order>` does.
    // Doing it per column pinpoints which one is broken instead of reporting
    // the first failure for the whole row.
    let jsonb_columns = ["recurrence", "execution_policy", "automation_state"];
    for name in jsonb_columns {
        let raw: Option<serde_json::Value> = row
            .try_get::<Option<serde_json::Value>, _>(name)
            .unwrap_or_else(|e| panic!("column {name} is not readable as jsonb: {e}"));
        // Both NULL and a value must decode; the point is the column is jsonb.
        if let Some(v) = raw {
            assert!(
                v.is_object() || v.is_string(),
                "column {name} holds an unexpected shape: {v}"
            );
        }
    }
}

/// The entity declares `started_at` and `completed_at`; the table did not have
/// them at all, which made every read fail with "no column found for name:
/// started_at". Guard the columns themselves.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set and demo seed"]
async fn entity_timestamps_exist() {
    let pool = pinned_pool().await;
    let cols = order_columns(&pool).await;
    for name in ["started_at", "completed_at", "last_completed_at"] {
        assert!(
            cols.iter().any(|c| c == name),
            "orders is missing column {name}; the Order entity declares it"
        );
    }
}

/// `order_type` and `status` are JSONB in the database and `#[sqlx(json)]` in
/// the entity. A VARCHAR column here fails decoding with a type mismatch.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set and demo seed"]
async fn enum_columns_are_jsonb() {
    let pool = pinned_pool().await;
    let types: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, data_type FROM information_schema.columns \
                        WHERE table_name = 'orders' AND column_name IN ('order_type', 'status')",
    )
    .fetch_all(&pool)
    .await
    .expect("read column types");

    assert_eq!(types.len(), 2, "both columns must exist");
    for (name, data_type) in types {
        assert_eq!(
            data_type, "jsonb",
            "orders.{name} must be jsonb to match #[sqlx(json)]"
        );
    }
}

/// `planned_date` and `deadline_date` are DATE columns. The entity uses
/// `NaiveDate`; a TIMESTAMPTZ mapping fails with "not compatible with DATE".
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set and demo seed"]
async fn planned_and_deadline_are_dates() {
    let pool = pinned_pool().await;
    let types: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name, data_type FROM information_schema.columns \
                        WHERE table_name = 'orders' \
                          AND column_name IN ('planned_date', 'deadline_date')",
    )
    .fetch_all(&pool)
    .await
    .expect("read column types");

    assert_eq!(types.len(), 2);
    for (name, data_type) in types {
        assert_eq!(
            data_type, "date",
            "orders.{name} must be date for NaiveDate"
        );
    }
}

/// Reproduces the exact repository call: `query_as::<_, Order>("SELECT * ...")`.
/// The name-based column reads above all passed, so if this fails the problem is
/// in how sqlx maps the row onto the struct, not in the schema.
#[tokio::test]
#[ignore = "needs DATABASE_URL with the full migration set and demo seed"]
async fn entity_decodes_full_row() {
    use agrocore_domain::entities::order::Order;
    let pool = pinned_pool().await;
    let res: Result<Vec<Order>, sqlx::Error> =
        sqlx::query_as("SELECT * FROM orders ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await;
    match res {
        Ok(rows) => assert!(!rows.is_empty(), "expected seeded orders"),
        Err(e) => panic!("query_as::<_, Order> failed: {e}"),
    }
}
