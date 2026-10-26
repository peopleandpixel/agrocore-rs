-- Migration 0038: Demo Mode Support
-- Adds is_demo column to tenants table for identifying demo tenants.
-- Demo tenants are excluded from backups and have special UI indicators.

BEGIN;

-- Add is_demo column to tenants table
ALTER TABLE IF EXISTS tenants ADD COLUMN IF NOT EXISTS is_demo BOOLEAN NOT NULL DEFAULT false;

-- Index for fast demo tenant lookups
CREATE INDEX IF NOT EXISTS idx_tenants_is_demo ON tenants(is_demo) WHERE is_demo = true;

COMMENT ON COLUMN tenants.is_demo IS 'True if this tenant contains demo data. Demo tenants are excluded from backups and have special UI indicators.';

COMMIT;
