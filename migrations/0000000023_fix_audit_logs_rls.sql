-- Migration 23: Fix audit_logs INSERT RLS policy
--
-- Problem: The audit_logs INSERT policy requires is_superadmin(), but the
-- database audit trigger fires for all users, including regular Admins.
-- When an Admin updates a user (e.g., toggling active/inactive), the trigger
-- tries to insert an audit log, but the RLS policy blocks it, causing a 500 error.
--
-- Solution: Allow INSERT for both SuperAdmin and Admin users.
-- The trigger runs as SECURITY DEFINER, so it uses the app.is_superadmin and
-- app.is_admin session settings to determine permissions.

DROP POLICY IF EXISTS audit_logs_insert ON audit_logs;
CREATE POLICY audit_logs_insert ON audit_logs FOR INSERT
    WITH CHECK (is_superadmin() OR current_setting('app.is_admin', true)::BOOLEAN);
