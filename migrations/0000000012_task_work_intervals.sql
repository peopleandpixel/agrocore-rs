-- Work intervals: the timestamps that a status flag cannot hold.
--
-- Why this table exists
-- ---------------------
-- `worker_task_statuses` has `status`, `started_at` and `completed_at` — one of each per
-- worker and task. That models "is it running, and since when", which is enough while a
-- worker starts and stops a task once. It stops being enough the moment a task runs more
-- than once, and that is not an edge case here: a task on a plot is expected to start and
-- stop repeatedly as the worker moves between plots and back.
--
-- With one pair of columns, a second start overwrites the first start time and the first
-- interval is simply lost. After five start/stop cycles the table says "Started" and
-- nothing about when any of it happened. Recording a plot visit by its entry and exit time
-- requires the intervals to be stored, not the current state.
--
-- `worker_task_statuses` is therefore left as the summary — what the UI shows now — and
-- this table becomes the record of what happened. `started_at` there can be derived from
-- the earliest open interval, so the two cannot disagree.
--
-- Dwell, not presence
-- --------------------
-- The trigger is not "the worker is inside the plot" but "the worker has been inside it
-- for at least `min_dwell_minutes`". A GPS fix on a tractor straddling a boundary, or a
-- worker walking to the gate, should not start a task.
--
-- End uses a longer tolerance than start, on purpose. Leaving to turn a tractor is normal
-- and must not stop the task, but walking to the canteen should. Hence
-- `min_dwell_minutes` (start) and `grace_minutes` (stop), both configurable.
--
-- The stop timestamp is the moment the worker left the plot, not the moment the grace
-- period expired. The grace period decides *whether* to stop, the exit time decides
-- *when* it counts as stopped — otherwise every stop would be padded by the grace period
-- and the recorded work time would grow by minutes on every cycle.
--
-- Only auto-closed intervals
-- -------------------------
-- `source` distinguishes an interval closed by the geometry engine from one closed by the
-- worker in the app or by an administrator. The rule is that the engine never completes a
-- task: an interval it closes is `open = false` with a stop time, but the task's own status
-- only moves to a running/stopped state. Completing is always manual, because the engine
-- cannot know whether the work was actually finished — presence in a plot means someone
-- worked there, not that the task is done.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. task_work_intervals
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS task_work_intervals (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- The `users` id, matching tasks.worker_id and worker_task_statuses.worker_id.
    worker_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- The plot the worker was in, which is what justifies the interval. Nullable because a
    -- manual start may not know the plot, and an interval without a plot is still valid
    -- time.
    site_id         UUID REFERENCES sites(id) ON DELETE SET NULL,
    started_at      TIMESTAMPTZ NOT NULL,
    -- NULL while the interval is still open. The engine closes an interval by setting
    -- this; it never sets the task's completion.
    stopped_at      TIMESTAMPTZ,
    -- How this interval came to exist.
    source          TEXT NOT NULL DEFAULT 'manual'
                    CHECK (source IN ('manual', 'auto_geometry', 'admin')),
    -- Why the engine closed it, for auditing a stop that a worker disputes:
    -- 'left_plot', 'grace_expired', 'worker_offline', 'manual'.
    stop_reason     TEXT,
    -- Minutes the worker was inside the plot before the start was accepted. Kept so a
    -- start that took longer than configured can be explained.
    dwell_minutes   DOUBLE PRECISION,
    note            TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A worker cannot have two open intervals on one task.
    --
    -- `UNIQUE (task_id, worker_id, stopped_at)` does NOT enforce this. In Postgres NULLs are
    -- distinct in a unique index by default, so two rows with `stopped_at IS NULL` are
    -- both accepted -- and the test confirmed it: a second open interval went in, and the
    -- constraint then fired on the *closed* rows instead, because closing them wrote the
    -- same timestamp twice.
    --
    -- `NULLS NOT DISTINCT` treats NULL as a value to compare, which is exactly the rule
    -- wanted: all open intervals for one worker and task share the key (task_id,
    -- worker_id, NULL) and collide, while closed intervals differ by their stop time.
    -- Requires PostgreSQL 15 or newer, which the project targets from 18 onwards.
    CONSTRAINT uq_open_interval_per_task_worker
        UNIQUE NULLS NOT DISTINCT (task_id, worker_id, stopped_at)
);

