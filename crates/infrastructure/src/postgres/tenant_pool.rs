//! A `PgPool` wrapper that pins the tenant before every query.
//!
//! # Why this exists
//!
//! The row-level-security policies call `get_current_tenant_id()`, which reads
//! the session setting `app.current_tenant_id`. That setting is *session
//! scoped*, and a pooled connection may have served a different tenant before.
//! So the pin has to be re-applied after every checkout.
//!
//! Applying it in each of the 47 repositories would mean 47 chances to forget
//! it. `TenantPool` implements sqlx's [`Executor`], so repositories keep their
//! existing `fetch_one(&pool)` / `fetch_all(&pool)` call sites unchanged while
//! the pin becomes impossible to omit: you cannot have a `TenantPool` without
//! naming a tenant.
//!
//! ```ignore
//! // inside a repository method that receives `tid: TenantId`
//! let pool = TenantPool::new(&self.pool, tid.0);
//! sqlx::query("SELECT ... FROM sites WHERE tenant_id = $1").bind(tid)
//!     .fetch_all(&pool).await?;   // pin applied by fetch_many, same connection
//! ```
//!
//! Only `fetch_many` is implemented; the other [`Executor`] methods have
//! default bodies that derive from it. That matters because the defaults call
//! `fetch_many` once and then consume the stream, which keeps the pinned
//! connection alive for the whole query — including multi-statement ones.

use futures::{Stream, StreamExt, TryStreamExt};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use sqlx::Acquire;
use sqlx::Execute;
use sqlx::postgres::{PgConnection, PgPool, PgPoolOptions};
use sqlx::{Database, Either, Error, Executor};
use std::sync::Arc;
use uuid::Uuid;

/// The pool plus the tenant whose rows this connection may see.
#[derive(Clone, Debug)]
pub struct TenantPool {
    pool: Arc<PgPool>,
    tenant_id: Uuid,
}

impl TenantPool {
    /// Bind a tenant to the pool.
    pub fn new(pool: &PgPool, tenant_id: Uuid) -> Self {
        Self {
            pool: Arc::new(pool.clone()),
            tenant_id,
        }
    }

    /// The pool without a tenant, for work that is genuinely not tenant-scoped
    /// (migrations, health checks).
    ///
    /// Uses the nil UUID, which matches no tenant, so the policies deny
    /// everything. That is the safe direction: reading nothing rather than
    /// reading everything.
    pub fn unscoped(pool: &PgPool) -> Self {
        Self {
            pool: Arc::new(pool.clone()),
            tenant_id: Uuid::nil(),
        }
    }

    /// The tenant this pool is bound to.
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Take a pinned connection.
    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, Error> {
        let mut conn = self.pool.acquire().await?;
        set_tenant(&mut conn, self.tenant_id).await?;
        Ok(conn)
    }

    /// Begin a transaction on a pinned connection.
    ///
    /// Two statements, in this order, and the order matters:
    ///
    /// 1. `BEGIN` explicitly, so a real transaction block exists before the pin
    ///    is set. `SET LOCAL` outside a transaction block is a no-op in
    ///    PostgreSQL and would silently leave the pin unset.
    ///
    /// 2. `set_config(..., true)`, which scopes the pin to that block.
    ///
    /// Doing it the other way round, by issuing `set_config` on a bare
    /// connection, would appear to work and then reset on the first statement.
    /// That is exactly the kind of silent failure which leaves RLS inert.
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, Error> {
        self.begin_with_superadmin(false).await
    }

    /// Begin a transaction with superadmin flag (bypasses RLS for setup/bootstrap).
    pub async fn begin_superadmin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, Error> {
        self.begin_with_superadmin(true).await
    }

    /// Internal: begin with optional superadmin flag.
    async fn begin_with_superadmin(
        &self,
        is_superadmin: bool,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.current_tenant_id', $1, true), \
                    set_config('app.is_superadmin', $2, true)",
        )
        .bind(self.tenant_id.to_string())
        .bind(is_superadmin.to_string())
        .execute(&mut *tx)
        .await?;
        Ok(tx)
    }

    /// The underlying pool, for the rare call that genuinely needs it.
    ///
    /// Every use here is a place where the tenant pin is not applied, so it
    /// should be rare and deliberate.
    pub fn raw(&self) -> &PgPool {
        &self.pool
    }

    pub fn size(&self) -> u32 {
        self.pool.size()
    }

    pub fn num_idle(&self) -> usize {
        self.pool.num_idle()
    }
}

/// Pin the tenant on the connection that runs the query.
///
/// `app.is_superadmin` is cleared as well: the existing policies consult it, so
/// a connection that previously served an administrative query must not carry
/// elevated rights into the next checkout.
pub async fn set_tenant(conn: &mut PgConnection, tenant_id: Uuid) -> Result<(), Error> {
    sqlx::query(
        "SELECT set_config('app.current_tenant_id', $1, false), \
                set_config('app.is_superadmin', 'false', false)",
    )
    .bind(tenant_id.to_string())
    .execute(conn)
    .await?;
    Ok(())
}

