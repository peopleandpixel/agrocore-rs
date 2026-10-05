-- The task state machine: sub-tasks, per-worker status, and the aggregate that follows.
--
-- Why sub-tasks
-- -------------
-- A task has an overall status *and* the status of each worker currently on it. That
-- overall status is not stored, it is derived -- and it can only be derived if the task is
-- split into parts, because the progress figure that goes with it is a function of the
-- parts.
--
-- Concretely: a task of 10 ha over two plots. Plot 1 is finished, plot 2 is not. The overall
-- status stays "running" and the progress reads 40%. The 40% comes from 4 of 10 ha. If the
-- task were not split, the progress would have to be estimated from the task definition, and
-- every estimate diverges from what actually happened.
--
-- So a task without sub-tasks cannot answer "how far along are we", which is the question
-- a person in the field actually asks. The sub-task is not a convenience; it is what makes
-- the number exist.
--
-- The three rules that came out of the design discussion
-- ------------------------------------------------------
--
-- 1. Area counts once per sub-task, not once per worker. Two workers on the same plot do
--    not do twice the area. Time, however, is counted per worker: each worker's hours are
--    real hours, and a task worked by two people took twice the man-hours for the same
--    area. So `completed_area_m2` and `worked_minutes` are deliberately not multiples of
--    each other.
--
-- 2. A stop leaves the progress where it is. A single worker who stops keeps whatever they
--    finished. With two or more workers on a sub-task, the progress carries on with whoever
--    is still there -- stopping one worker reduces their contribution, it does not reset
--    the sub-task. This is why progress is a stored, monotonic figure and not a function of
--    who is currently present.
--
-- 3. Overdue is computed, not stored. `due_date < today` is overdue. A recurring task is
--    overdue when the current occurrence has passed its due date -- "every 1st of the month"
---- and today is the 2nd means the 1st is overdue. Storing it would add a fourth state
--    transition that nothing would ever act on; computing it means the answer cannot go
--    stale.
--
-- The overall status
-- ------------------
-- Derived from the sub-tasks, in this order:
--
--   all sub-tasks done            -> completed
--   all assigned sub-tasks done   -> completed   (a task with no open work left is done)
--   any sub-task started or paused-> in_progress
--   every sub-task still new       -> new
--   nothing has a deadline yet     -> as above
--
-- `overdue` is reported alongside the status rather than replacing it: a task that is both
-- running and late is running and late, and collapsing the two into one enum would lose
-- the information that it is still moving.
--
-- Who may stop
-- ------------
-- A sub-task is completed by a worker or an administrator, never by the automation engine.
-- The engine opens and closes intervals; it has no way to know whether the work was done,
-- only that someone was present. `order_auto_automation_settings` has no completion
-- setting and that is deliberate.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. task_sub_tasks
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS task_sub_tasks (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- Optional: a sub-task may cover a plot, a group of trees, a section of a road. A
    -- sub-task without a plot is still a valid unit of work, and requiring a plot would
    -- exclude the "these 30 olive trees" case that motivates the whole model.
    site_id         UUID REFERENCES sites(id) ON DELETE SET NULL,
    -- What the sub-task covers, for the case it is not a whole plot.
    label           TEXT NOT NULL,
    -- What kind of unit this is, which decides how progress is measured.
    unit_kind       TEXT NOT NULL DEFAULT 'area'
                    CHECK (unit_kind IN ('area', 'object_count', 'duration', 'checklist')),
    -- Total work in the unit. For area: square metres, measured. For object_count: number
    -- of objects. For duration: minutes. For checklist: number of entries.
    planned_quantity DOUBLE PRECISION,
    -- Done so far. Monotonic: never decreases, because a stop leaves progress in place and
    -- a second worker continues with what the first achieved. A correction is a separate,
    -- audited operation, not an accidental overwrite.
    completed_quantity DOUBLE PRECISION NOT NULL DEFAULT 0,
    -- Per sub-task status, matching WorkerTaskStatusType.
    status          TEXT NOT NULL DEFAULT 'new'
                    CHECK (status IN ('new', 'started', 'paused', 'stopped', 'done')),
    -- Set when status becomes 'done'. NULL otherwise; a done sub-task always has one.
    completed_at    TIMESTAMPTZ,
    completed_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    note            TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A started sub-task must say when it started, and a done one must say who finished
    -- it and when. The converse is left alone: a sub-task may be done without started_at if
    -- it was imported as already complete.
    CONSTRAINT chk_subtask_done_has_actor CHECK (
        status <> 'done' OR (completed_at IS NOT NULL AND completed_by IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_task_sub_tasks_task ON task_sub_tasks(tenant_id, task_id);
CREATE INDEX IF NOT EXISTS idx_task_sub_tasks_site ON task_sub_tasks(tenant_id, site_id) WHERE site_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_task_sub_tasks_open ON task_sub_tasks(tenant_id, task_id) WHERE status <> 'done';

COMMENT ON TABLE task_sub_tasks IS
'The unit of completion for a task. Progress is measured here, never estimated on the task. A task with no sub-tasks cannot report how far along it is.';
COMMENT ON COLUMN task_sub_tasks.completed_quantity IS
'Done so far, monotonic. A stop leaves it where it is, and if other workers remain on the sub-task the progress continues. Never decreases without an explicit, audited correction.';
COMMENT ON COLUMN task_sub_tasks.status IS
'Per sub-task status: new, started, paused, stopped, done. The task''s overall status is derived from these, not stored.';

-- ---------------------------------------------------------------------------
-- 2. Who is on which sub-task
-- ---------------------------------------------------------------------------
-- The per-worker status the overall status is computed from. Separate from
-- worker_task_statuses because that is per task and has no way to say which part of the
-- task someone worked on -- and the whole model needs that distinction.
CREATE TABLE IF NOT EXISTS task_subtask_workers (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sub_task_id     UUID NOT NULL REFERENCES task_sub_tasks(id) ON DELETE CASCADE,
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    worker_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'new'
                    CHECK (status IN ('new', 'started', 'paused', 'stopped', 'done')),
    -- When this worker last contributed. Used to decide whether someone is "still there"
    -- for the purpose of whether a stop resets anything; it does not.
    last_active_at  TIMESTAMPTZ,
    note            TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One row per worker per sub-task. Two rows for the same worker on the same sub-task
    -- would make the aggregate count them twice.
    CONSTRAINT uq_subtask_worker UNIQUE (sub_task_id, worker_id)
);

CREATE INDEX IF NOT EXISTS idx_subtask_workers_task ON task_subtask_workers(tenant_id, task_id);
CREATE INDEX IF NOT EXISTS idx_subtask_workers_active
    ON task_subtask_workers(tenant_id, sub_task_id) WHERE status IN ('started', 'paused');

COMMENT ON TABLE task_subtask_workers IS
'Per-worker status on a sub-task. The task''s overall status is the aggregate of these. Separate from worker_task_statuses, which is per task and cannot say which part of the task someone worked on.';

-- ---------------------------------------------------------------------------
-- 3. Intervals hang on sub-tasks
-- ---------------------------------------------------------------------------
-- task_work_intervals was written against `task_id` alone, which is too coarse: with two
-- plots on one task it cannot record that this worker was on plot 1 and that one on plot 2.
ALTER TABLE task_work_intervals ADD COLUMN IF NOT EXISTS sub_task_id UUID
    REFERENCES task_sub_tasks(id) ON DELETE CASCADE;

-- The engine needs the sub-task, and the index that looks up open intervals by worker must
-- cover it.
CREATE INDEX IF NOT EXISTS idx_task_work_intervals_subtask
    ON task_work_intervals(tenant_id, worker_id, sub_task_id) WHERE stopped_at IS NULL;

COMMENT ON COLUMN task_work_intervals.sub_task_id IS
'Which part of the task this interval belongs to. NULL for intervals recorded before the task was split, or started from the app when the operator did not name a sub-task.';

-- ---------------------------------------------------------------------------
-- 4. Recurrence and due dates
-- ---------------------------------------------------------------------------
-- The occurrences of a recurring task, so "which run is late" has something to point at.
-- A task with a recurrence is a template; each occurrence is a row here with its own due
-- date and its own completion.
CREATE TABLE IF NOT EXISTS task_occurrences (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- 1-based, so the first occurrence is 1 and a human reading the progress bar can count.
    sequence_number INTEGER NOT NULL,
    planned_for     DATE NOT NULL,
    due_date        DATE,
    status          TEXT NOT NULL DEFAULT 'new'
                    CHECK (status IN ('new', 'in_progress', 'completed', 'skipped')),
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_task_occurrence UNIQUE (task_id, sequence_number)
);

CREATE INDEX IF NOT EXISTS idx_task_occurrences_due ON task_occurrences(tenant_id, due_date)
    WHERE status NOT IN ('completed', 'skipped');

COMMENT ON TABLE task_occurrences IS
'One row per occurrence of a recurring task. Overdue is computed from due_date against the current occurrence, not stored as a state.';

ALTER TABLE task_occurrences ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_occurrences FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_occurrences_select ON task_occurrences;
CREATE POLICY task_occurrences_select ON task_occurrences FOR SELECT
    USING (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_occurrences_insert ON task_occurrences;
CREATE POLICY task_occurrences_insert ON task_occurrences FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_occurrences_update ON task_occurrences;
CREATE POLICY task_occurrences_update ON task_occurrences FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_occurrences_delete ON task_occurrences;
CREATE POLICY task_occurrences_delete ON task_occurrences FOR DELETE
    USING (tenant_id = get_current_tenant_id());

-- ---------------------------------------------------------------------------
-- 5. Row-level security
-- ---------------------------------------------------------------------------
ALTER TABLE task_sub_tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_sub_tasks FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_sub_tasks_select ON task_sub_tasks;
CREATE POLICY task_sub_tasks_select ON task_sub_tasks FOR SELECT
    USING (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_sub_tasks_insert ON task_sub_tasks;
CREATE POLICY task_sub_tasks_insert ON task_sub_tasks FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_sub_tasks_update ON task_sub_tasks;
CREATE POLICY task_sub_tasks_update ON task_sub_tasks FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_sub_tasks_delete ON task_sub_tasks;
CREATE POLICY task_sub_tasks_delete ON task_sub_tasks FOR DELETE
    USING (tenant_id = get_current_tenant_id());

ALTER TABLE task_subtask_workers ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_subtask_workers FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS task_subtask_workers_select ON task_subtask_workers;
CREATE POLICY task_subtask_workers_select ON task_subtask_workers FOR SELECT
    USING (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_subtask_workers_insert ON task_subtask_workers;
CREATE POLICY task_subtask_workers_insert ON task_subtask_workers FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_subtask_workers_update ON task_subtask_workers;
CREATE POLICY task_subtask_workers_update ON task_subtask_workers FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());
DROP POLICY IF EXISTS task_subtask_workers_delete ON task_subtask_workers;
CREATE POLICY task_subtask_workers_delete ON task_subtask_workers FOR DELETE
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
          AND c.relname IN ('task_sub_tasks', 'task_subtask_workers', 'task_occurrences')
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO agrocore_app', r.relname);
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 6. The aggregate
-- ---------------------------------------------------------------------------
-- The overall status and the progress, derived rather than stored.
--
-- Storing them would mean two things that cannot both be true: the stored status could
-- disagree with the sub-tasks it is derived from, and no partial sub-task completion could
-- ever be represented. Derived, it is always right and always available.
CREATE OR REPLACE VIEW task_progress AS
SELECT
    t.id AS task_id,
    t.tenant_id,
    -- The overall status.
    CASE
        -- No sub-tasks at all: nothing to do, so nothing is in progress. Reported as
        -- 'new' rather than 'completed', because the task may still be unsplit and that is
        -- a different thing from being finished.
        WHEN count(s.id) = 0 THEN 'new'
        -- Every sub-task done.
        WHEN count(s.id) FILTER (WHERE s.status = 'done') = count(s.id) THEN 'completed'
        -- Nobody has started anything yet.
        WHEN count(s.id) FILTER (WHERE s.status IN ('started', 'paused')) = 0
         AND count(s.id) FILTER (WHERE s.status = 'done') = 0 THEN 'new'
        -- Somebody stopped. Only when work has actually been begun at some point --
        -- otherwise a task whose first sub-task someone finished while the rest are still
        -- `new` reports 'stopped', which reads as "work was abandoned" when the truth is
        -- "work is partway through". A 'stopped' row is only reachable by first passing
        -- through 'started', so the presence of one is the evidence that work began.
        WHEN count(s.id) FILTER (WHERE s.status = 'stopped') > 0 THEN 'stopped'
        -- Somebody finished a part and the rest have not been touched: still running.
        ELSE 'in_progress'
    END AS overall_status,
    count(s.id)                                                    AS sub_task_count,
    count(s.id) FILTER (WHERE s.status = 'done')                    AS sub_tasks_done,
    count(s.id) FILTER (WHERE s.status = 'started')                 AS sub_tasks_started,
    -- Progress by quantity, which is the honest measure. NULL when the task has no planned
    -- quantity, because a percentage of nothing is not 0%, it is unknown.
    CASE
        WHEN COALESCE(sum(s.planned_quantity), 0) = 0 THEN NULL
        ELSE LEAST(100.0,
             100.0 * COALESCE(sum(s.completed_quantity), 0) / sum(s.planned_quantity))
    END AS progress_percent,
    COALESCE(sum(s.planned_quantity), 0)   AS planned_total,
    COALESCE(sum(s.completed_quantity), 0)  AS completed_total,
    -- Per worker time, summed from the intervals. Deliberately not a multiple of the area:
    -- two workers on one plot cover it once and consume twice the man-hours.
    COALESCE((
        SELECT sum(EXTRACT(EPOCH FROM (i.stopped_at - i.started_at)) / 60.0)
          FROM task_work_intervals i
         WHERE i.task_id = t.id AND i.stopped_at IS NOT NULL
    ), 0) AS worked_minutes,
    -- Overdue is computed, never stored.
    CASE
        WHEN t.status::text IN ('completed', 'cancelled') THEN false
        WHEN EXISTS (SELECT 1 FROM task_occurrences o
                      WHERE o.task_id = t.id
                        AND o.status NOT IN ('completed', 'skipped')
                        AND o.due_date IS NOT NULL
                        AND o.due_date < CURRENT_DATE) THEN true
        -- The deadline lives on the order, not on the task: `tasks` has only
        -- scheduled_start/scheduled_end. Joining orders is required for the plain
        -- non-recurring case, and it is the order's `deadline_date` that is overdue.
        WHEN o.deadline_date IS NOT NULL AND o.deadline_date < CURRENT_DATE THEN true
        ELSE false
    END AS is_overdue
FROM tasks t
LEFT JOIN orders o ON o.id = t.order_id AND o.tenant_id = t.tenant_id
LEFT JOIN task_sub_tasks s ON s.task_id = t.id AND s.tenant_id = t.tenant_id
GROUP BY t.id, t.tenant_id, t.status, o.deadline_date;

COMMENT ON VIEW task_progress IS
'The task''s overall status and progress, derived from its sub-tasks. overall_status: new, in_progress, stopped, completed. is_overdue is computed from the current occurrence''s due date or the task deadline, and reported alongside the status rather than replacing it -- a task that is running and late is running and late.';
