-- 0000000017_initial_system_setup.sql
-- Create the SECURITY DEFINER function for initial system setup
-- This function runs as the table owner (agrocore) and handles FORCE RLS internally.
-- It must be called outside any transaction block for ALTER TABLE to work.

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
    -- Create the first tenant
    INSERT INTO tenants (name, slug, config, is_active, created_at, updated_at)
    VALUES (p_tenant_name, p_tenant_slug, COALESCE(p_tenant_config, '{}'::JSONB), TRUE, NOW(), NOW())
    RETURNING id INTO v_tenant_id;

    -- Create the initial admin role
    INSERT INTO roles (tenant_id, name, description, is_system, created_at, updated_at)
    VALUES (v_tenant_id, 'Admin', 'System administrator with full access', TRUE, NOW(), NOW())
    RETURNING id INTO v_role_id;

    -- Create the initial admin user
    INSERT INTO users (
        tenant_id, firstname, lastname, email, password_hash, is_active,
        internal_cost_per_hour, external_cost_per_hour, created_at, updated_at
    )
    VALUES (
        v_tenant_id, p_admin_firstname, p_admin_lastname, p_admin_email,
        p_password_hash, TRUE, NULL, NULL, NOW(), NOW()
    )
    RETURNING id INTO v_user_id;

    -- Assign the Admin role to the user
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

    -- Create default policies
    -- Tenants: superadmin can see all, tenant members see only their tenant
    CREATE POLICY tenants_select_policy ON tenants
        FOR SELECT TO PUBLIC
        USING (is_superadmin() OR id = get_current_tenant_id());

    CREATE POLICY tenants_insert_policy ON tenants
        FOR INSERT TO PUBLIC
        WITH CHECK (is_superadmin());

    -- Users policies
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

    -- Roles policies
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

    -- User roles policies
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

-- Grant execute permission
GRANT EXECUTE ON FUNCTION public.initial_system_setup(
    TEXT, TEXT, JSONB, TEXT, TEXT, TEXT, TEXT
) TO PUBLIC;