// Mirrors sqlx's own `Executor for &Pool` implementation
// (sqlx-core/src/pool/executor.rs), with one addition: the connection is pinned
// to the tenant before the query runs. The `conn` binding stays in scope for
// the whole stream, so the pin holds for every statement of a multi-statement
// query.

// Mirrors sqlx's own `Executor for &Pool` implementation
// (sqlx-core/src/pool/executor.rs), with one addition: the connection is pinned
// to the tenant before the query runs.
//
// The connection is owned by the stream, not borrowed. That is what keeps the
// pin in effect for every statement of a multi-statement query: the connection
// is only returned to the pool once the stream is exhausted or dropped.
struct PinnedQuery<'e> {
    /// Runs the query. `None` once it has completed.
    inner: Option<BoxFuture<'e, Result<Vec<Step>, Error>>>,
    steps: std::vec::IntoIter<Step>,
}

type Step = Either<<sqlx::Postgres as Database>::QueryResult, <sqlx::Postgres as Database>::Row>;

impl Unpin for PinnedQuery<'_> {}

impl<'e> Stream for PinnedQuery<'e> {
    type Item = Result<Step, Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(step) = this.steps.next() {
                return std::task::Poll::Ready(Some(Ok(step)));
            }
            let Some(fut) = this.inner.take() else {
                return std::task::Poll::Ready(None);
            };
            let mut fut = fut;
            match fut.as_mut().poll(cx) {
                std::task::Poll::Pending => {
                    this.inner = Some(fut);
                    return std::task::Poll::Pending;
                }
                std::task::Poll::Ready(Err(e)) => return std::task::Poll::Ready(Some(Err(e))),
                std::task::Poll::Ready(Ok(steps)) => this.steps = steps.into_iter(),
            }
        }
    }
}

fn fetch_many_pinned<'e, 'q: 'e, E>(
    pool: &PgPool,
    tenant_id: Uuid,
    query: E,
) -> BoxStream<'e, Result<Step, Error>>
where
    E: 'q + Execute<'q, sqlx::Postgres> + Send + 'e,
{
    let pool = pool.clone();
    let fut: BoxFuture<'e, Result<Vec<Step>, Error>> = Box::pin(async move {
        let mut conn = pool.acquire().await?;
        set_tenant(&mut conn, tenant_id).await?;
        let steps: Vec<Step> = conn.fetch_many(query).try_collect().await?;
        // `conn` drops here, returning the pinned connection to the pool.
        Ok(steps)
    });
    Box::pin(PinnedQuery {
        inner: Some(fut),
        steps: Vec::new().into_iter(),
    })
}

impl<'c> Executor<'c> for &TenantPool {
    type Database = sqlx::Postgres;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<sqlx::postgres::PgQueryResult, sqlx::postgres::PgRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        let tenant_id = self.tenant_id;
        fetch_many_pinned(self.raw(), tenant_id, query)
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        let pool = self.raw().clone();
        let tenant_id = self.tenant_id;
        Box::pin(async move {
            let mut conn = pool.acquire().await?;
            set_tenant(&mut conn, tenant_id).await?;
            conn.fetch_optional(query).await
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as Database>::Statement<'q>, Error>>
    where
        'c: 'e,
    {
        let pool = self.raw().clone();
        let tenant_id = self.tenant_id;
        Box::pin(async move {
            let mut conn = pool.acquire().await?;
            set_tenant(&mut conn, tenant_id).await?;
            conn.prepare_with(sql, parameters).await
        })
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        let pool = self.raw().clone();
        let tenant_id = self.tenant_id;
        Box::pin(async move {
            let mut conn = pool.acquire().await?;
            set_tenant(&mut conn, tenant_id).await?;
            conn.describe(sql).await
        })
    }
}

impl<'c> Executor<'c> for TenantPool {
    type Database = sqlx::Postgres;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<sqlx::postgres::PgQueryResult, sqlx::postgres::PgRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        (&self).fetch_many(query)
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        (&self).fetch_optional(query)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> BoxFuture<'e, Result<<Self::Database as Database>::Statement<'q>, Error>>
    where
        'c: 'e,
    {
        (&self).prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        (&self).describe(sql)
    }
}

/// Build a pool whose connections already switch to the RLS-enforcing role.
///
/// Failure to switch is fatal rather than silent: without the role the
/// policies are inert, which would look like working isolation while providing
/// none — the exact failure mode this exists to eliminate.
pub fn rls_pool_options(base: PgPoolOptions) -> PgPoolOptions {
    base.after_connect(|conn, _meta| {
        Box::pin(async move {
            sqlx::query("SET ROLE agrocore_app")
                .execute(conn)
                .await
                .map(|_| ())
                .map_err(|e| {
                    Error::Configuration(Box::new(std::io::Error::other(format!(
                        "cannot SET ROLE agrocore_app: {e}"
                    ))))
                })
        })
    })
}
