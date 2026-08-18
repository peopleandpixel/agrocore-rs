-- Complete the task_data shape used by the domain entity.
ALTER TABLE task_data
    ADD COLUMN IF NOT EXISTS pause_resume_cycles JSONB NOT NULL DEFAULT '[]'::JSONB,
    ADD COLUMN IF NOT EXISTS finished_for_day_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS handoff_to_worker_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS is_session_complete BOOLEAN NOT NULL DEFAULT FALSE;
