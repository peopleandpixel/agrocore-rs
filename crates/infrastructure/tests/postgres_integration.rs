//! PostgreSQL Integration Tests using testcontainers
//!
//! These tests spin up a real PostgreSQL instance with PostGIS
//! and test the actual repository implementations.

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::{self, Postgres};
use testcontainers::ImageExt;
use std::time::Duration;
use agrocore_infrastructure::postgres::Database;

/// Test fixture for PostgreSQL integration tests
pub struct PostgresTestFixture {
    pub pool: sqlx::PgPool,
    pub database: Database,
    _container: testcontainers::ContainerAsync<Postgres>,
}

impl PostgresTestFixture {
    /// Creates a new test fixture with a fresh PostgreSQL container
    pub async fn new() -> anyhow::Result<Self> {
        // Start PostgreSQL with PostGIS
        let container = postgres::Postgres::default()
            .with_tag("16-alpine")
            .with_name("agrocore_test")
            .with_user("test_user")
            .with_env_var("POSTGRES_PASSWORD", "test_password")
            .start()
            .await?;

        let port = container.get_host_port_ipv4(5432).await?;
        let host = container.get_host().await?;
        
        let database_url = format!(
            "postgres://test_user:***@{}:{}/postgres",
            host, port
        );

        // Wait for database to be ready
        tokio::time::sleep(Duration::from_secs(2)).await;

        let pool = sqlx::PgPool::connect(&database_url).await?;
        
        // Create test database
        sqlx::query("CREATE DATABASE agrocore_test").execute(&pool).await.ok();

        // Connect to test database
        let test_database_url = format!(
            "postgres://test_user:***@{}:{}/agrocore_test",
            host, port
        );
        let pool = sqlx::PgPool::connect(&test_database_url).await?;
        
        // Run migrations
        sqlx::migrate!("../../migrations").run(&pool).await?;

        let database = Database::Postgres(
            agrocore_infrastructure::postgres::PostgresDb { pool: pool.clone() }
        );

        Ok(Self {
            pool,
            database,
            _container: container,
        })
    }

    /// Get a fresh tenant ID for testing
    pub async fn create_test_tenant(&self) -> uuid::Uuid {
        use sqlx::Row;
        let row = sqlx::query("INSERT INTO tenants (name, slug, config) VALUES ('Test Tenant', 'test-tenant', '{}') RETURNING id")
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.get("id")
    }

    /// Clean up test data (truncate all tables)
    pub async fn cleanup(&self) {
        let tables = [
            "order_sites", "user_sites", "order_sites", "sites", "equipment", 
            "orders", "animals", "tasks", "task_data", "users", "weather_data", 
            "weather_stations", "tenants"
        ];
        for table in tables {
            let _ = sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool)
                .await;
        }
    }
}

/// Helper to create test tenant
pub async fn create_test_tenant(pool: &sqlx::PgPool) -> uuid::Uuid {
    use sqlx::Row;
    let row = sqlx::query("INSERT INTO tenants (name, slug, config) VALUES ('Test Tenant', 'test-tenant', '{}') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    row.get("id")
}