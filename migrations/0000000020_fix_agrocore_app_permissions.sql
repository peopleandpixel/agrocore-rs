-- Fix: Grant missing permissions to agrocore_app role
--
-- The original migration 0000000004_force_rls.sql only granted permissions
-- to tables that had RLS enabled at the time it ran. Later migrations enabled
-- RLS on more tables, but agrocore_app never received permissions for them.
--
-- This migration re-grants permissions on ALL tables with RLS enabled,
-- ensuring agrocore_app can access every RLS-protected table.
-- It is idempotent and safe to run multiple times.

-- Re-grant on ALL tables with RLS enabled, not just the ones from the original run
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

-- Also ensure sequences are granted
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
