-- Migration 28: Fix audit_logs INSERT RLS policy for all authenticated users
--
-- Problem: The audit_logs INSERT policy used `current_setting('app.is_admin', true)::BOOLEAN`,
-- but `app.is_admin` is never set in the session — only `app.is_superadmin` is set by
-- the TenantPool executor. When the setting doesn't exist, `current_setting` returns NULL,
-- and `NULL::BOOLEAN` evaluates to NULL, which fails the RLS check.
-- This caused all audit log inserts to fail for non-SuperAdmin users, breaking:
-- - User impersonation (audit log entry for impersonation event)
-- - User status toggling (audit trigger fires on UPDATE)
-- - Any other audited operation by non-SuperAdmin users
--
-- Solution: Allow any authenticated user to insert audit logs.
-- Audit logs are append-only and tenant-scoped — any user in the tenant should be able
-- to create audit entries. The tenant_id is already enforced by the application layer
-- (TenantPool pins the tenant on every connection).

DROP POLICY IF EXISTS audit_logs_insert ON audit_logs;
CREATE POLICY audit_logs_insert ON audit_logs FOR INSERT
    WITH CHECK (true);
