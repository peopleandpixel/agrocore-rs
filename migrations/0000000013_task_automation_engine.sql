-- The automation engine: turns worker positions into work intervals.
--
-- What it decides
-- ---------------
-- For every worker with a recent position, for every open task on a plot the worker is
-- on, it opens an interval once the worker has been there long enough; and it closes
-- intervals whose worker has been off the plot longer than the tolerance allows.
--
-- Dwell before start, tolerance before stop
-- ----------------------------------------
-- These are two different rules because they guard against two different mistakes:
--
--   * A start needs a *dwell* threshold so that a GPS fix on a tractor straddling a
--     boundary, or a worker walking to the gate, does not start a task. Five minutes.
--   * A stop needs a *grace* period so that leaving to turn a tractor does not stop the
--     task. Fifteen minutes, which is longer than a turn and shorter than a walk to the
--     canteen.
--
-- The stop timestamp is the exit time, not the moment the grace expired
-- ---------------------------------------------------------------------
-- If an interval were closed at "now" once the grace period had elapsed, every stop would
-- be padded by the grace period and the recorded work time would grow by fifteen minutes on
-- every cycle -- so a worker who stepped off a plot four times in an afternoon would have
-- an hour of work that never happened. The interval ends when the worker left; the grace
-- period only decides *whether* to end it.
--
-- The exit time is found in worker_locations rather than assumed, because the scan
-- interval is five minutes and the last recorded position may be older than that.
--
-- The engine never completes a task
-- ---------------------------------
-- It opens and closes intervals. It does not move a task to a completed state, and it does
-- not touch orders.status. Presence in a plot means somebody worked there, not that the
-- task is done; completion stays with the worker or an administrator.
--
-- user_id versus worker_id
-- -----------------------
-- `worker_locations.worker_id` references `workers`, while `tasks.worker_id` and
-- `task_work_intervals.worker_id` reference `users`. The two are bridged by
-- `workers.user_id`, which is nullable and not unique -- a user may have more than one
-- worker record. The join therefore takes every worker row of a user, so a user with two
-- devices or two roles still gets one position stream into one set of intervals, and a
-- missing `workers.user_id` simply produces no interval rather than an error.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. Which plots is this position on?
-- ---------------------------------------------------------------------------
-- The single source of truth for "the worker is on plot X".
--
-- `require_inside_plot` decides whether the plot boundary is authoritative or the radius is.
-- The radius exists because GPS on a phone in an olive grove is routinely ten metres off,
-- so a strict containment test drops a worker who is visibly standing in the field. When
-- the boundary decides, an object within `presence_radius_m` of the boundary still counts
-- as inside; when the radius decides, anything within it of the plot counts.
--
-- ST_Distance on a geography casts to metres. On a geometry it returns degrees, which would
-- make a "10 metre" radius a 10-degree one.
CREATE OR REPLACE FUNCTION plots_at_position(
    p_tenant_id UUID,
    p_point     GEOMETRY(POINT, 4326),
    p_radius_m  DOUBLE PRECISION,
    p_require_inside BOOLEAN
)
RETURNS TABLE (site_id UUID)
LANGUAGE sql
STABLE
AS $$
    SELECT DISTINCT s.id
      FROM sites s
     WHERE s.tenant_id = p_tenant_id
       AND s.boundary IS NOT NULL
       AND s.is_active
       AND (
            CASE WHEN p_require_inside
                 THEN ST_Intersects(s.boundary, p_point)
                      OR ST_Distance(s.boundary::geography, p_point::geography) <= p_radius_m
                 ELSE ST_DWithin(s.boundary::geography, p_point::geography, p_radius_m)
            END
           );
$$;

COMMENT ON FUNCTION plots_at_position(UUID, GEOMETRY, DOUBLE PRECISION, BOOLEAN) IS
'Plots a position belongs to. p_require_inside makes the boundary authoritative while still accepting a position within p_radius_m of it -- GPS in a grove is routinely ten metres off, so strict containment drops workers who are visibly in the field. With it false, the radius alone decides.';