-- The lookup the engine runs on every position fix: open intervals for a worker.
CREATE INDEX IF NOT EXISTS idx_task_work_intervals_open
    ON task_work_intervals(tenant_id, worker_id) WHERE stopped_at IS NULL;

-- The reporting query: all intervals of a task, oldest first.
CREATE INDEX IF NOT EXISTS idx_task_work_intervals_task
    ON task_work_intervals(tenant_id, task_id, started_at DESC);

-- The reporting query: all intervals of a worker on a day.
CREATE INDEX IF NOT EXISTS idx_task_work_intervals_worker_time
    ON task_work_intervals(tenant_id, worker_id, started_at DESC);

COMMENT ON TABLE task_work_intervals IS
'Work intervals: one row per start/stop pair. This is the record; worker_task_statuses is a summary of it. An interval with stopped_at IS NULL is open. The engine closes intervals but never completes a task.';
COMMENT ON COLUMN task_work_intervals.source IS
'manual = the worker started it in the app; auto_geometry = the engine inferred it from the worker being on the plot; admin = an administrator adjusted it.';
COMMENT ON COLUMN task_work_intervals.stopped_at IS
'The moment the worker left the plot, NOT the moment the grace period expired. The grace period decides whether to stop; the exit time decides when. Padding the timestamp by the grace period would inflate recorded work time on every cycle.';

-- ---------------------------------------------------------------------------
-- 2. The trigger configuration
-- ---------------------------------------------------------------------------
-- Per order, because two tasks on the same farm can reasonably have different rules: an
-- olive harvest tolerates a long tractor turn, a hand-thinning task does not.
--
CREATE TABLE IF NOT EXISTS order_auto_automation_settings (
    order_id       UUID PRIMARY KEY REFERENCES orders(id) ON DELETE CASCADE,
    tenant_id      UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    -- Start automatically when a worker dwells on one of the order's plots.
    auto_start     BOOLEAN NOT NULL DEFAULT false,
    -- Close the interval automatically when the worker leaves.
    auto_stop      BOOLEAN NOT NULL DEFAULT true,
    -- Minutes inside the plot before an auto start is accepted. GPS jitter on a boundary
    -- and a worker walking to the gate must not start a task.
    min_dwell_minutes DOUBLE PRECISION NOT NULL DEFAULT 5,
    -- Minutes outside the plot tolerated before an interval is closed. Long enough to
    -- turn a tractor, short enough that walking to the canteen stops the clock.
    grace_minutes  DOUBLE PRECISION NOT NULL DEFAULT 15,
    -- Radius in metres around the worker's position used to decide "on the plot". The
    -- plot boundary is authoritative; this absorbs GPS error.
    presence_radius_m DOUBLE PRECISION NOT NULL DEFAULT 10,
    -- Also require the worker to be inside the plot rather than merely near it. Off means
    -- the radius alone decides, which is what a small or badly mapped plot needs.
    require_inside_plot BOOLEAN NOT NULL DEFAULT true,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A dwell longer than the grace would mean a task could never start.
    CONSTRAINT chk_dwell_positive CHECK (min_dwell_minutes >= 0),
    CONSTRAINT chk_grace_positive CHECK (grace_minutes >= 0),
    CONSTRAINT chk_radius_positive CHECK (presence_radius_m > 0),
    -- A grace shorter than the dwell would stop an interval in the same window it started.
    CONSTRAINT chk_grace_ge_dwell CHECK (grace_minutes >= min_dwell_minutes)
);

COMMENT ON TABLE order_auto_automation_settings IS
'Per-order configuration for geometry-driven task automation. A task on several plots starts when the worker is on any one of them; the plots are in order_sites.';

