-- Fix: Update audit_trigger_function to check audit_suppressed flag
--
-- Migrations 8 and 13 updated this function, but the files were modified after
-- being applied, so sqlx skipped re-applying them. The database still has the
-- original version that does not check app.audit_suppressed and sets
-- tenant_id = NULL for the tenants table, causing FK violations during setup.
--
-- This migration replaces the trigger with the correct version that:
-- 1. Checks app.audit_suppressed and skips audit writes when suppressed
-- 2. Derives tenant_id from the tenants table row itself (NEW.id) instead of NULL
-- 3. Handles tables without tenant_id (like order_sites) via FK lookup

CREATE OR REPLACE FUNCTION public.audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
    v_tenant_id UUID;
    v_user_id UUID;
    v_action TEXT;
    v_old_value JSONB;
    v_new_value JSONB;
    v_changed_fields TEXT[];
    v_ip INET;
    v_user_agent TEXT;
    v_request_id UUID;
    v_session_id UUID;
    v_entity_id UUID;
BEGIN
    IF TG_TABLE_NAME = 'audit_logs' THEN
        RETURN NEW;
    END IF;

    -- Skip while audit is suppressed (e.g., during initial setup or tenant delete)
    IF current_setting('app.audit_suppressed', true) = 'on' THEN
        RETURN CASE TG_OP
            WHEN 'DELETE' THEN OLD
            ELSE NEW
        END;
    END IF;

    v_user_id := COALESCE(current_setting('app.current_user_id', true)::UUID, NULL);
    v_tenant_id := COALESCE(current_setting('app.current_tenant_id', true)::UUID, NULL);
    v_ip := COALESCE(current_setting('app.current_ip', true)::INET, NULL);
    v_user_agent := current_setting('app.current_user_agent', true);
    v_request_id := COALESCE(current_setting('app.current_request_id', true)::UUID, gen_random_uuid());
    v_session_id := COALESCE(current_setting('app.current_session_id', true)::UUID, NULL);

    -- Derive tenant_id from the row if not set in session
    IF v_tenant_id IS NULL THEN
        IF TG_TABLE_NAME = 'tenants' THEN
            -- When auditing the tenants table itself, use the newly created tenant's id
            v_tenant_id := (to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> 'id')::UUID;
        ELSIF has_column(TG_TABLE_NAME, 'tenant_id') THEN
            v_tenant_id := (to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> 'tenant_id')::UUID;
        ELSIF TG_TABLE_NAME = 'order_sites' THEN
            -- order_sites has no tenant_id; derive from the parent order
            v_tenant_id := (SELECT o.tenant_id
                              FROM orders o
                             WHERE o.id = (to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> 'order_id')::UUID);
        END IF;
    END IF;

    -- Derive entity_id
    IF has_column(TG_TABLE_NAME, 'id') THEN
        v_entity_id := CASE TG_OP
            WHEN 'DELETE' THEN (to_jsonb(OLD) ->> 'id')::UUID
            ELSE (to_jsonb(NEW) ->> 'id')::UUID
        END;
    ELSE
        -- Composite key: hash it to produce a stable UUID
        v_entity_id := (
            SELECT (substr(d.h, 1, 8) || '-' || substr(d.h, 9, 4) || '-' || substr(d.h, 13, 4)
                    || '-' || substr(d.h, 17, 4) || '-' || substr(d.h, 21, 12))::UUID
              FROM (
                    SELECT md5(string_agg(format('%s=%s', a.attname,
                             COALESCE(to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> a.attname, '')),
                             ',' ORDER BY a.attname)) AS h
                      FROM pg_index ix
                      JOIN pg_attribute a
                        ON a.attrelid = ix.indrelid
                       AND a.attnum = ANY(ix.indkey)
                     WHERE ix.indrelid = TG_RELID
                       AND ix.indisprimary
                   ) d
        );
    END IF;

    v_action := CASE TG_OP
        WHEN 'INSERT' THEN 'CREATE'
        WHEN 'UPDATE' THEN 'UPDATE'
        ELSE 'DELETE'
    END;

    IF TG_OP <> 'INSERT' THEN
        v_old_value := to_jsonb(OLD);
    END IF;
    IF TG_OP <> 'DELETE' THEN
        v_new_value := to_jsonb(NEW);
    END IF;

    IF TG_OP = 'UPDATE' THEN
        SELECT ARRAY(
            SELECT key FROM jsonb_each(v_new_value)
            WHERE value IS DISTINCT FROM (v_old_value -> key)
        ) INTO v_changed_fields;
    ELSE
        v_changed_fields := ARRAY[]::TEXT[];
    END IF;

    INSERT INTO audit_logs (
            tenant_id, user_id, action, entity_type, entity_id,
            old_value, new_value, changed_fields,
            ip_address, user_agent, request_id, session_id
        ) VALUES (
            v_tenant_id,
            v_user_id,
            v_action,
            TG_TABLE_NAME,
            v_entity_id,
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
$$;
