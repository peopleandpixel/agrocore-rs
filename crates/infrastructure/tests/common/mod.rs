use agrocore_infrastructure::postgres::Database;
use std::time::Duration;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::{self, Postgres};

pub struct PostgresTestFixture {
    pub pool: sqlx::PgPool,
    pub database: Database,
    _container: testcontainers::ContainerAsync<Postgres>,
}

impl PostgresTestFixture {
    pub async fn new() -> anyhow::Result<Self> {
        let container = postgres::Postgres::default()
            .with_tag("16-alpine")
            .with_env_var("POSTGRES_USER", "test_user")
            .with_env_var("POSTGRES_PASSWORD", "test_password")
            .start()
            .await?;

        let port = container.get_host_port_ipv4(5432).await?;
        let host = container.get_host().await?;
        let database_url = format!(
            "postgres://test_user:test_password@{}:{}/postgres",
            host, port
        );

        tokio::time::sleep(Duration::from_secs(2)).await;
        let pool = sqlx::PgPool::connect(&database_url).await?;

        sqlx::query("CREATE DATABASE agrocore_test")
            .execute(&pool)
            .await
            .ok();

        let test_database_url = format!(
            "postgres://test_user:test_password@{}:{}/agrocore_test",
            host, port
        );
        let pool = sqlx::PgPool::connect(&test_database_url).await?;

        sqlx::migrate!("../../migrations").run(&pool).await?;

        let database = Database::Postgres(agrocore_infrastructure::postgres::PostgresDb {
            pool: pool.clone(),
        });

        Ok(Self {
            pool,
            database,
            _container: container,
        })
    }

    pub async fn create_test_tenant(&self) -> uuid::Uuid {
        use sqlx::Row;
        let row = sqlx::query("INSERT INTO tenants (name, slug, config) VALUES ('Test Tenant', 'test-tenant', '{}') RETURNING id")
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.get("id")
    }
}
