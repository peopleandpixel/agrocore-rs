-- 0000000022_rbac_permission_matrix.sql
-- Adds tables for the role-based permission matrix system and user permission overrides.

-- Role permissions: defines what each role can do per module
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module VARCHAR(50) NOT NULL,
    actions JSONB NOT NULL DEFAULT '[]'::JSONB,
    scope VARCHAR(20) NOT NULL DEFAULT 'tenant',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (role_id, module)
);

CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_tenant_id ON role_permissions(tenant_id);

-- User permission overrides: per-user grants and revocations that override role permissions
CREATE TABLE IF NOT EXISTS user_permission_overrides (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module VARCHAR(50) NOT NULL,
    actions JSONB NOT NULL DEFAULT '[]'::JSONB,
    scope VARCHAR(20) NOT NULL DEFAULT 'tenant',
    is_grant BOOLEAN NOT NULL DEFAULT true, -- true = grant, false = revoke (deny)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (user_id, module, is_grant)
);

CREATE INDEX IF NOT EXISTS idx_user_permission_overrides_user_id ON user_permission_overrides(user_id);
CREATE INDEX IF NOT EXISTS idx_user_permission_overrides_tenant_id ON user_permission_overrides(tenant_id);

-- RLS policies for role_permissions
ALTER TABLE role_permissions ENABLE ROW LEVEL SECURITY;

CREATE POLICY role_permissions_select ON role_permissions FOR SELECT 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY role_permissions_insert ON role_permissions FOR INSERT 
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY role_permissions_update ON role_permissions FOR UPDATE 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY role_permissions_delete ON role_permissions FOR DELETE 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- RLS policies for user_permission_overrides
ALTER TABLE user_permission_overrides ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_permission_overrides_select ON user_permission_overrides FOR SELECT 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY user_permission_overrides_insert ON user_permission_overrides FOR INSERT 
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY user_permission_overrides_update ON user_permission_overrides FOR UPDATE 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY user_permission_overrides_delete ON user_permission_overrides FOR DELETE 
    USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- Seed default permissions for system roles
-- These are inserted after system roles are created

-- Function to set up default permissions for a role
CREATE OR REPLACE FUNCTION setup_default_role_permissions(p_role_id UUID, p_tenant_id UUID, p_role_name VARCHAR)
RETURNS VOID AS $$
BEGIN
    -- Admin: full access to all modules
    IF p_role_name = 'Admin' OR p_role_name = 'SuperAdmin' THEN
        INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope)
        VALUES 
            (p_role_id, p_tenant_id, 'sites', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'orders', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'equipment', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'workers', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'livestock', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'weather', '["read"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'harvest', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'vineyard', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'olive', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'water', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'compliance', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'finance', '["create","read","update","delete","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'analytics', '["read","export"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'settings', '["read","update","manage"]'::jsonb, 'all'),
            (p_role_id, p_tenant_id, 'admin', '["create","read","update","delete","manage"]'::jsonb, 'all')
        ON CONFLICT (role_id, module) DO NOTHING;
    
    -- Manager: broad access but not full admin
    ELSIF p_role_name = 'Manager' THEN
        INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope)
        VALUES 
            (p_role_id, p_tenant_id, 'sites', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'orders', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'equipment', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'workers', '["create","read","update","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'livestock', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'weather', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'harvest', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'vineyard', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'olive', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'water', '["create","read","update","delete","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'compliance', '["create","read","update","manage"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'finance', '["read","update"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'analytics', '["read","export"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'settings', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'admin', '["read"]'::jsonb, 'tenant')
        ON CONFLICT (role_id, module) DO NOTHING;
    
    -- Worker: read and update for operational modules
    ELSIF p_role_name = 'Worker' THEN
        INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope)
        VALUES 
            (p_role_id, p_tenant_id, 'sites', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'orders', '["read","update"]'::jsonb, 'own'),
            (p_role_id, p_tenant_id, 'equipment', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'workers', '["read"]'::jsonb, 'own'),
            (p_role_id, p_tenant_id, 'livestock', '["read","update"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'weather', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'harvest', '["read","update"]'::jsonb, 'own'),
            (p_role_id, p_tenant_id, 'vineyard', '["read","update"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'olive', '["read","update"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'water', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'compliance', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'finance', '[]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'analytics', '[]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'settings', '[]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'admin', '[]'::jsonb, 'tenant')
        ON CONFLICT (role_id, module) DO NOTHING;
    
    -- Viewer: read-only access
    ELSIF p_role_name = 'Viewer' THEN
        INSERT INTO role_permissions (role_id, tenant_id, module, actions, scope)
        VALUES 
            (p_role_id, p_tenant_id, 'sites', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'orders', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'equipment', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'workers', '[]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'livestock', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'weather', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'harvest', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'vineyard', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'olive', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'water', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'compliance', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'finance', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'analytics', '["read"]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'settings', '[]'::jsonb, 'tenant'),
            (p_role_id, p_tenant_id, 'admin', '[]'::jsonb, 'tenant')
        ON CONFLICT (role_id, module) DO NOTHING;
    END IF;
END;
$$ LANGUAGE plpgsql;
