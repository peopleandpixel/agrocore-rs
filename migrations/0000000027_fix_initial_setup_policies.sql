-- Migration 27: Fix initial_system_setup function to handle existing policies
--
-- The function fails when called after a system reset because the RLS policies
-- created by migration 0000000000 (consolidated_init) still exist. The function
-- must drop policies before creating them, and clean up existing data before
-- inserting to allow re-running setup.

CREATE OR REPLACE FUNCTION public.initial_system_setup(
    p_tenant_name TEXT,
    p_tenant_slug TEXT,
    p_tenant_config JSONB,
    p_admin_firstname TEXT,
    p_admin_lastname TEXT,
    p_admin_email TEXT,
    p_password_hash TEXT
) RETURNS UUID AS $$
DECLARE
    v_tenant_id UUID;
    v_user_id UUID;
    v_role_id UUID;
BEGIN
    -- Suppress audit triggers during initial setup
    PERFORM set_config('app.audit_suppressed', 'on', false);

    -- Clean up any existing setup data before inserting.
    -- This allows re-running setup after a system reset.
    DELETE FROM user_roles;
    DELETE FROM users;
    DELETE FROM roles;
    DELETE FROM tenants;

    -- Drop existing policies that may remain from migration 0000000000 or a
    -- previous setup run. The consolidated migration creates these policies
    -- unconditionally, so a system reset (TRUNCATE) leaves them behind.
    DROP POLICY IF EXISTS tenants_select_policy ON tenants;
    DROP POLICY IF EXISTS tenants_insert_policy ON tenants;
    DROP POLICY IF EXISTS users_select_policy ON users;
    DROP POLICY IF EXISTS users_insert_policy ON users;
    DROP POLICY IF EXISTS users_update_policy ON users;
    DROP POLICY IF EXISTS users_delete_policy ON users;
    DROP POLICY IF EXISTS roles_select_policy ON roles;
    DROP POLICY IF EXISTS roles_insert_policy ON roles;
    DROP POLICY IF EXISTS roles_update_policy ON roles;
    DROP POLICY IF EXISTS roles_delete_policy ON roles;
    DROP POLICY IF EXISTS user_roles_select_policy ON user_roles;
    DROP POLICY IF EXISTS user_roles_insert_policy ON user_roles;
    DROP POLICY IF EXISTS user_roles_delete_policy ON user_roles;

    -- Create the first tenant
    INSERT INTO tenants (name, slug, config, is_active, created_at, updated_at)
    VALUES (p_tenant_name, p_tenant_slug, COALESCE(p_tenant_config, '{}'::JSONB), TRUE, NOW(), NOW())
    RETURNING id INTO v_tenant_id;

    -- Create the initial superadmin role
    INSERT INTO roles (tenant_id, name, description, is_system, created_at, updated_at)
    VALUES (v_tenant_id, 'SuperAdmin', 'System super-administrator with full access', TRUE, NOW(), NOW())
    RETURNING id INTO v_role_id;

    -- Create the initial superadmin user with roles in the JSONB column
    INSERT INTO users (
        tenant_id, firstname, lastname, email, password_hash, roles, is_active,
        internal_cost_per_hour, external_cost_per_hour, created_at, updated_at
    )
    VALUES (
        v_tenant_id, p_admin_firstname, p_admin_lastname, p_admin_email,
        p_password_hash, '["SuperAdmin"]'::jsonb, TRUE, NULL, NULL, NOW(), NOW()
    )
    RETURNING id INTO v_user_id;

    -- Assign the SuperAdmin role to the user via the normalized table as well
    INSERT INTO user_roles (user_id, role_id, tenant_id, created_at)
    VALUES (v_user_id, v_role_id, v_tenant_id, NOW());

    -- Enable RLS on all tables that need it
    ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
    ALTER TABLE users ENABLE ROW LEVEL SECURITY;
    ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
    ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;
    ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
    ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
    ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
    ALTER TABLE task_data ENABLE ROW LEVEL SECURITY;
    ALTER TABLE task_sub_tasks ENABLE ROW LEVEL SECURITY;
    ALTER TABLE equipment ENABLE ROW LEVEL SECURITY;
    ALTER TABLE inventory_items ENABLE ROW LEVEL SECURITY;
    ALTER TABLE inventory_locations ENABLE ROW LEVEL SECURITY;
    ALTER TABLE inventory_transactions ENABLE ROW LEVEL SECURITY;
    ALTER TABLE financial_records ENABLE ROW LEVEL SECURITY;
    ALTER TABLE customers ENABLE ROW LEVEL SECURITY;
    ALTER TABLE workers ENABLE ROW LEVEL SECURITY;
    ALTER TABLE work_logs ENABLE ROW LEVEL SECURITY;
    ALTER TABLE clock_entries ENABLE ROW LEVEL SECURITY;
    ALTER TABLE worker_locations ENABLE ROW LEVEL SECURITY;

    -- Create default policies (safe now that old ones are dropped)
    CREATE POLICY tenants_select_policy ON tenants
        FOR SELECT TO PUBLIC
        USING (is_superadmin() OR id = get_current_tenant_id());

    CREATE POLICY tenants_insert_policy ON tenants
        FOR INSERT TO PUBLIC
        WITH CHECK (is_superadmin());

    CREATE POLICY users_select_policy ON users
        FOR SELECT TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY users_insert_policy ON users
        FOR INSERT TO PUBLIC
        WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY users_update_policy ON users
        FOR UPDATE TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id())
        WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY users_delete_policy ON users
        FOR DELETE TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY roles_select_policy ON roles
        FOR SELECT TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY roles_insert_policy ON roles
        FOR INSERT TO PUBLIC
        WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY roles_update_policy ON roles
        FOR UPDATE TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id())
        WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY roles_delete_policy ON roles
        FOR DELETE TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY user_roles_select_policy ON user_roles
        FOR SELECT TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY user_roles_insert_policy ON user_roles
        FOR INSERT TO PUBLIC
        WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

    CREATE POLICY user_roles_delete_policy ON user_roles
        FOR DELETE TO PUBLIC
        USING (is_superadmin() OR tenant_id = get_current_tenant_id());

    RETURN v_tenant_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
