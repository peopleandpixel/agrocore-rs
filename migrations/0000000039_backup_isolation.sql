-- Migration 0039: Backup Isolation for Demo Tenants
-- Creates functions to temporarily exclude demo tenant data from backups.
-- Demo tenants are identified by tenants.is_demo = true.

BEGIN;

-- Function to get all demo tenant IDs
CREATE OR REPLACE FUNCTION get_demo_tenant_ids()
RETURNS TABLE(tenant_id UUID)
LANGUAGE sql
STABLE
AS $$
    SELECT id FROM tenants WHERE is_demo = true AND is_active = true;
$$;

-- Function to check if any demo tenants exist
CREATE OR REPLACE FUNCTION has_demo_tenants()
RETURNS BOOLEAN
LANGUAGE sql
STABLE
AS $$
    SELECT EXISTS(SELECT 1 FROM tenants WHERE is_demo = true AND is_active = true LIMIT 1);
$$;

COMMENT ON FUNCTION get_demo_tenant_ids() IS 'Returns all active demo tenant IDs for backup isolation.';
COMMENT ON FUNCTION has_demo_tenants() IS 'Checks if any active demo tenants exist.';

COMMIT;
