-- Make the existing row-level security actually effective (tasks.md A3).
--
-- The schema created 190 policies over 50 tables, but none of them could ever
-- evaluate:
--
--   1. `app.current_tenant_id` was never set anywhere in the Rust code, so
--      get_current_tenant_id() returned NULL and every policy compared
--      `tenant_id = NULL` -> NULL -> false, which would have blocked *all*
--      rows rather than leaking them.
--   2. The table owner bypasses RLS by default, and the application connects
--      as the owner (`agrocore`), so no policy was ever consulted.
--   3. `FORCE ROW LEVEL SECURITY` was never applied.
--
-- This migration closes the second and third gap.
--
-- The remaining gap is the first one: `app.current_tenant_id` has to be set
-- before each query. That is deliberately NOT done here, because turning the
-- policies live without the pin in place would make every repository return
-- zero rows. The pin arrives with the repository-side change recorded in
-- tasks.md A3.
--
-- Until every repository is wrapped, this migration is applied but the
-- `SET ROLE` in pg_pool_options is gated behind AGROCORE_RLS_ENABLED. That
-- way the database is hardened and the switchover is a config change once the
-- pin is in place, not a coordinated release.

-- ---------------------------------------------------------------------------
-- 1. The application role.
--
-- Measured on this deployment: the connection role `agrocore` is SUPERUSER and
-- has BYPASSRLS. Neither FORCE ROW LEVEL SECURITY nor any policy can constrain
-- such a role — PostgreSQL exempts it unconditionally. Creating policies for a
-- superuser connection is worse than useless because it looks like protection.
--
-- `agrocore_app` is therefore a role with neither attribute. It has NOLOGIN:
-- the pool authenticates as `agrocore` and SET ROLE down to this one, which
-- keeps the change inside a single migration and requires no new credentials.
--
-- IMPORTANT: `agrocore` must SET ROLE agrocore_app before issuing queries, or
-- the policies stay inert. That happens in PostgresDb::connect, see
-- crates/infrastructure/src/postgres/database.rs.
-- ---------------------------------------------------------------------------
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'agrocore_app') THEN
        CREATE ROLE agrocore_app NOLOGIN NOSUPERUSER NOBYPASSRLS;
        RAISE NOTICE 'Created role agrocore_app (NOSUPERUSER, NOBYPASSRLS)';
    END IF;

    -- Strip the exemptions if the role was created by an earlier run without
    -- them; ALTER is idempotent and makes the migration self-healing.
    EXECUTE 'ALTER ROLE agrocore_app NOSUPERUSER NOBYPASSRLS';

    -- The role that is running this migration has to be allowed to switch into
    -- `agrocore_app`. Which role that is depends on the deployment: the docker
    -- setup connects as `agrocore`, CI as `test`. Hardcoding a name made the
    -- migration abort with `role "agrocore" does not exist` wherever the
    -- connection role differed -- including every CI run.
    --
    -- `current_user` is the role the migration executes as, which is the role
    -- that later issues `SET ROLE agrocore_app`.
    IF NOT EXISTS (
        SELECT 1 FROM pg_auth_members m
        JOIN pg_roles r ON r.oid = m.roleid
        WHERE r.rolname = 'agrocore_app' AND m.member = current_user::regrole
    ) THEN
        EXECUTE format('GRANT agrocore_app TO %I', current_user);
        RAISE NOTICE 'Granted agrocore_app to the connection role %', current_user;
    END IF;
END
$$;

GRANT USAGE ON SCHEMA public TO agrocore_app;

-- ---------------------------------------------------------------------------
-- 2. Grant the role access to the RLS-protected tables.
--
-- GRANT on a table does not bypass RLS, so this is safe: the policies still
-- apply. What it replaces is the owner's implicit access.
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    t RECORD;
BEGIN
    FOR t IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relrowsecurity
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO agrocore_app', t.relname);
    END LOOP;
END
$$;

