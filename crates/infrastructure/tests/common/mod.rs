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

        // Migration 4 grants `agrocore_app` to a role named `agrocore`, because
        // that is the connection role of the deployment. The container connects
        // as `test_user`, so the migration would abort on a missing role before
        // ever creating a policy. Create the role here as a plain group role:
        // NOLOGIN, so nothing can actually connect as it, and `test_user` is a
        // member, which is what the migration's GRANT needs.
        sqlx::query("CREATE ROLE agrocore NOLOGIN")
            .execute(&pool)
            .await
            .ok();
        sqlx::query("GRANT agrocore TO test_user")
            .execute(&pool)
            .await
            .ok();

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

    /// A fresh tenant with a unique slug.
    ///
    /// The slug has to be unique, and several tests create more than one tenant
    /// against the same database, so a fixed value collided on the second call.
    pub async fn create_test_tenant(&self) -> uuid::Uuid {
        use sqlx::Row;
        let slug = format!("test-tenant-{}", uuid::Uuid::new_v4());
        let row = sqlx::query(
            "INSERT INTO tenants (name, slug, config) VALUES ('Test Tenant', $1, '{}') RETURNING id",
        )
        .bind(&slug)
        .fetch_one(&self.pool)
        .await
        .expect("insert test tenant");
        row.get("id")
    }

    /// A user in the given tenant.
    ///
    /// `workers.user_id` and `worker_task_statuses.worker_id` both reference
    /// `users(id)`, so a test that exercises those repositories needs a real user
    /// row: a random UUID is rejected by the foreign key.
    ///
    /// `common` is compiled into every test binary, and not all of them use this
    /// helper, so the allow keeps the unused warning out of the ones that do not.
    #[allow(dead_code)]
    pub async fn create_test_user(&self, tenant_id: uuid::Uuid) -> uuid::Uuid {
        use sqlx::Row;
        let email = format!("test-{}@example.invalid", uuid::Uuid::new_v4());
        let row = sqlx::query(
            "INSERT INTO users (tenant_id, firstname, lastname, email, password_hash, is_active)
             VALUES ($1, 'Test', 'User', $2, 'not-a-real-hash', true) RETURNING id",
        )
        .bind(tenant_id)
        .bind(&email)
        .fetch_one(&self.pool)
        .await
        .expect("insert test user");
        row.get("id")
    }

    /// A task in the given tenant, for `worker_task_statuses.task_id`.
    #[allow(dead_code)]
    pub async fn create_test_task(&self, tenant_id: uuid::Uuid) -> uuid::Uuid {
        use sqlx::Row;
        let row = sqlx::query(
            "INSERT INTO tasks (tenant_id, title, status) VALUES ($1, 'Test Task', 'pending') RETURNING id",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .expect("insert test task");
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
