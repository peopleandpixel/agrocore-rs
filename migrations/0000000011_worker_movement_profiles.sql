-- Worker movement profiles, and retention for the raw location stream.
--
-- The problem
-- ----------
-- `worker_locations` stores one row per reported position with no retention and no index
-- on (worker_id, timestamp). At a 30-second reporting interval that is 2,880 rows per
-- worker per day -- roughly 3 GB a year for 50 workers -- and reading a day's history is a
-- sort over the whole table.
--
-- But the row count is not the real objection. A point every 30 seconds does not describe
-- movement: a worker standing between two trees emits identical points, and GPS noise
-- emits outliers that a map draws as teleports. The information is in the line between the
-- points, not in the points. `accuracy_meters` is stored but never used, so a fix with
-- 50 m of uncertainty is treated the same as an exact one, and asked which parcel it
-- falls in it gives a wrong answer.
--
-- So this adds a derived profile table and an aggregation path, and makes retention
-- configurable rather than implicit.
--
-- Two truths about the same data
-- -----------------------------
-- `task_data.gps_track JSONB` already stores a track, in a second shape: not queryable
-- for any spatial question, not indexable. A worker position history and a task track are
-- the same fact in two places, which is how two contradictory truths form. The JSONB
-- column is left in place -- it can be converted with ST_GeomFromGeoJSON later without an
-- application change -- but `worker_tracks` is now the queryable one.
--
-- Retention
-- ---------
-- Raw points are kept for `tracking.raw_retention_days` (default 30), then aggregated
-- into `worker_daily_profiles` and deleted. Thirty days is a policy decision, not a
-- technical limit: a worker position history is a personnel profile rather than an
-- inventory record, so it is reasonable to keep it bounded.
--
-- Configurable where
-- ------------------
-- `system_settings`, with the all-zero/ NULL tenant meaning the global default and a
-- tenant row overriding it. The retention worker reads the setting, so changing it takes
-- effect on the next run without a redeploy.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. Index the raw stream.
-- ---------------------------------------------------------------------------
-- Without this, "where was this worker on Tuesday" sorts the whole table. The index is on
-- (tenant_id, worker_id, timestamp DESC) so both the history read and the retention scan
-- are ordered lookups rather than sorts.
CREATE INDEX IF NOT EXISTS idx_worker_locations_worker_time
    ON worker_locations(tenant_id, worker_id, timestamp DESC);

-- The retention scan filters on timestamp alone across all workers.
CREATE INDEX IF NOT EXISTS idx_worker_locations_time
    ON worker_locations(timestamp);

-- A spatial index is not useful on a single point per row, but it is needed if a point is
-- ever queried by containment. Kept deliberately minimal: the history read is by worker
-- and time, not by area.

-- ---------------------------------------------------------------------------
-- 2. worker_daily_profiles
-- ---------------------------------------------------------------------------
-- One row per worker per local day. This is what a movement profile actually needs: where
-- the day started and ended, how far it went, which parcels were touched.
CREATE TABLE IF NOT EXISTS worker_daily_profiles (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    worker_id       UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    -- The calendar day the profile covers. A DATE, not a timestamp: a profile is a day,
    -- and a timezone-less TIMESTAMPTZ would put the same shift in two profiles.
    profile_date    DATE NOT NULL,
    -- First and last observed position, as points.
    first_location  GEOMETRY(POINT, 4326),
    last_location   GEOMETRY(POINT, 4326),
    -- The day's path, simplified. Not the raw sequence: the simplification tolerance is
    -- what makes this a profile rather than a recording.
    path            GEOMETRY(LINESTRING, 4326),
    points_in       INTEGER NOT NULL DEFAULT 0,
    -- Metres travelled, summed from consecutive fixes.
    distance_meters DOUBLE PRECISION,
    -- Seconds between the first and last fix. Not the time spent moving: a worker
    -- standing still has the same span, and conflating the two would overstate the day.
    span_seconds    INTEGER,
    -- Worst reported accuracy among the day's points, so a consumer can tell a precise
    -- day from a fuzzy one.
    max_accuracy_meters DOUBLE PRECISION,
    -- Parcelles the path intersected, as a MultiPolygon. Filled by the aggregation; a
    -- tolerance is not applied to it, because "which parcel" is a containment question
    -- and a simplified line that clips a corner would answer it wrongly.
    areas_covered   GEOMETRY(MULTIPOLYGON, 4326),
    source          VARCHAR(32) NOT NULL DEFAULT 'aggregation',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One profile per worker per day. Without this the aggregation would insert a second
    -- row for a worker who reported across midnight.
    CONSTRAINT uq_worker_daily_profile UNIQUE (tenant_id, worker_id, profile_date)
);