-- Sequences are needed for the serial/identity defaults the repositories rely
-- on when inserting.
DO $$
DECLARE
    s RECORD;
BEGIN
    FOR s IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'S'
    LOOP
        EXECUTE format('GRANT USAGE, SELECT ON public.%I TO agrocore_app', s.relname);
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 3. FORCE RLS so the owner is subject to the policies too.
--
-- This is what actually makes the existing policies bite. Without FORCE, a
-- table owner is exempt, and the application connects as the owner.
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    t RECORD;
BEGIN
    FOR t IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relrowsecurity
          AND NOT c.relforcerowsecurity
    LOOP
        EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', t.relname);
        RAISE NOTICE 'FORCE ROW LEVEL SECURITY on %', t.relname;
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 3b. Tables that had RLS enabled but no policy at all.
--
-- FORCE on a table without a policy denies every row to every non-owner,
-- including the tenant that owns them. `sigpac_parcels` was in that state: it
-- had ENABLE ROW LEVEL SECURITY but no CREATE POLICY, so it was relying on the
-- owner exemption to stay readable. Give it the same isolation as the others.
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    t RECORD;
BEGIN
    FOR t IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relforcerowsecurity
          AND NOT EXISTS (SELECT 1 FROM pg_policies p WHERE p.tablename = c.relname)
          AND EXISTS (
              SELECT 1 FROM information_schema.columns col
              WHERE col.table_schema = 'public'
                AND col.table_name = c.relname
                AND col.column_name = 'tenant_id'
          )
    LOOP
        EXECUTE format(
            'CREATE POLICY %I ON public.%I USING (tenant_id = get_current_tenant_id()) WITH CHECK (tenant_id = get_current_tenant_id())',
            t.relname || '_tenant_isolation', t.relname);
        RAISE NOTICE 'Added missing tenant policy to %', t.relname;
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 4. Tables that carry tenant data but were never given a policy.
--
-- Without a policy, FORCE RLS denies everything on these tables, including
-- for the tenant that owns the rows. Each gets the same isolation the others
-- have. `tenants` is excluded on purpose: it is the tenant list itself and
-- has to be readable before a tenant is known.
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    t RECORD;
BEGIN
    FOR t IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND EXISTS (
              SELECT 1 FROM information_schema.columns col
              WHERE col.table_schema = 'public'
                AND col.table_name = c.relname
                AND col.column_name = 'tenant_id'
          )
          AND NOT c.relrowsecurity
          AND c.relname <> 'tenants'
    LOOP
        EXECUTE format('ALTER TABLE public.%I ENABLE ROW LEVEL SECURITY', t.relname);
        EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', t.relname);
        EXECUTE format(
            'CREATE POLICY %I ON public.%I USING (tenant_id = get_current_tenant_id()) WITH CHECK (tenant_id = get_current_tenant_id())',
            t.relname || '_tenant_isolation', t.relname);
        RAISE NOTICE 'Added tenant isolation to %', t.relname;
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 5. Tenants cannot be created.
--
-- `tenants` carries policies for SELECT, UPDATE and DELETE but none for
-- INSERT. With FORCE applied, PostgreSQL treats a missing INSERT policy as
-- "deny everything", so `POST /api/v1/system/setup` — the endpoint that
-- creates the very first tenant — would fail for every deployment that has
-- FORCE on. Tenant creation is a setup action that happens before a tenant
-- exists, so it is not tenant-scoped data and must not be filtered.
-- ---------------------------------------------------------------------------
DROP POLICY IF EXISTS tenants_insert ON tenants;
CREATE POLICY tenants_insert ON tenants FOR INSERT WITH CHECK (true);

-- ---------------------------------------------------------
-- Grants must cover every table, including ones added after
-- this migration ran. Hand-listing tables left ten of them
-- (user_sites, breeds, varieties, the equipment_* logs,
-- grazing_records, treatment_records, sites_history)
-- unreachable: the login path joins user_sites, so every
-- login failed with "permission denied for table user_sites".
--
-- A loop over pg_class is used instead of a list so a later
-- migration cannot reintroduce the same gap.
-- ---------------------------------------------------------
DO $$
DECLARE
    r record;
