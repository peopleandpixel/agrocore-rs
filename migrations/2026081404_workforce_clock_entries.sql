-- Job & Arbeitskräfte-Management: Arbeitszeit-Erfassung (Clock-In/Clock-Out mit GPS)
-- and Arbeitskosten-Tracking (Stundensatz pro Arbeiter)

-- Add hourly_rate column to workers table for Arbeitskosten-Tracking
ALTER TABLE workers ADD COLUMN IF NOT EXISTS hourly_rate NUMERIC(10,2);

-- Clock entries table for Arbeitszeit-Erfassung (Clock-In/Clock-Out mit GPS)
CREATE TABLE IF NOT EXISTS clock_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    entry_type TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lat DOUBLE PRECISION,
    lng DOUBLE PRECISION,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_clock_entries_tenant ON clock_entries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_clock_entries_worker ON clock_entries(worker_id);
CREATE INDEX IF NOT EXISTS idx_clock_entries_type ON clock_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_clock_entries_timestamp ON clock_entries(timestamp);
CREATE INDEX IF NOT EXISTS idx_clock_entries_worker_time ON clock_entries(worker_id, timestamp);