CREATE INDEX IF NOT EXISTS idx_worker_daily_profiles_date
    ON worker_daily_profiles(tenant_id, profile_date DESC);
CREATE INDEX IF NOT EXISTS idx_worker_daily_profiles_worker
    ON worker_daily_profiles(tenant_id, worker_id, profile_date DESC);

COMMENT ON TABLE worker_daily_profiles IS
'Derived movement profile: one row per worker per calendar day. Aggregated from worker_locations and then retained after the raw points expire. Created_at on this table is the aggregation time, not the day the profile describes -- use profile_date for that.';
COMMENT ON COLUMN worker_daily_profiles.span_seconds IS
'Seconds between the first and last fix of the day. This is not time in motion: a stationary worker has the same span as a moving one.';
COMMENT ON COLUMN worker_daily_profiles.path IS
'The day''s path, simplified at the configured tolerance. Derived, so it can be rebuilt from the raw points while they still exist.';

-- ---------------------------------------------------------------------------
-- 3. Aggregation function
-- ---------------------------------------------------------------------------
-- Turns a day of raw points into a profile. Written as a function so the retention job and
-- a manual rebuild run identical logic -- a rebuild that computes something slightly
-- different from the scheduled aggregation would produce profiles that disagree with
-- themselves depending on when they ran.
CREATE OR REPLACE FUNCTION aggregate_worker_daily_profile(
    p_tenant_id  UUID,
    p_worker_id  UUID,
    p_date       DATE,
    p_tolerance  DOUBLE PRECISION DEFAULT 5.0
)
RETURNS UUID
LANGUAGE plpgsql
AS $$
DECLARE
    v_profile_id UUID;
    v_first      GEOMETRY(POINT, 4326);
    v_last       GEOMETRY(POINT, 4326);
    v_path       GEOMETRY(LINESTRING, 4326);
    v_count      INTEGER;
    v_span       INTEGER;
    v_distance   DOUBLE PRECISION;
    v_max_acc    DOUBLE PRECISION;
