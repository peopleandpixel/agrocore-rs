-- Migration: Automatic Audit Trail for All Entities
-- Creates audit_logs table with enhanced fields and PostgreSQL triggers for automatic auditing

-- ============================================================
-- 1. Enhanced audit_logs table
-- Columns added to existing audit_logs table (created in 2026072501)
-- Rename old columns first, then add any new ones that don't exist
-- ============================================================
-- Rename old_data/new_data to old_value/new_value if they still exist (from 2026072501)
ALTER TABLE audit_logs RENAME COLUMN IF EXISTS old_data TO old_value;
ALTER TABLE audit_logs RENAME COLUMN IF EXISTS new_data TO new_value;
-- Add columns that don't exist (after rename, if applicable)
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS old_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS new_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS changed_fields TEXT[];
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS session_id UUID;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS request_id UUID;

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_request ON audit_logs(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_session ON audit_logs(session_id);

-- ============================================================
-- 2. Audit trigger function
-- ============================================================
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
DECLARE
    v_old_value JSONB;
    v_new_value JSONB;
    v_changed_fields TEXT[];
    v_user_id UUID;
    v_tenant_id UUID;
    v_action TEXT;
    v_ip INET;
    v_user_agent TEXT;
    v_request_id UUID;
    v_session_id UUID;
BEGIN
    -- Skip audit for audit_logs table itself to prevent infinite recursion
    IF TG_TABLE_NAME = 'audit_logs' THEN
        RETURN NEW;
    END IF;

    -- Get context variables from session
    v_user_id := COALESCE(current_setting('app.current_user_id', true)::UUID, NULL);
    v_tenant_id := COALESCE(current_setting('app.current_tenant_id', true)::UUID, NULL);
    v_ip := COALESCE(current_setting('app.current_ip', true)::INET, NULL);
    v_user_agent := current_setting('app.current_user_agent', true);
    v_request_id := COALESCE(current_setting('app.current_request_id', true)::UUID, gen_random_uuid());
    v_session_id := COALESCE(current_setting('app.current_session_id', true)::UUID, NULL);

    -- Determine action
    v_action := CASE TG_OP
        WHEN 'INSERT' THEN 'Created'
        WHEN 'UPDATE' THEN 'Updated'
        WHEN 'DELETE' THEN 'Deleted'
        ELSE TG_OP
    END;

    -- Capture old and new values
    IF TG_OP = 'UPDATE' THEN
        v_old_value := to_jsonb(OLD) - 'created_at' - 'updated_at';
        v_new_value := to_jsonb(NEW) - 'created_at' - 'updated_at';
        
        -- Calculate changed fields
        SELECT ARRAY(
            SELECT key FROM jsonb_each(v_new_value) 
            WHERE value IS DISTINCT FROM (v_old_value -> key)
        ) INTO v_changed_fields;
    ELSIF TG_OP = 'INSERT' THEN
        v_new_value := to_jsonb(NEW) - 'created_at' - 'updated_at';
        v_old_value := NULL;
        v_changed_fields := ARRAY[]::TEXT[];
    ELSIF TG_OP = 'DELETE' THEN
        v_old_value := to_jsonb(OLD) - 'created_at' - 'updated_at';
        v_new_value := NULL;
        v_changed_fields := ARRAY[]::TEXT[];
    END IF;

    -- Insert audit log entry
    INSERT INTO audit_logs (
        tenant_id, user_id, action, entity_type, entity_id,
        old_value, new_value, changed_fields,
        ip_address, user_agent, request_id, session_id
    ) VALUES (
        COALESCE(v_tenant_id, 
            CASE 
                WHEN TG_TABLE_NAME = 'tenants' THEN NEW.id
                WHEN has_column(TG_TABLE_NAME, 'tenant_id') THEN 
                    COALESCE(NEW.tenant_id, OLD.tenant_id)
                ELSE NULL
            END
        ),
        v_user_id,
        v_action,
        TG_TABLE_NAME,
        COALESCE(NEW.id, OLD.id),
        v_old_value,
        v_new_value,
        v_changed_fields,
        v_ip,
        v_user_agent,
        v_request_id,
        v_session_id
    );

    RETURN CASE TG_OP
        WHEN 'DELETE' THEN OLD
        ELSE NEW
    END;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- 3. Helper function to check if column exists
-- ============================================================
CREATE OR REPLACE FUNCTION has_column(table_name TEXT, column_name TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_schema = 'public' 
        AND table_name = $1 
        AND column_name = $2
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- 4. Apply audit triggers to all tenant-scoped tables
-- ============================================================

DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN 
        SELECT table_name FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_type = 'BASE TABLE'
        AND table_name NOT IN (
            'audit_logs', 'spatial_ref_sys', 'schema_migrations',
            'user_sites', 'order_sites'  -- Junction tables handled by parent entities
        )
    LOOP
        -- Check if table has tenant_id column
        IF EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_schema = 'public' 
            AND table_name = t 
            AND column_name = 'tenant_id'
        ) THEN
            -- Drop existing trigger if exists
            EXECUTE format('DROP TRIGGER IF EXISTS audit_trigger_%I ON %I', t, t);
            
            -- Create audit trigger
            EXECUTE format(
                'CREATE TRIGGER audit_trigger_%I 
                 AFTER INSERT OR UPDATE OR DELETE ON %I 
                 FOR EACH ROW EXECUTE FUNCTION audit_trigger_function()', 
                t, t
            );
        END IF;
    END LOOP;
END $$;

-- ============================================================
-- 5. Special handling for junction tables (no tenant_id)
-- ============================================================

-- user_sites audit trigger
DROP TRIGGER IF EXISTS audit_trigger_user_sites ON user_sites;
CREATE TRIGGER audit_trigger_user_sites
    AFTER INSERT OR DELETE ON user_sites
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- order_sites audit trigger  
DROP TRIGGER IF EXISTS audit_trigger_order_sites ON order_sites;
CREATE TRIGGER audit_trigger_order_sites
    AFTER INSERT OR DELETE ON order_sites
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

-- ============================================================
-- 6. Audit log export view for auditors
-- ============================================================
CREATE OR REPLACE VIEW audit_log_export AS
SELECT 
    al.id,
    al.created_at,
    al.tenant_id,
    t.name AS tenant_name,
    al.user_id,
    u.email AS user_email,
    u.firstname || ' ' || u.lastname AS user_name,
    al.action,
    al.entity_type,
    al.entity_id,
    al.changed_fields,
    al.old_value,
    al.new_value,
    al.ip_address,
    al.user_agent,
    al.request_id,
    al.session_id
FROM audit_logs al
LEFT JOIN tenants t ON al.tenant_id = t.id
LEFT JOIN users u ON al.user_id = u.id
ORDER BY al.created_at DESC;

-- ============================================================
-- 7. Audit log export function for auditors (CSV/JSON)
-- ============================================================
CREATE OR REPLACE FUNCTION export_audit_logs(
    p_tenant_id UUID DEFAULT NULL,
    p_start_date TIMESTAMPTZ DEFAULT NULL,
    p_end_date TIMESTAMPTZ DEFAULT NULL,
    p_entity_type TEXT DEFAULT NULL,
    p_action TEXT DEFAULT NULL,
    p_format TEXT DEFAULT 'json'  -- 'json' or 'csv'
)
RETURNS TABLE (export_data TEXT) AS $$
DECLARE
    v_query TEXT;
    v_where_conditions TEXT[] := ARRAY[];
BEGIN
    -- Build dynamic WHERE clause
    IF p_tenant_id IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.tenant_id = %L', p_tenant_id);
    END IF;
    
    IF p_start_date IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.created_at >= %L', p_start_date);
    END IF;
    
    IF p_end_date IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.created_at <= %L', p_end_date);
    END IF;
    
    IF p_entity_type IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.entity_type = %L', p_entity_type);
    END IF;
    
    IF p_action IS NOT NULL THEN
        v_where_conditions := v_where_conditions || format('al.action = %L', p_action);
    END IF;

    v_query := 'SELECT ' || CASE 
        WHEN p_format = 'csv' THEN 
            'string_agg(quote_literal(al.id) || '','' || quote_literal(al.created_at) || '','' || 
                    quote_literal(t.name) || '','' || quote_literal(u.email) || '','' || 
                    quote_literal(al.action) || '','' || quote_literal(al.entity_type) || '','' || 
                    quote_literal(al.entity_id) || '','' || 
                    quote_literal(al.changed_fields) || '','' || 
                    quote_literal(al.old_value) || '','' || 
                    quote_literal(al.new_value), E''\n'')'
        ELSE 
            'json_agg(json_build_object(''id'', al.id, ''created_at'', al.created_at, ''tenant'', t.name, ''user'', u.email, ''action'', al.action, ''entity_type'', al.entity_type, ''entity_id'', al.entity_id, ''changed_fields'', al.changed_fields, ''old_value'', al.old_value, ''new_value'', al.new_value))'
    END || '
    FROM audit_logs al
    LEFT JOIN tenants t ON al.tenant_id = t.id
    LEFT JOIN users u ON al.user_id = u.id';

    IF array_length(v_where_conditions, 1) > 0 THEN
        v_query := v_query || ' WHERE ' || array_to_string(v_where_conditions, ' AND ');
    END IF;

    v_query := v_query || ' ORDER BY al.created_at DESC';

    RETURN QUERY EXECUTE v_query;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================