ALTER TABLE order_auto_automation_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE order_auto_automation_settings FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS order_auto_automation_settings_select ON order_auto_automation_settings;
CREATE POLICY order_auto_automation_settings_select ON order_auto_automation_settings FOR SELECT
    USING (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS order_auto_automation_settings_insert ON order_auto_automation_settings;
CREATE POLICY order_auto_automation_settings_insert ON order_auto_automation_settings FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS order_auto_automation_settings_update ON order_auto_automation_settings;
CREATE POLICY order_auto_automation_settings_update ON order_auto_automation_settings FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS order_auto_automation_settings_delete ON order_auto_automation_settings;
CREATE POLICY order_auto_automation_settings_delete ON order_auto_automation_settings FOR DELETE
    USING (tenant_id = get_current_tenant_id());

-- ---------------------------------------------------------------------------
-- 3. Row-level security on the intervals
-- ---------------------------------------------------------------------------
ALTER TABLE task_work_intervals ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_work_intervals FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS task_work_intervals_select ON task_work_intervals;
CREATE POLICY task_work_intervals_select ON task_work_intervals FOR SELECT
    USING (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_work_intervals_insert ON task_work_intervals;
CREATE POLICY task_work_intervals_insert ON task_work_intervals FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_work_intervals_update ON task_work_intervals;
CREATE POLICY task_work_intervals_update ON task_work_intervals FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_work_intervals_delete ON task_work_intervals;
CREATE POLICY task_work_intervals_delete ON task_work_intervals FOR DELETE
    USING (tenant_id = get_current_tenant_id());

DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname IN ('task_work_intervals', 'order_auto_automation_settings')
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO agrocore_app', r.relname);
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 4. Configurable defaults
-- ---------------------------------------------------------------------------
SET LOCAL app.audit_suppressed = 'on';

INSERT INTO system_settings (tenant_id, key, value, value_type, description, is_sensitive)
VALUES
    (NULL, 'task_automation.min_dwell_minutes', '5', 'number',
     'Default minutes a worker must stay on a plot before an automatic start is accepted. GPS jitter on a boundary and a worker walking to the gate must not start a task.', FALSE),
    (NULL, 'task_automation.grace_minutes', '15', 'number',
     'Default minutes outside a plot tolerated before an interval is closed. Long enough to turn a tractor, short enough that walking to the canteen stops the clock. Must be at least min_dwell_minutes.', FALSE),
    (NULL, 'task_automation.presence_radius_m', '10', 'number',
     'Default radius in metres around the worker position used to decide "on the plot". Absorbs GPS error; the plot boundary itself is authoritative.', FALSE),
    (NULL, 'task_automation.scan_interval_minutes', '5', 'number',
     'How often the automation engine evaluates worker positions. The stop timestamp is the true exit time from worker_locations, not the scan time, so a longer interval costs latency in deciding, not accuracy in the record.', FALSE),
    (NULL, 'task_automation.require_inside_plot', 'true', 'boolean',
     'Require the worker to be inside the plot boundary rather than merely within presence_radius_m of it. Set false for small or badly mapped plots.', FALSE),
    (NULL, 'task_automation.max_open_interval_hours', '12', 'number',
     'Safety valve: an interval still open after this many hours is closed automatically. A missing position stream would otherwise leave a task running indefinitely.', FALSE)
ON CONFLICT DO NOTHING;

RESET app.audit_suppressed;

-- ---------------------------------------------------------------------------
-- 5. Reporting view
-- ---------------------------------------------------------------------------
-- Duration per interval in minutes, so a report does not have to remember to subtract
-- and to handle the open case. `NULL` duration on an open interval is deliberate: an open
-- interval has no duration yet, and reporting a partial one as if it were complete would
-- overstate the day's work.
CREATE OR REPLACE VIEW task_work_interval_summary AS
SELECT
    i.id,
    i.tenant_id,
    i.task_id,
    i.worker_id,
    i.site_id,
    i.started_at,
    i.stopped_at,
    i.source,
    i.stop_reason,
    CASE
        WHEN i.stopped_at IS NULL THEN NULL
        ELSE EXTRACT(EPOCH FROM (i.stopped_at - i.started_at)) / 60.0
    END AS duration_minutes,
    (i.stopped_at IS NULL) AS is_open
FROM task_work_intervals i;

COMMENT ON VIEW task_work_interval_summary IS
'task_work_intervals with the duration computed. duration_minutes is NULL while the interval is open, on purpose: an open interval has no duration yet, and reporting a partial one as complete would overstate the work.';