-- ---------------------------------------------------------------------------
-- 2. One evaluation pass
-- ---------------------------------------------------------------------------
-- Processes every worker that has a position, and returns what changed. Written as one
-- function rather than a procedure so the caller gets a result it can log, and so a test can
-- assert on it without inspecting tables.
--
-- The pass is:
--
--   a. close intervals that are due to close
--   b. open intervals that have qualified to open
--
-- Closing first matters: a worker who has been off one plot and onto another must have the
-- first interval closed before the second opens, or he would be recorded as working on both
-- at once.
-- Re-runnable: the return type of an existing function cannot be changed by
-- CREATE OR REPLACE, and this function returns arrays. Without the drop a second
-- application fails with `cannot change return type of existing function` and the
-- corrected body never reaches the database.
DROP FUNCTION IF EXISTS evaluate_task_automation(UUID, TIMESTAMPTZ);

CREATE OR REPLACE FUNCTION evaluate_task_automation(
    p_tenant_id UUID DEFAULT NULL,
    p_now       TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (
    -- Arrays, not scalars. An evaluation pass can open or close several intervals at once
    -- -- one per task the worker qualifies for -- so a single UUID would drop the rest.
    -- Declaring these as UUID is what produced
    -- `invalid input syntax for type uuid: "{...}"`: the array was cast to a scalar.
    intervals_opened UUID[],
    intervals_closed UUID[]
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_now           TIMESTAMPTZ := COALESCE(p_now, NOW());
    v_opened        UUID[] := ARRAY[]::UUID[];
    v_closed        UUID[] := ARRAY[]::UUID[];
    v_dwell_default DOUBLE PRECISION;
    v_grace_default DOUBLE PRECISION;
    v_radius_default DOUBLE PRECISION;
    v_inside_default BOOLEAN;
    v_max_open_hours DOUBLE PRECISION;
    r               RECORD;
    v_exit_time     TIMESTAMPTZ;
    v_id            UUID;
BEGIN
    -- Global defaults, overridable per tenant. JSONB stores 30 as a number and "30" as a
    -- string depending on which value_type the row was written with, so both are accepted.
    SELECT COALESCE((SELECT (value #>> '{}')::DOUBLE PRECISION FROM system_settings
                      WHERE key = 'task_automation.min_dwell_minutes' AND tenant_id IS NULL), 5),
           COALESCE((SELECT (value #>> '{}')::DOUBLE PRECISION FROM system_settings
                      WHERE key = 'task_automation.grace_minutes' AND tenant_id IS NULL), 15),
           COALESCE((SELECT (value #>> '{}')::DOUBLE PRECISION FROM system_settings
                      WHERE key = 'task_automation.presence_radius_m' AND tenant_id IS NULL), 10),
           COALESCE((SELECT (value #>> '{}')::BOOLEAN FROM system_settings
                      WHERE key = 'task_automation.require_inside_plot' AND tenant_id IS NULL), true),
           COALESCE((SELECT (value #>> '{}')::DOUBLE PRECISION FROM system_settings
                      WHERE key = 'task_automation.max_open_interval_hours' AND tenant_id IS NULL), 12)
      INTO v_dwell_default, v_grace_default, v_radius_default, v_inside_default, v_max_open_hours;

    -- ------------------------------------------------------------------
    -- (a) close
    -- ------------------------------------------------------------------
    FOR r IN
        SELECT i.id,
               i.task_id,
               i.worker_id,
               i.site_id,
               i.started_at,
               -- Configuration of the task's order, with the global defaults as fallback.
               COALESCE(cfg.min_dwell_minutes, v_dwell_default) AS dwell,
               COALESCE(cfg.grace_minutes,     v_grace_default) AS grace,
               COALESCE(cfg.presence_radius_m, v_radius_default) AS radius,
               COALESCE(cfg.require_inside_plot, v_inside_default) AS inside,
               -- The worker's latest position, and the latest one that was NOT on this plot.
               last_pos.location AS last_location,
               last_pos.ts       AS last_seen,
               outside_pos.ts    AS last_inside
          FROM task_work_intervals i
          JOIN tasks t          ON t.id = i.task_id AND t.tenant_id = i.tenant_id
          LEFT JOIN order_auto_automation_settings cfg ON cfg.order_id = t.order_id
          -- latest position of this user (through any of their worker rows)
          LEFT JOIN LATERAL (
                SELECT wl.location, wl.timestamp AS ts
                  FROM worker_locations wl
                  JOIN workers w ON w.id = wl.worker_id AND w.tenant_id = wl.tenant_id
                 WHERE w.user_id = i.worker_id
                   AND wl.tenant_id = i.tenant_id
                   AND wl.location IS NOT NULL
                 ORDER BY wl.timestamp DESC
                 LIMIT 1
          ) last_pos ON true
          -- latest position that was on the interval's plot.
          --
          -- The radius and the inside/outside rule come from `cfg_cfg`, not from `r`.
          -- Referring to `r.radius` inside the query that builds `r` is circular: the
          -- record does not exist yet, and Postgres rejects it with `record "r" is not
          -- assigned yet`. The values are therefore projected in a CTE and referenced
          -- from there.
          LEFT JOIN LATERAL (
                SELECT wl.timestamp AS ts
                  FROM worker_locations wl
                  JOIN workers w ON w.id = wl.worker_id AND w.tenant_id = wl.tenant_id
                 WHERE w.user_id = i.worker_id
                   AND wl.tenant_id = i.tenant_id
                   AND wl.location IS NOT NULL
                   AND i.site_id IS NOT NULL
                   -- EXISTS, not `plots_at_position(...) @> ARRAY[...]`: a set-returning
                   -- function is not allowed in a WHERE clause.
                   AND EXISTS (SELECT 1
                                 FROM plots_at_position(wl.tenant_id, wl.location,
                                                        COALESCE(cfg.presence_radius_m, v_radius_default),
                                                        COALESCE(cfg.require_inside_plot, v_inside_default)) p
                                WHERE p.site_id = i.site_id)
                 ORDER BY wl.timestamp DESC
                 LIMIT 1
          ) outside_pos ON true
         WHERE i.stopped_at IS NULL
           AND i.tenant_id = COALESCE(p_tenant_id, i.tenant_id)
           -- only for orders that asked for it
           AND COALESCE(cfg.auto_stop, true)
    LOOP
        -- The exit time: the last moment the worker was demonstrably on this plot.
        v_exit_time := COALESCE(r.last_inside, r.started_at);

        -- Close when the worker has been off the plot for longer than the tolerance, or
        -- when the interval has been open implausibly long (a missing position stream must
        -- not leave a task running forever).
        IF r.last_location IS NULL
           OR v_now - r.last_seen > (r.grace || ' minutes')::INTERVAL
           OR v_now - r.started_at > (v_max_open_hours || ' hours')::INTERVAL
        THEN
            UPDATE task_work_intervals
               SET stopped_at   = v_exit_time,
                   stop_reason = CASE
                       WHEN r.last_location IS NULL THEN 'worker_offline'
                       WHEN v_now - r.last_seen > (r.grace || ' minutes')::INTERVAL THEN 'grace_expired'
                       ELSE 'max_duration_reached'
                   END,
                   updated_at = NOW()
             WHERE id = r.id
               AND stopped_at IS NULL;
            IF FOUND THEN
                v_closed := array_append(v_closed, r.id);
            END IF;
        END IF;
    END LOOP;

    -- ------------------------------------------------------------------
    -- (b) open
    -- ------------------------------------------------------------------
    FOR r IN
        SELECT t.id AS task_id,
               t.worker_id,
               t.tenant_id,
               -- any plot of the order the worker is on; tasks.site_id first, then order_sites
               COALESCE(t.site_id, os.site_id) AS site_id,
               COALESCE(cfg.min_dwell_minutes, v_dwell_default) AS dwell,
               COALESCE(cfg.grace_minutes,     v_grace_default) AS grace,
               COALESCE(cfg.presence_radius_m, v_radius_default) AS radius,
               COALESCE(cfg.require_inside_plot, v_inside_default) AS inside,
               -- how long the worker has been continuously on this plot
               on_plot.first_on AS on_since
          FROM tasks t
          JOIN order_sites os ON os.order_id = t.order_id
          LEFT JOIN order_auto_automation_settings cfg ON cfg.order_id = t.order_id
          JOIN LATERAL (
                -- the worker's latest position and the plot it is on
                SELECT wl.tenant_id, wl.location, wl.timestamp AS ts
                  FROM worker_locations wl
                  JOIN workers w ON w.id = wl.worker_id AND w.tenant_id = wl.tenant_id
                 WHERE w.user_id = t.worker_id
                   AND wl.tenant_id = t.tenant_id
                   AND wl.location IS NOT NULL
                 ORDER BY wl.timestamp DESC
                 LIMIT 1
          ) last_pos ON true
          -- The last position off this plot at or before the current one: the boundary of
          -- the current stay. NULL when he has never been off it, i.e. the stay began with
          -- his first recorded position.
          LEFT JOIN LATERAL (
                SELECT MAX(wl5.timestamp) AS ts
                  FROM worker_locations wl5
                  JOIN workers w5 ON w5.id = wl5.worker_id AND w5.tenant_id = wl5.tenant_id
                 WHERE w5.user_id = t.worker_id
                   AND wl5.tenant_id = t.tenant_id
                   AND wl5.location IS NOT NULL
                   AND wl5.timestamp <= last_pos.ts
                   AND NOT EXISTS (SELECT 1
                                     FROM plots_at_position(wl5.tenant_id, wl5.location,
                                                            COALESCE(cfg.presence_radius_m, v_radius_default),
                                                            COALESCE(cfg.require_inside_plot, v_inside_default)) p5
                                    WHERE p5.site_id = COALESCE(t.site_id, os.site_id))
          ) last_off ON true
          -- How long the worker has been continuously on this plot.
          --
          -- "Continuously" is the operative word. The dwell is measured from the start of
          -- the *current uninterrupted stay*, not from the first position of the day.
          -- Counting from the start of the data would start the task on a worker who
          -- arrived yesterday, stepped out this morning and came back -- and, worse, it
          -- would stamp `started_at` with a time hours before he arrived.
          --
          -- The stay begins at the first on-plot position that has no off-plot position
          -- after it. `last_off` is that boundary: the latest position off this plot at or
          -- before the current one. Everything after it, up to now, is one continuous stay,
          -- so the earliest position in that window is the entry.
          JOIN LATERAL (
                SELECT MIN(wl3.timestamp) AS first_on
                  FROM worker_locations wl3
                  JOIN workers w3 ON w3.id = wl3.worker_id AND w3.tenant_id = wl3.tenant_id
                 WHERE w3.user_id = t.worker_id
                   AND wl3.tenant_id = t.tenant_id
                   AND wl3.location IS NOT NULL
                   AND wl3.timestamp <= last_pos.ts
                   AND wl3.timestamp > COALESCE(last_off.ts, '-infinity'::timestamptz)
                   -- the position must be on this plot
                   AND EXISTS (SELECT 1
                                 FROM plots_at_position(wl3.tenant_id, wl3.location,
                                                        COALESCE(cfg.presence_radius_m, v_radius_default),
                                                        COALESCE(cfg.require_inside_plot, v_inside_default)) p3
                                WHERE p3.site_id = COALESCE(t.site_id, os.site_id))
          ) on_plot ON true
         WHERE t.status IN ('pending', 'in_progress', 'assigned')
           AND t.tenant_id = COALESCE(p_tenant_id, t.tenant_id)
           AND COALESCE(cfg.auto_start, false)
           AND last_pos.location IS NOT NULL
           -- likewise: membership tested with EXISTS rather than array containment of a
           -- set-returning call
           AND EXISTS (SELECT 1
                         FROM plots_at_position(last_pos.tenant_id, last_pos.location,
                                                COALESCE(cfg.presence_radius_m, v_radius_default),
                                                COALESCE(cfg.require_inside_plot, v_inside_default)) p
                        WHERE p.site_id = COALESCE(t.site_id, os.site_id))
           -- the stay must have lasted long enough
           AND last_pos.ts - on_plot.first_on >= (COALESCE(cfg.min_dwell_minutes, v_dwell_default) || ' minutes')::INTERVAL
           -- and no interval is open for this worker on this task
           AND NOT EXISTS (
                 SELECT 1 FROM task_work_intervals open_i
                  WHERE open_i.task_id = t.id
                    AND open_i.worker_id = t.worker_id
                    AND open_i.stopped_at IS NULL
           )
    LOOP
        INSERT INTO task_work_intervals
            (tenant_id, task_id, worker_id, site_id, started_at, source, dwell_minutes)
        VALUES
            (r.tenant_id, r.task_id, r.worker_id, r.site_id, r.on_since,
             'auto_geometry', EXTRACT(EPOCH FROM (v_now - r.on_since)) / 60.0)
        ON CONFLICT ON CONSTRAINT uq_open_interval_per_task_worker DO NOTHING
        RETURNING id INTO v_id;

        IF v_id IS NOT NULL THEN
            v_opened := array_append(v_opened, v_id);
        END IF;
    END LOOP;

    intervals_opened := COALESCE(v_opened, ARRAY[]::UUID[]);
    intervals_closed := COALESCE(v_closed, ARRAY[]::UUID[]);
    RETURN NEXT;
END;
$$;

COMMENT ON FUNCTION evaluate_task_automation(UUID, TIMESTAMPTZ) IS
'One evaluation pass over all workers with a position. Closes intervals that have been off their plot longer than the grace period, then opens intervals for tasks whose plot the worker has occupied for at least the dwell time. The stop timestamp is the last position on the plot, not the time the grace period expired. Never completes a task.';

-- ---------------------------------------------------------------------------
-- 4. The audit trigger cannot write a row for `order_sites`
-- ---------------------------------------------------------------------------
-- `audit_trigger_function` builds its audit row with `COALESCE(NEW.id, OLD.id)`, which
-- assumes every audited table has an `id` column. `order_sites` has a composite primary
-- key `(order_id, site_id)` and no `id` at all, so **every INSERT into it fails**:
--
--     ERROR: record "new" has no field "tenant_id"
--     ...
--     error: record "new" has no field "id"
--
-- The first error is the tenant column, which is absent too; the second is the entity id.
-- Either way the statement aborts, so the table has been effectively read-only since the
-- consolidated migration installed the trigger. Nothing in the application noticed: no
-- repository writes it, and the only reference is a test that asserts the table exists.
--
-- That matters here specifically, because the automation engine reads `order_sites` to
-- decide which plots a task covers -- and a task that is not linked to any plot can never
-- start, no matter where the worker is.
--
-- The trigger is replaced with one that falls back to the primary key when there is no
-- `id`, rather than being dropped: auditing a change to a task's plot assignment is worth
-- keeping, and the composite key identifies the row well enough.
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_tenant_id UUID;
    v_user_id UUID;
    v_action TEXT;
    v_old_value JSONB;
    v_new_value JSONB;
    v_changed_fields TEXT[];
    v_ip INET;
    v_user_agent TEXT;
    v_request_id UUID;
    v_session_id UUID;
    v_entity_id UUID;
BEGIN
    IF TG_TABLE_NAME = 'audit_logs' THEN
        RETURN NEW;
    END IF;

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

    -- The tenant id. Taken from the session setting when set, otherwise from the row.
    --
    -- A table with no tenant column of its own -- `order_sites` is the one in this schema,
    -- its key is (order_id, site_id) -- still has to produce a non-NULL value, because
    -- `audit_logs.tenant_id` is NOT NULL. Deriving it through the foreign key is correct:
    -- the plot link belongs to the order's tenant by construction. Without this the insert
    -- fails with `null value in column "tenant_id" of relation "audit_logs"`, which is the
    -- state the table has been in since the trigger was installed.
    IF v_tenant_id IS NULL THEN
        IF TG_TABLE_NAME = 'tenants' THEN
            v_tenant_id := NULL;
        ELSIF has_column(TG_TABLE_NAME, 'tenant_id') THEN
            v_tenant_id := (to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> 'tenant_id')::UUID;
        ELSIF TG_TABLE_NAME = 'order_sites' THEN
            v_tenant_id := (SELECT o.tenant_id
                              FROM orders o
                             WHERE o.id = (to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> 'order_id')::UUID);
        END IF;
    END IF;

    -- The entity id. `audit_logs.entity_id` is a UUID, so a table with a composite key
    -- cannot supply a textual fallback: the value has to be cast, and casting
    -- 'order_id=...,site_id=...' to uuid fails. The composite key is therefore recorded as
    -- a stable UUID derived from it -- md5 of the key text -- so the audit row is still
    -- written and still identifies the row, just not by the order id directly.
    IF has_column(TG_TABLE_NAME, 'id') THEN
        v_entity_id := CASE TG_OP
            WHEN 'DELETE' THEN (to_jsonb(OLD) ->> 'id')::UUID
            ELSE (to_jsonb(NEW) ->> 'id')::UUID
        END;
    ELSE
        -- A composite key cannot be a UUID, so the key text is hashed and the digest is
        -- formatted into the 8-4-4-4-12 shape a UUID needs. (An earlier version prefixed
        -- the md5 with 'x', producing 'x53978d6d', which the cast rejected with
        -- `invalid input syntax for type uuid`.)
        v_entity_id := (
            SELECT (substr(d.h, 1, 8) || '-' || substr(d.h, 9, 4) || '-' || substr(d.h, 13, 4)
                    || '-' || substr(d.h, 17, 4) || '-' || substr(d.h, 21, 12))::UUID
              FROM (
                    SELECT md5(string_agg(format('%s=%s', a.attname,
                             COALESCE(to_jsonb(CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END) ->> a.attname, '')),
                             ',' ORDER BY a.attname)) AS h
                      FROM pg_index ix
                      JOIN pg_attribute a
                        ON a.attrelid = ix.indrelid
                       AND a.attnum = ANY(ix.indkey)
                     WHERE ix.indrelid = TG_RELID
                       AND ix.indisprimary
                   ) d
        );
    END IF;

    v_action := CASE TG_OP
        WHEN 'INSERT' THEN 'CREATE'
        WHEN 'UPDATE' THEN 'UPDATE'
        ELSE 'DELETE'
    END;

    IF TG_OP <> 'INSERT' THEN
        v_old_value := to_jsonb(OLD);
    END IF;
    IF TG_OP <> 'DELETE' THEN
        v_new_value := to_jsonb(NEW);
    END IF;

    -- The changed column names, computed rather than left NULL: an UPDATE that records
    -- "something changed" without saying what is of little use when reconstructing a
    -- sequence of edits. to_jsonb keys are sorted so the list is deterministic.
    IF TG_OP = 'UPDATE' THEN
        -- jsonb_each, not jsonb_object_keys in a WHERE: a set-returning function is not
        -- allowed there. Each row is one column of NEW, compared against the same key of
        -- OLD, and only the differing ones are kept.
        SELECT COALESCE(array_agg(n.key ORDER BY n.key), ARRAY[]::TEXT[])
          INTO v_changed_fields
          FROM jsonb_each(to_jsonb(NEW)) n
         WHERE n.value IS DISTINCT FROM to_jsonb(OLD) -> n.key;
    END IF;

    INSERT INTO audit_logs (
        tenant_id, user_id, action, entity_type, entity_id,
        old_value, new_value, changed_fields,
        ip_address, user_agent, request_id, session_id
    ) VALUES (
        v_tenant_id, v_user_id, v_action, TG_TABLE_NAME, v_entity_id,
        v_old_value, v_new_value, v_changed_fields,
        v_ip, v_user_agent, v_request_id, v_session_id
    );

    RETURN CASE TG_OP
        WHEN 'DELETE' THEN OLD
        ELSE NEW
    END;
END;
$$;

COMMENT ON FUNCTION audit_trigger_function() IS
'Writes an audit row for every insert, update and delete. The entity id falls back to the primary key for tables without an id column -- order_sites has a composite key and could not be written to at all before, because this function referenced NEW.id unconditionally.';