-- 8. Updated_at trigger function (reusable)
-- ============================================================
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at trigger to all tables with updated_at column
DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN 
        SELECT table_name FROM information_schema.columns 
        WHERE table_schema = 'public' 
        AND column_name = 'updated_at'
        AND table_name NOT IN ('audit_logs', 'spatial_ref_sys')
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS set_updated_at_%I ON %I', t, t);
        EXECUTE format('CREATE TRIGGER set_updated_at_%I BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()', t, t);
    END LOOP;
END $$;

-- ============================================================
-- 9. Audit summary view for dashboards
-- ============================================================
CREATE OR REPLACE VIEW audit_summary AS
SELECT 
    tenant_id,
    entity_type,
    action,
    COUNT(*) as count,
    MIN(created_at) as first_occurrence,
    MAX(created_at) as last_occurrence
FROM audit_logs
GROUP BY tenant_id, entity_type, action
ORDER BY tenant_id, entity_type, action;

-- ============================================================
-- 10. Grant permissions
-- ============================================================
GRANT SELECT ON audit_logs TO agrocore_app;
GRANT SELECT ON audit_log_export TO agrocore_app;
GRANT SELECT ON audit_summary TO agrocore_app;
GRANT EXECUTE ON FUNCTION export_audit_logs(UUID, TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT, TEXT) TO agrocore_app;