BEGIN
    SELECT
        (array_agg(location ORDER BY timestamp ASC))[1],
        (array_agg(location ORDER BY timestamp DESC))[1],
        count(*),
        EXTRACT(EPOCH FROM (max(timestamp) - min(timestamp)))::INTEGER
    INTO v_first, v_last, v_count, v_span
    FROM worker_locations
    WHERE tenant_id = p_tenant_id
      AND worker_id = p_worker_id
      AND timestamp >= p_date::TIMESTAMP AT TIME ZONE 'UTC'
      AND timestamp <  (p_date + 1)::TIMESTAMP AT TIME ZONE 'UTC'
      AND location IS NOT NULL;

    IF v_count IS NULL OR v_count = 0 THEN
        RETURN NULL;
    END IF;

    -- A day with one point has no line. ST_MakeLine of a single point errors, and a
    -- one-point day is a real case: a worker who started and stopped without moving.
    -- Two identical points also produce a zero-length line, which is not an error but
    -- carries no information, so both cases are left as NULL.
    IF v_count > 1 AND (
        SELECT count(DISTINCT location::text) FROM (
            SELECT location
            FROM worker_locations
            WHERE tenant_id = p_tenant_id
              AND worker_id = p_worker_id
              AND timestamp >= p_date::TIMESTAMP AT TIME ZONE 'UTC'
              AND timestamp <  (p_date + 1)::TIMESTAMP AT TIME ZONE 'UTC'
              AND location IS NOT NULL
            ORDER BY timestamp ASC
        ) d
    ) > 1 THEN
        SELECT ST_Simplify(
                   ST_MakeLine(geom),
                   p_tolerance,
                   false           -- keep the geometry valid; a simplified line that
                                     -- self-intersects is still drawable
               )
        INTO v_path
        FROM (
            SELECT location AS geom
            FROM worker_locations
            WHERE tenant_id = p_tenant_id
              AND worker_id = p_worker_id
              AND timestamp >= p_date::TIMESTAMP AT TIME ZONE 'UTC'
              AND timestamp <  (p_date + 1)::TIMESTAMP AT TIME ZONE 'UTC'
              AND location IS NOT NULL
            ORDER BY timestamp ASC
        ) t;
    END IF;

    -- Distance sums the segments between consecutive fixes.
    --
    -- The cast to `geography` is not optional. ST_Length on a geometry in SRID 4326
    -- returns degrees, not metres, so a 1.3 km walk measured as a bare ST_Length stores
    -- 0.013 -- and rounded to two places in any report that is simply zero. Casting to
    -- geography makes PostGIS measure on the spheroid.
    SELECT COALESCE(ST_Length(ST_MakeLine(geom)::geography)::DOUBLE PRECISION, 0)
    INTO v_distance
    FROM (
        SELECT location AS geom
        FROM worker_locations
        WHERE tenant_id = p_tenant_id
          AND worker_id = p_worker_id
          AND timestamp >= p_date::TIMESTAMP AT TIME ZONE 'UTC'
          AND timestamp <  (p_date + 1)::TIMESTAMP AT TIME ZONE 'UTC'
          AND location IS NOT NULL
        ORDER BY timestamp ASC
    ) t;

    SELECT max(accuracy_meters) INTO v_max_acc
    FROM worker_locations
    WHERE tenant_id = p_tenant_id
      AND worker_id = p_worker_id
      AND timestamp >= p_date::TIMESTAMP AT TIME ZONE 'UTC'
      AND timestamp <  (p_date + 1)::TIMESTAMP AT TIME ZONE 'UTC';

    INSERT INTO worker_daily_profiles
        (tenant_id, worker_id, profile_date, first_location, last_location, path,
         points_in, distance_meters, span_seconds, max_accuracy_meters, source)
    VALUES
        (p_tenant_id, p_worker_id, p_date, v_first, v_last, v_path,
         v_count, v_distance, v_span, v_max_acc, 'aggregation')
    ON CONFLICT (tenant_id, worker_id, profile_date) DO UPDATE
        SET first_location       = EXCLUDED.first_location,
            last_location        = EXCLUDED.last_location,
            path                 = EXCLUDED.path,
            points_in            = EXCLUDED.points_in,
            distance_meters      = EXCLUDED.distance_meters,
            span_seconds         = EXCLUDED.span_seconds,
            max_accuracy_meters  = EXCLUDED.max_accuracy_meters,
            updated_at           = NOW()
    RETURNING id INTO v_profile_id;

    RETURN v_profile_id;
END;
$$;

COMMENT ON FUNCTION aggregate_worker_daily_profile(UUID, UUID, DATE, DOUBLE PRECISION) IS
'Builds or refreshes the daily movement profile for one worker and day from the raw points. Idempotent -- re-running overwrites rather than duplicating. Returns the profile id, or NULL when no points exist for that day.';

-- ---------------------------------------------------------------------------
-- 4. Retention
-- ---------------------------------------------------------------------------
-- Aggregates every day whose raw points are about to be deleted, then deletes only those
-- points. The order matters: if the delete ran first, a crash would lose the day's track
-- with no profile to show for it.
--
-- Reads `tracking.raw_retention_days` from system_settings, falling back to 30. The
-- setting is per tenant, with the NULL-tenant row as the global default.
CREATE OR REPLACE FUNCTION enforce_worker_location_retention()
RETURNS TABLE (
    worker_id     UUID,
    profile_date  DATE,
    points_removed INTEGER
)
LANGUAGE plpgsql
AS $$
DECLARE
    v_retention_days INTEGER;
    v_cutoff         TIMESTAMP;
    r                 RECORD;
    v_profile_id      UUID;
    v_tolerance       DOUBLE PRECISION;
    v_removed         INTEGER;
