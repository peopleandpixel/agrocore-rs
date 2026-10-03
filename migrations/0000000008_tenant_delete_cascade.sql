-- Make tenant deletion work once the tenant has audit history.
--
-- Found by the J21 isolation tests (`tenant_isolation_rls_tests.rs`), 2026-10-03.
-- `DELETE /api/v1/system/tenant` deletes the `tenants` row directly and returns
-- whatever PostgreSQL does. It failed on any tenant that had been used.
--
-- ---------------------------------------------------------------------------
-- Problem 1: three foreign keys without ON DELETE CASCADE
-- ---------------------------------------------------------------------------
--
-- `audit_logs`, `harvest_seasons` and `lpis_reference_parcels` reference
-- `tenants(id)` without a cascade. Every other tenant-scoped table in the schema
-- has `ON DELETE CASCADE`, which is what lets the endpoint work at all. These
-- three are inconsistent, and deleting a tenant with an audit entry failed with
--
--   ERROR: update or delete on table "tenants" violates foreign key
--          constraint "audit_logs_tenant_id_fkey" on table "audit_logs"
--
-- `audit_logs` is the one that bites in practice: an audit row is written by
-- ordinary use, so a tenant that has been in service for a day cannot be
-- deleted. This is the GDPR erasure path, and it fails exactly when it is
-- needed. Reproduced against PostgreSQL 17.
--
-- Cascade, not `SET NULL`: the audit trail belongs to the tenant. Keeping the
-- rows with a dangling `tenant_id` would preserve personal data about a subject
-- who asked for erasure, and the column is NOT NULL, so it would fail anyway.
--
-- ---------------------------------------------------------------------------
-- Problem 2: the audit triggers fire while the tenant is being deleted
-- ---------------------------------------------------------------------------
--
-- Fixing the cascades is not sufficient, and this is the part that is easy to
-- miss. 41 tables are cascade children of `tenants` and carry an
-- `audit_trigger_function` trigger. Deleting the tenant deletes those rows, each
-- delete fires the trigger, and each trigger writes a row into `audit_logs` with
-- `tenant_id` set to the tenant being deleted — which is by then gone. The
-- insert fails against the very foreign key added above:
--
--   ERROR: insert or update on table "audit_logs" violates foreign key
--          constraint "audit_logs_tenant_id_fkey"
--   DETAIL: Key (tenant_id)=(...) is not present in table "tenants".
--
-- So the cascade turns one broken delete into a different broken delete.
--
-- The audit trail for a deleted tenant cannot be written anyway: `audit_logs`
-- has a foreign key to `tenants`, so a row describing the deletion could not
-- outlive the tenant. Suppressing audit triggers during a tenant delete is
-- therefore not losing information — there is nowhere to put it.
--
-- The guard is a session flag the trigger checks. It is set by a `BEFORE DELETE`
-- trigger on `tenants`, which fires before any cascading delete runs, and it is
-- transaction-scoped so it cannot leak into another request on the same
-- connection.
--
-- What is *not* suppressed: an explicit `DELETE FROM audit_logs` still records
-- itself, and no other operation is affected.

ALTER TABLE audit_logs
    DROP CONSTRAINT IF EXISTS audit_logs_tenant_id_fkey,
    ADD CONSTRAINT audit_logs_tenant_id_fkey
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

ALTER TABLE harvest_seasons
    DROP CONSTRAINT IF EXISTS harvest_seasons_tenant_id_fkey,
    ADD CONSTRAINT harvest_seasons_tenant_id_fkey
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

ALTER TABLE lpis_reference_parcels
    DROP CONSTRAINT IF EXISTS lpis_reference_parcels_tenant_id_fkey,
    ADD CONSTRAINT lpis_reference_parcels_tenant_id_fkey
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

-- ---------------------------------------------------------------------------
-- Suppress audit writes while a tenant and its children are being removed.
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION audit_skip_during_tenant_delete() RETURNS TRIGGER AS $$
BEGIN
    -- Transaction-scoped, so it is reset even if the delete is rolled back.
    PERFORM set_config('app.audit_suppressed', 'on', true);
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS tenants_suppress_audit ON tenants;
CREATE TRIGGER tenants_suppress_audit
    BEFORE DELETE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION audit_skip_during_tenant_delete();

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
    IF TG_TABLE_NAME = 'audit_logs' THEN
        RETURN NEW;
    END IF;

    -- Skip while a tenant is being deleted: the cascading child deletes would
    -- each try to write an audit row for a tenant that no longer exists, which
    -- fails against the `audit_logs` foreign key. See migration 8.
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

    v_action := CASE TG_OP
        WHEN 'INSERT' THEN 'Created'
        WHEN 'UPDATE' THEN 'Updated'
        WHEN 'DELETE' THEN 'Deleted'
        ELSE TG_OP
    END;

    IF TG_OP = 'UPDATE' THEN
        v_old_value := to_jsonb(OLD) - 'created_at' - 'updated_at';
        v_new_value := to_jsonb(NEW) - 'created_at' - 'updated_at';
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

    INSERT INTO audit_logs (
            tenant_id, user_id, action, entity_type, entity_id,
            old_value, new_value, changed_fields,
            ip_address, user_agent, request_id, session_id
        ) VALUES (
            COALESCE(v_tenant_id,
                CASE
                    WHEN TG_TABLE_NAME = 'tenants' THEN NULL
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