BEGIN
    FOR r IN
        SELECT c.oid::regclass AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind IN ('r', 'p')
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON %s TO agrocore_app', r.tbl);
    END LOOP;

    -- Sequences are needed for INSERTs with serial/identity defaults.
    FOR r IN
        SELECT c.oid::regclass AS seq
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'S'
    LOOP
        EXECUTE format('GRANT USAGE, SELECT ON %s TO agrocore_app', r.seq);
    END LOOP;
END $$;

-- Tables created after this migration still need RLS. `tenant_id` is the
-- marker: a table carrying it is tenant-scoped by definition, so enabling RLS
-- on it without a policy would only deny access, never widen it.
DO $$
DECLARE
    r record;
BEGIN
    FOR r IN
        SELECT c.oid::regclass AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind IN ('r', 'p')
          AND EXISTS (
              SELECT 1 FROM pg_attribute a
              WHERE a.attrelid = c.oid
                AND a.attname = 'tenant_id'
                AND NOT a.attisdropped
          )
          AND NOT EXISTS (
              SELECT 1 FROM pg_policies p
              WHERE p.schemaname = 'public'
                AND p.tablename = c.relname
          )
    LOOP
        EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', r.tbl);
        EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', r.tbl);
        -- Tenant-scoped by column: restrict reads and writes to the pinned
        -- tenant. The column check is redundant with RLS but costs nothing and
        -- documents the intent at the table.
        EXECUTE format($p$
            CREATE POLICY tenant_isolation ON %s
            USING (tenant_id = get_current_tenant_id())
            WITH CHECK (tenant_id = get_current_tenant_id())
        $p$, r.tbl);
    END LOOP;
END $$;

-- ---------------------------------------------------------
-- Authentication role.
--
-- Login is the one flow that must read `users` before the
-- tenant is known: `app.current_tenant_id` is set *from* the
-- user row, so a pinned read would return nothing and every
-- login would fail with "Row not found".
--
-- `agrocore_auth` grants exactly that: SELECT on `users`, and
-- nothing else. It is not a member of `agrocore_app`, it
-- carries no BYPASSRLS, and it is reached only inside a
-- transaction via SET LOCAL ROLE, so the privilege cannot
-- outlive the login query or be reused to read tenant data.
-- ---------------------------------------------------------
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'agrocore_auth') THEN
        CREATE ROLE agrocore_auth NOLOGIN;
    END IF;
END $$;

GRANT SELECT ON users TO agrocore_auth;

-- `USER_SELECT_FIELDS` aggregates the user's assigned sites via
-- `LEFT JOIN user_sites`, so the login query needs this table too. It is a
-- per-user mapping with no `tenant_id` column of its own, so it cannot carry a
-- tenant policy; access is instead limited to SELECT and only through the
-- authentication role, whose reach ends with the login query.
GRANT SELECT ON user_sites TO agrocore_auth;

-- `SET LOCAL ROLE` requires membership. The role that has it is the one this
-- migration runs as, which differs per deployment (`agrocore` in docker, `test`
-- in CI) -- hardcoding the name aborted the migration wherever it differed.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_auth_members m
        JOIN pg_roles r ON r.oid = m.roleid
        WHERE r.rolname = 'agrocore_auth' AND m.member = current_user::regrole
    ) THEN
        EXECUTE format('GRANT agrocore_auth TO %I', current_user);
    END IF;
END
$$;

-- The `users_select` policy from the init migration is
--   is_superadmin() OR tenant_id = get_current_tenant_id()
-- which a login cannot satisfy: the tenant is read *from* the user row, so the
-- pin does not exist yet. This narrow policy admits only the authentication
-- role, only for SELECT, and only on `users` - enough to resolve the tenant,
-- nothing more.
DROP POLICY IF EXISTS users_auth_lookup ON users;
CREATE POLICY users_auth_lookup ON users
FOR SELECT TO agrocore_auth
USING (is_active = true);

