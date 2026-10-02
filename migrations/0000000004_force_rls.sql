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

    -- The connection role has to be allowed to switch into it.
    IF NOT EXISTS (
        SELECT 1 FROM pg_auth_members m
        JOIN pg_roles r ON r.oid = m.roleid
        JOIN pg_roles u ON u.oid = m.member
        WHERE r.rolname = 'agrocore_app' AND u.rolname = 'agrocore'
    ) THEN
        EXECUTE 'GRANT agrocore_app TO agrocore';
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
