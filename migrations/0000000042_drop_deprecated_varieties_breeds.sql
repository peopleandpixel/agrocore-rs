-- Migration 42: Drop deprecated varieties and breeds tables
--
-- These tables were migrated to type_entries in migration 0041.
-- All code references have been removed. The tables are no longer needed.

-- Drop the deprecated tables
DROP TABLE IF EXISTS varieties CASCADE;
DROP TABLE IF EXISTS breeds CASCADE;

-- Clean up the sentinel UUID usage: any code that used
-- '00000000-0000-0000-0000-000000000000' for global varieties/breeds
-- should now use tenant_id = NULL in type_entries.
