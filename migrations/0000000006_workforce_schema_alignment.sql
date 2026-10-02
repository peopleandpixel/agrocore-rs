-- Align the `workers` and `worker_task_statuses` tables with their entities.
--
-- Both repositories read `SELECT *` and map the row onto a Rust struct, so any
-- column the struct declares but the table lacks, or vice versa, breaks the
-- whole endpoint. Two such drifts are live right now:
--
--   1. `workers` was created as an HR table (employee_id, firstname, lastname,
--      email, phone, role_in_company, social_security_number, bank_account).
--      `crates/domain/src/entities/workforce.rs` models a contract worker:
--      contract_type, language, skills, certifications, emergency_contact,
--      nationality. The repository inserts exactly those, so
--      `worker_repo().create()` failed with
--      `column "contract_type" of relation "workers" does not exist`.
--
--   2. `worker_task_statuses` has status/started_at/completed_at, but
--      `WorkerTaskStatus` declares paused_at, resumed_at, stopped_at, done_at
--      and updated_at. The INSERT names `updated_at`, so it failed with
--      `column "updated_at" of relation "worker_task_statuses" does not exist`.
--
-- Note on scope: the columns the repository never writes are added rather than
-- the entity changed, because `Worker` is serialised in API responses and
-- narrowing it would drop fields callers already read. The legacy HR columns
-- stay: they are NOT NULL in places, and a data-preserving migration is better
-- than one that deletes payroll data. Their values are backfilled from `users`
-- where a worker row has a user_id, so a row created before this migration is
-- not left with placeholders.

-- ---------------------------------------------------------------------------
-- 1. workers: the columns Worker declares.
-- ---------------------------------------------------------------------------
ALTER TABLE workers ADD COLUMN IF NOT EXISTS contract_type JSONB NOT NULL DEFAULT '"Permanent"'::jsonb;
ALTER TABLE workers ADD COLUMN IF NOT EXISTS language TEXT;
ALTER TABLE workers ADD COLUMN IF NOT EXISTS skills JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE workers ADD COLUMN IF NOT EXISTS certifications JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE workers ADD COLUMN IF NOT EXISTS emergency_contact TEXT;
ALTER TABLE workers ADD COLUMN IF NOT EXISTS nationality TEXT;

-- The legacy HR columns are NOT NULL with no default, so a row inserted by the
-- repaired repository would fail without them. Give them defaults rather than
-- dropping NOT NULL: the columns still exist and keep whatever they held.
ALTER TABLE workers ALTER COLUMN employee_id SET DEFAULT 'UNKNOWN';
ALTER TABLE workers ALTER COLUMN firstname SET DEFAULT '';
ALTER TABLE workers ALTER COLUMN lastname SET DEFAULT '';

-- Backfill the names for workers that already exist, from their user account.
UPDATE workers w
SET firstname = COALESCE(NULLIF(u.firstname, ''), w.firstname),
    lastname  = COALESCE(NULLIF(u.lastname,  ''), w.lastname),
    email     = COALESCE(w.email, u.email)
FROM users u
WHERE u.id = w.user_id
  AND (w.firstname = '' OR w.lastname = '' OR w.email IS NULL);

-- `Worker` reads `hourly_rate` as f64 while the column is NUMERIC(10,2).
-- NUMERIC decodes into f64 in sqlx only via a cast, and the entity is the
-- contract; widen to DOUBLE PRECISION so `SELECT *` maps cleanly.
ALTER TABLE workers ALTER COLUMN hourly_rate TYPE DOUBLE PRECISION USING hourly_rate::double precision;

CREATE INDEX IF NOT EXISTS idx_workers_contract_type ON workers (contract_type);

-- ---------------------------------------------------------------------------
-- 2. worker_task_statuses: the timestamps WorkerTaskStatus declares.
-- ---------------------------------------------------------------------------
ALTER TABLE worker_task_statuses ADD COLUMN IF NOT EXISTS paused_at TIMESTAMPTZ;
ALTER TABLE worker_task_statuses ADD COLUMN IF NOT EXISTS resumed_at TIMESTAMPTZ;
ALTER TABLE worker_task_statuses ADD COLUMN IF NOT EXISTS stopped_at TIMESTAMPTZ;
ALTER TABLE worker_task_statuses ADD COLUMN IF NOT EXISTS done_at TIMESTAMPTZ;
ALTER TABLE worker_task_statuses ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- `status` is declared `#[sqlx(json)]` on the entity, so the column has to be
-- JSONB holding the serialised enum ("\"in_progress\""), not TEXT holding the
-- bare word. sqlx refuses the decode outright otherwise:
--   mismatched types; Rust type `Json<WorkerTaskStatusType>` (as SQL type JSONB)
--   is not compatible with SQL type TEXT
-- Existing rows are converted in the same ALTER so no row is left undecodable.
-- The column carries DEFAULT 'pending', and PostgreSQL cannot cast a TEXT
-- default to JSONB implicitly, so the default is dropped and re-added in the
-- new type within the same transaction.
ALTER TABLE worker_task_statuses ALTER COLUMN status DROP DEFAULT;
ALTER TABLE worker_task_statuses
    ALTER COLUMN status TYPE JSONB USING to_jsonb(status);
ALTER TABLE worker_task_statuses
    ALTER COLUMN status SET DEFAULT '"pending"'::jsonb;

-- A plain 'pending' becomes '"pending"', but a value that is already quoted is
-- left alone: the cast above would turn '"started"' into '"\"started\"' and the
-- decode would then fail on the extra escaping.
UPDATE worker_task_statuses
SET status = to_jsonb(trim(both '"' from status::text))::jsonb
WHERE status::text LIKE '"%"';

-- A status row is unique per (task, worker); the repository reads it back with
-- that pair, so a duplicate would make the read non-deterministic.
CREATE UNIQUE INDEX IF NOT EXISTS worker_task_statuses_task_worker_idx
    ON worker_task_statuses (task_id, worker_id);

-- ---------------------------------------------------------------------------
-- 3. worker_locations: worker_id references workers(id), but the repository is
--    handed a *user* id by the workforce module. The FK was rejecting valid
--    rows with "violates foreign key constraint worker_locations_worker_id_fkey".
--    A user who is not a worker has no workers row, so the FK has to go; the
--    tenant column already scopes the row and RLS already enforces it.
-- ---------------------------------------------------------------------------
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'worker_locations_worker_id_fkey'
          AND conrelid = 'worker_locations'::regclass
    ) THEN
        ALTER TABLE worker_locations DROP CONSTRAINT worker_locations_worker_id_fkey;
    END IF;
END
$$;

-- `location` is nullable in the table but every repository write supplies it.
-- NOT NULL turns a bug that currently produces a row with no position — which
-- `ST_X(location)` then reads as NULL and the GPS view shows as the null island
-- — into an error at the point of the write.
ALTER TABLE worker_locations ALTER COLUMN location SET NOT NULL;

COMMENT ON COLUMN workers.contract_type IS
    'Serialised ContractType enum (Permanent, Temporary, Seasonal, Freelance, Apprenticeship, Custom).';
COMMENT ON COLUMN workers.certifications IS
    'JSONB array of Certification objects.';
COMMENT ON TABLE worker_task_statuses IS
    'Per task/worker status with the full pause/resume/stop/done timeline.';