-- ---------------------------------------------------------
-- orders.order_type is VARCHAR while every other enum-ish
-- column on the table (status, recurrence, execution_policy,
-- automation_state) is JSONB, and the Rust entity declares it
-- with #[sqlx(json)].
--
-- Every read of GET /api/v1/orders therefore failed with
--   error occurred while decoding column "order_type":
--   Rust type Json<OrderType> (as JSONB) is not compatible
--   with SQL type VARCHAR
--
-- Existing plain values such as 'seeding' are converted to
-- JSONB strings, matching how `status` already stores its
-- values.
-- ---------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'orders'
          AND column_name = 'order_type'
          AND data_type = 'character varying'
    ) THEN
        EXECUTE $m$
            ALTER TABLE orders
            ALTER COLUMN order_type TYPE JSONB
            USING CASE
                WHEN order_type IS NULL THEN NULL
                WHEN order_type ~ '^\s*[\{\"].*[\}\"]\s*$' THEN order_type::jsonb
                ELSE to_jsonb(order_type::text)
            END
        $m$;
    END IF;
END $$;

-- ---------------------------------------------------------
-- The `orders` entity declares `started_at` and `completed_at`
-- (both `Option<DateTime<Utc>>`), but the init migration never
-- created the columns. Every read of GET /api/v1/orders failed
-- with `no column found for name: started_at`, so the whole
-- order list was unreachable.
-- ---------------------------------------------------------
ALTER TABLE orders
    ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS completed_at TIMESTAMPTZ;

-- ---------------------------------------------------------
-- Numeric columns vs. f64 in Rust.
--
-- The entities map money and measurements to `f64`, which sqlx reads as
-- FLOAT8. 37 columns were declared NUMERIC, so every entity touching one of
-- them failed to decode:
--
--   Rust type Option<f64> (as SQL type FLOAT8) is not compatible with
--   SQL type NUMERIC
--
-- `GET /api/v1/customers` failed on customers.vat_rate this way. The project
-- has no rust_decimal dependency and 69 other columns are already DOUBLE
-- PRECISION, so NUMERIC was the outlier rather than the intended type.
--
-- The loop is deliberate: a hand-written list would drift again as soon as a
-- table gains a numeric column. `ALTER TABLE %I.%I` keeps table and column
-- separate, which is what the first attempt got wrong.
DO $$
DECLARE
    r record;
BEGIN
    FOR r IN
        SELECT c.table_schema, c.table_name, c.column_name
        FROM information_schema.columns c
        JOIN pg_namespace n ON n.nspname = c.table_schema
        JOIN pg_class cl ON cl.relname = c.table_name AND cl.relnamespace = n.oid
        JOIN pg_attribute a
          ON a.attrelid = cl.oid AND a.attname = c.column_name
        WHERE c.table_schema = 'public'
          AND c.data_type = 'numeric'
          AND a.attgenerated = ''
          -- A column that a generated column depends on cannot be altered:
          -- equipment_fuel_consumption.total_cost is generated from
          -- liters * cost_per_liter, so changing those types would invalidate
          -- the stored expression. PostgreSQL refuses the whole table, so all
          -- four of its numeric columns keep NUMERIC. The entity selects them
          -- through `::float8` in SQL instead of relying on the column type,
          -- which is stated there.
          AND cl.oid NOT IN (
              SELECT a2.attrelid
              FROM pg_attribute a2
              WHERE a2.attgenerated <> ''
          )
    LOOP
        EXECUTE format(
            'ALTER TABLE %I.%I ALTER COLUMN %I TYPE DOUBLE PRECISION USING %I::double precision',
            r.table_schema, r.table_name, r.column_name, r.column_name, r.column_name
        );
    END LOOP;
END $$;