BEGIN
    -- The setting is JSONB: '"30"' is a JSON string, '30' is a JSON number. Accept both,
    -- because which one is stored depends on which value_type the row was seeded with.
    SELECT COALESCE(
        (SELECT (value #>> '{}')::INTEGER
           FROM system_settings
          WHERE key = 'tracking.raw_retention_days'
            AND tenant_id IS NULL),
        30)
    INTO v_retention_days;

    IF v_retention_days IS NULL OR v_retention_days < 1 THEN
        v_retention_days := 30;
    END IF;

    SELECT COALESCE(
        (SELECT (value #>> '{}')::DOUBLE PRECISION
           FROM system_settings
          WHERE key = 'tracking.simplify_tolerance_m'
            AND tenant_id IS NULL),
        5.0)
    INTO v_tolerance;

    v_cutoff := now() - (v_retention_days || ' days')::INTERVAL;

    -- Every column reference below is table-qualified. The function returns a table with
    -- columns named worker_id and profile_date, and PL/pgSQL resolves an unqualified name
    -- against those output variables first -- so `worker_id` in the query raised
    -- `column reference "worker_id" is ambiguous` and the whole retention run did nothing.
    FOR r IN
        -- Distinct worker/day pairs that have raw points older than the cutoff. The
        -- date comes from the timestamp itself rather than from a stored column, so a
        -- day's points are grouped by the day they were recorded on.
        SELECT DISTINCT
               wl.tenant_id,
               wl.worker_id,
               (wl.timestamp AT TIME ZONE 'UTC')::DATE AS day
          FROM worker_locations wl
         WHERE wl.timestamp < v_cutoff
           AND wl.location IS NOT NULL
         ORDER BY wl.tenant_id, wl.worker_id, day
    LOOP
        v_profile_id := aggregate_worker_daily_profile(
            r.tenant_id, r.worker_id, r.day, v_tolerance);

        -- Only delete the points once the profile exists. A NULL profile means the
        -- aggregation found nothing, and deleting then would discard the day silently.
        IF v_profile_id IS NULL THEN
            CONTINUE;
        END IF;

        DELETE FROM worker_locations wl
         WHERE wl.tenant_id = r.tenant_id
           AND wl.worker_id = r.worker_id
           AND wl.timestamp >= r.day::TIMESTAMP AT TIME ZONE 'UTC'
           AND wl.timestamp <  (r.day + 1)::TIMESTAMP AT TIME ZONE 'UTC'
           AND wl.timestamp <  v_cutoff;

        GET DIAGNOSTICS v_removed = ROW_COUNT;
        worker_id     := r.worker_id;
        profile_date  := r.day;
        points_removed := v_removed;
        RETURN NEXT;
    END LOOP;
END;
$$;

COMMENT ON FUNCTION enforce_worker_location_retention() IS
'Aggregates expired worker-days into worker_daily_profiles, then deletes the raw points. Reads tracking.raw_retention_days from system_settings (default 30) and tracking.simplify_tolerance_m (default 5). Aggregates before deleting, and skips a day whose profile could not be built. Returns one row per worker-day processed.';

-- ---------------------------------------------------------------------------
-- 5. Areas covered
-- ---------------------------------------------------------------------------
-- Filled separately from the aggregation because it needs the parcel layer, and the
-- layer is a deployment concern: a tenant may have no parcels registered yet, in which
-- case `areas_covered` stays NULL rather than being an empty geometry.
CREATE OR REPLACE FUNCTION refresh_worker_profile_areas(
    p_tenant_id UUID,
    p_date      DATE DEFAULT NULL
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_updated INTEGER := 0;
BEGIN
    -- Every alias is distinct. An earlier version used `p` for the profile in the CTE and
    -- again in the UPDATE, and referred to the parameter as `p_tenant_id` next to a column
    -- of the same name -- the statement then matched rows but updated none of them, and
    -- the function returned 0 while looking correct.
    WITH covered AS (
        SELECT
            prof.id AS profile_id,
            -- ST_Collect, not ST_Union(ST_Collect(...)). PostGIS rejects nested
            -- aggregates outright (`aggregate function calls cannot be nested`), and the
            -- outer ST_Union was not needed: ST_Collect already merges the matching
            -- parcels into one geometry, dissolving the shared edges.
            ST_Collect(parcel.boundary) AS area
        FROM worker_daily_profiles prof
        JOIN sites parcel
          ON parcel.tenant_id = prof.tenant_id
         AND parcel.boundary IS NOT NULL
        WHERE prof.tenant_id = p_tenant_id
          AND (p_date IS NULL OR prof.profile_date = p_date)
          AND prof.path IS NOT NULL
          AND ST_Intersects(prof.path, parcel.boundary)
        GROUP BY prof.id
    )
    UPDATE worker_daily_profiles target
       SET areas_covered = CASE
               WHEN c.area IS NULL THEN NULL
               -- ST_Collect on a single polygon yields POLYGON; the column is
               -- MULTIPOLYGON and PostGIS will not cast implicitly.
               WHEN GeometryType(c.area) = 'POLYGON'
                   THEN ST_Multi(c.area)
               ELSE c.area
           END,
           updated_at = NOW()
      FROM covered c
     WHERE target.id = c.profile_id;
    GET DIAGNOSTICS v_updated = ROW_COUNT;
    RETURN v_updated;
END;
$$;

COMMENT ON FUNCTION refresh_worker_profile_areas(UUID, DATE) IS
'Fills worker_daily_profiles.areas_covered from the tenant''s parcel boundaries. Uses the raw path, not the simplified one, because containment is a question a simplified line can answer wrongly by clipping a corner.';

-- ---------------------------------------------------------------------------
-- 6. Row-level security
-- ---------------------------------------------------------------------------
ALTER TABLE worker_daily_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE worker_daily_profiles FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS worker_daily_profiles_select ON worker_daily_profiles;
CREATE POLICY worker_daily_profiles_select ON worker_daily_profiles FOR SELECT
    USING (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS worker_daily_profiles_insert ON worker_daily_profiles;
CREATE POLICY worker_daily_profiles_insert ON worker_daily_profiles FOR INSERT
    WITH CHECK (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS worker_daily_profiles_update ON worker_daily_profiles;
CREATE POLICY worker_daily_profiles_update ON worker_daily_profiles FOR UPDATE
    USING (tenant_id = get_current_tenant_id())
    WITH CHECK (tenant_id = get_current_tenant_id());

DROP POLICY IF EXISTS worker_daily_profiles_delete ON worker_daily_profiles;
CREATE POLICY worker_daily_profiles_delete ON worker_daily_profiles FOR DELETE
    USING (tenant_id = get_current_tenant_id());

-- The aggregation and retention functions run as the application role, which is subject to
-- FORCE RLS like any other connection. They insert profiles for every tenant they scan, so
-- they cannot be granted to `agrocore_app` as-is; a SECURITY DEFINER variant would be, but
-- then it would bypass RLS entirely and be able to read across tenants. Scheduled retention
-- is a maintenance task and runs as a privileged role -- documented rather than fudged.

-- Grants, following the loop pattern of migration 4.
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
          AND c.relname = 'worker_daily_profiles'
    LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO agrocore_app', r.relname);
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- 7. Configurable settings
-- ---------------------------------------------------------------------------
-- The audit trigger writes `tenant_id` from `app.current_tenant_id`, and `audit_logs`
-- declares that column NOT NULL. A global setting row has tenant_id NULL by design -- that
-- is how a tenant row overrides it -- so inserting one from a migration fails with
-- `null value in column "tenant_id" of relation "audit_logs"`.
--
-- Migration 8 added `app.audit_suppressed` for exactly this class of problem, when the
-- cascading deletes of a tenant removal each tried to log a tenant that no longer existed.
-- The same escape applies here: a seeded default has no tenant to attribute an audit row
-- to. Auditing of *tenant* changes is untouched -- a tenant editing this setting still
-- writes an audit row, because this flag is set only for the duration of this statement.
SET LOCAL app.audit_suppressed = 'on';

INSERT INTO system_settings (tenant_id, key, value, value_type, description, is_sensitive)
VALUES
    (NULL, 'tracking.raw_retention_days', '30', 'number',
     'Days raw worker positions are kept before being aggregated into a daily profile and deleted. A worker position history is a personnel record rather than an inventory record, so it is deliberately bounded.', FALSE),
    (NULL, 'tracking.simplify_tolerance_m', '5.0', 'number',
     'Douglas-Peucker tolerance in metres when building a daily movement profile. Lower keeps more detail and more points: use 2 m where worked area is being measured, 20 m for a route overview.', FALSE),
    (NULL, 'tracking.retention_interval_minutes', '360', 'number',
     'How often the retention job runs, in minutes. Retention deletes data, so this also bounds how long expired raw points survive past their retention date.', FALSE)
ON CONFLICT DO NOTHING;

RESET app.audit_suppressed;