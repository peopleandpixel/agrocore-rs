-- Tasks table — schema update (tasks table created in 20240101_init.sql)
-- This migration adds missing columns and indexes to the existing tasks table
-- to support the full task management workflow (status, scheduling, duration tracking).
--
-- The original tasks table in 001_initial_schema.sql only had:
--   id, tenant_id, order_id, worker_id, site_id, description, started_at, ended_at,
--   created_at, updated_at
--
-- This migration adds: title, status, priority, scheduled_start/end, actual_start/end,
--   estimated_duration_hours, actual_duration_hours

-- Add columns that don't exist in the original tasks table
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS title TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'pending';
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS scheduled_start TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS scheduled_end TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS actual_start TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS actual_end TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS estimated_duration_hours DOUBLE PRECISION;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS actual_duration_hours DOUBLE PRECISION;

-- Ensure description allows NULL (was NOT NULL in init, now nullable for task updates)
ALTER TABLE tasks ALTER COLUMN description DROP NOT NULL;

-- Populate title from description for existing rows (title was missing in init schema)
UPDATE tasks SET title = description WHERE title IS NULL;
ALTER TABLE tasks ALTER COLUMN title SET NOT NULL;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_tasks_tenant ON tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_order ON tasks(order_id);
CREATE INDEX IF NOT EXISTS idx_tasks_worker ON tasks(worker_id);
CREATE INDEX IF NOT EXISTS idx_tasks_site ON tasks(site_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

-- The set_updated_at() function and set_updated_at_tasks trigger already exist
-- from 20240101_init.sql — no need to recreate them here.
