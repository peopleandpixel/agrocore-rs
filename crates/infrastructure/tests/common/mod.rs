use agrocore_infrastructure::postgres::Database;
use std::time::Duration;
use testcontainers::ImageExt;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

pub struct PostgresTestFixture {
    pub pool: sqlx::PgPool,
    pub database: Database,
    _container: ContainerAsync<GenericImage>,
}

impl PostgresTestFixture {
    pub async fn new() -> anyhow::Result<Self> {
        // The postgis image is required: migration 0000000000 creates the
        // postgis extension, which the plain postgres image does not ship.
        // testcontainers_modules::postgres is hardwired to postgres:11-alpine
        // and has no with_tag(), hence GenericImage.
        let container = GenericImage::new("postgis/postgis", "16-3.4")
            .with_exposed_port(5432_u16.into())
            .with_wait_for(WaitFor::message_on_either_std(
                "database system is ready to accept connections",
            ))
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

        // Retry the first connection: the log wait above matches the
        // "ready to accept connections" line, but postgres restarts once during
        // init, which resets in-flight connections.
        let pool = retry_connect(&database_url, 10).await?;

        sqlx::query("CREATE DATABASE agrocore_test")
            .execute(&pool)
            .await
            .ok();

        let test_database_url = format!(
            "postgres://test_user:test_password@{}:{}/agrocore_test",
            host, port
        );
        let pool = retry_connect(&test_database_url, 10).await?;

        sqlx::migrate!("../../migrations").run(&pool).await?;

        let database = Database::Postgres(
            agrocore_infrastructure::postgres::PostgresDb::from_pool(pool.clone()),
        );

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

/// Connect with retries, backing off. The container log signal fires during the
/// init phase, and postgres performs a restart immediately afterwards, so an
/// early connection attempt gets reset.
async fn retry_connect(url: &str, attempts: u32) -> anyhow::Result<sqlx::PgPool> {
    let mut last_err = None;
    for attempt in 0..attempts {
        match sqlx::PgPool::connect(url).await {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                last_err = Some(e);
                let delay = std::cmp::min(1 + attempt, 5);
                tokio::time::sleep(Duration::from_secs(delay.into())).await;
            }
        }
    }
    match last_err {
        Some(e) => Err(e.into()),
        None => Err(anyhow::anyhow!("could not connect to {url}")),
    }
}
