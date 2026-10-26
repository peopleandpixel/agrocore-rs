-- 0000000025_system_bootstrap_flag.sql
-- Add a reliable bootstrap flag to track system initialization.
--
-- The previous approach counted users/tenants to determine if the system was
-- initialized, but Row Level Security (RLS) caused inconsistent counts between
-- the status endpoint and the setup handler. This table provides a single source
-- of truth: a boolean flag that is set to TRUE only after a successful setup.
--
-- No RLS is applied: this table is accessed before any tenant exists, so RLS
-- would block all reads. The table contains only one row with a single boolean
-- column, so there is no sensitive data to protect.

CREATE TABLE IF NOT EXISTS system_bootstrap (
    id         SERIAL PRIMARY KEY,
    initialized BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed the initial row
INSERT INTO system_bootstrap (id, initialized, updated_at)
VALUES (1, FALSE, NOW())
ON CONFLICT (id) DO NOTHING;

-- Grant read/write to the app role
GRANT SELECT, UPDATE ON system_bootstrap TO agrocore_app;
