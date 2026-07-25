-- Migration: Row-Level Security (RLS) Policies for Multi-Tenant Isolation
-- This migration enables RLS on all tenant-scoped tables and creates policies
-- that enforce tenant isolation at the database level.

-- ============================================================
-- 1. Helper Functions
-- ============================================================

-- Function to get current tenant ID from session variable
-- This should be set by the application middleware on each request
CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to check if current user is a superadmin (bypasses RLS)
CREATE OR REPLACE FUNCTION is_superadmin()
RETURNS BOOLEAN AS $$
BEGIN
    RETURN current_setting('app.is_superadmin', true)::BOOLEAN;
EXCEPTION WHEN OTHERS THEN
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- 2. Enable RLS on all tenant-scoped tables
-- ============================================================

-- Core tables
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE sites ENABLE ROW LEVEL SECURITY;
ALTER TABLE equipment ENABLE ROW LEVEL SECURITY;
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_sites ENABLE ROW LEVEL SECURITY;

-- Weather tables
ALTER TABLE weather_stations ENABLE ROW LEVEL SECURITY;
ALTER TABLE weather_data ENABLE ROW LEVEL SECURITY;

-- Animal/Livestock tables
ALTER TABLE animals ENABLE ROW LEVEL SECURITY;

-- Task tables
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_data ENABLE ROW LEVEL SECURITY;

-- ============================================================
-- 3. RLS Policies
-- ============================================================

-- TENANTS: Superadmins see all, users see their own tenant
CREATE POLICY tenants_select ON tenants
    FOR SELECT USING (is_superadmin() OR id = get_current_tenant_id());

CREATE POLICY tenants_insert ON tenants
    FOR INSERT WITH CHECK (is_superadmin());

CREATE POLICY tenants_update ON tenants
    FOR UPDATE USING (is_superadmin() OR id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR id = get_current_tenant_id());

CREATE POLICY tenants_delete ON tenants
    FOR DELETE USING (is_superadmin());

-- USERS: Tenant users see users in their tenant, superadmins see all
CREATE POLICY users_select ON users
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY users_insert ON users
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY users_update ON users
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY users_delete ON users
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- SITES: Tenant isolation
CREATE POLICY sites_select ON sites
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY sites_insert ON sites
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY sites_update ON sites
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY sites_delete ON sites
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- EQUIPMENT: Tenant isolation
CREATE POLICY equipment_select ON equipment
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY equipment_insert ON equipment
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY equipment_update ON equipment
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY equipment_delete ON equipment
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ORDERS: Tenant isolation
CREATE POLICY orders_select ON orders
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY orders_insert ON orders
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY orders_update ON orders
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY orders_delete ON orders
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ORDER_SITES: Junction table - inherits from orders/sites
CREATE POLICY order_sites_select ON order_sites
    FOR SELECT USING (
        is_superadmin() OR
        order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY order_sites_insert ON order_sites
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) AND
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY order_sites_delete ON order_sites
    FOR DELETE USING (
        is_superadmin() OR
        order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

-- WEATHER_STATIONS: Tenant isolation
CREATE POLICY weather_stations_select ON weather_stations
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY weather_stations_insert ON weather_stations
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY weather_stations_update ON weather_stations
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY weather_stations_delete ON weather_stations
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- WEATHER_DATA: Tenant isolation (references weather_stations)
CREATE POLICY weather_data_select ON weather_data
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY weather_data_insert ON weather_data
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY weather_data_update ON weather_data
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY weather_data_delete ON weather_data
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        station_id IN (SELECT id FROM weather_stations WHERE tenant_id = get_current_tenant_id())
    );

-- ANIMALS: Tenant isolation
CREATE POLICY animals_select ON animals
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY animals_insert ON animals
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY animals_update ON animals
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY animals_delete ON animals
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- TASKS: Tenant isolation
CREATE POLICY tasks_select ON tasks
    FOR SELECT USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    );

CREATE POLICY tasks_insert ON tasks
    FOR INSERT WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY tasks_update ON tasks
    FOR UPDATE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id() OR
        order_id IN (SELECT id FROM orders WHERE tenant_id = get_current_tenant_id()) OR
        worker_id IN (SELECT id FROM users WHERE tenant_id = get_current_tenant_id()) OR
        site_id IN (SELECT id FROM sites WHERE tenant_id = get_current_tenant_id())
    )
    WITH CHECK (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

CREATE POLICY tasks_delete ON tasks
    FOR DELETE USING (
        is_superadmin() OR
        tenant_id = get_current_tenant_id()
    );

-- TASK_DATA: Tenant isolation
CREATE POLICY task_data_select ON task_data
    FOR SELECT USING (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY task_data_insert ON task_data
    FOR INSERT WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY task_data_update ON task_data
    FOR UPDATE USING (is_superadmin() OR tenant_id = get_current_tenant_id())
    WITH CHECK (is_superadmin() OR tenant_id = get_current_tenant_id());

CREATE POLICY task_data_delete ON task_data
    FOR DELETE USING (is_superadmin() OR tenant_id = get_current_tenant_id());

-- ============================================================
-- 4. Grant permissions
-- ============================================================

-- Grant usage on the helper functions to the application role
GRANT EXECUTE ON FUNCTION get_current_tenant_id() TO PUBLIC;
GRANT EXECUTE ON FUNCTION is_superadmin() TO PUBLIC;

-- ============================================================
-- 5. Documentation
-- ============================================================

COMMENT ON FUNCTION get_current_tenant_id() IS
'Returns the current tenant ID from session variable app.current_tenant_id.
Set by application middleware on each request. Returns NULL if not set.';

COMMENT ON FUNCTION is_superadmin() IS
'Returns true if current session is a superadmin (app.is_superadmin = true).
Used to bypass RLS for administrative operations.';

COMMENT ON POLICY tenants_select ON tenants IS 'Superadmins see all tenants; users see only their own tenant';
COMMENT ON POLICY users_select ON users IS 'Tenant users see users in their tenant; superadmins see all';
COMMENT ON POLICY sites_select ON sites IS 'Tenant isolation: users only see sites in their tenant';
COMMENT ON POLICY equipment_select ON equipment IS 'Tenant isolation: users only see equipment in their tenant';
COMMENT ON POLICY orders_select ON orders IS 'Tenant isolation: users only see orders in their tenant';
COMMENT ON POLICY weather_stations_select ON weather_stations IS 'Tenant isolation: users only see stations in their tenant';
COMMENT ON POLICY weather_data_select ON weather_data IS 'Tenant isolation via tenant_id or station ownership';
COMMENT ON POLICY animals_select ON animals IS 'Tenant isolation: users only see animals in their tenant';
COMMENT ON POLICY tasks_select ON tasks IS 'Tenant isolation: users see tasks in their tenant or related to their resources';
COMMENT ON POLICY task_data_select ON task_data IS 'Tenant isolation: users only see task data in their tenant';