-- Migration 0016: Materialized Views for Automatic Groupings and Operational Views
-- Provides pre-computed groupings and operational dashboards

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- 1. Automatic Groupings (Materialized Views with pg_cron refresh)
-- ---------------------------------------------------------------------------

-- All olive trees across all sites/tenants
CREATE MATERIALIZED VIEW IF NOT EXISTS all_olive_trees AS
SELECT
    so.id,
    so.tenant_id,
    so.site_id,
    s.label AS site_label,
    so.parent_id,
    so.label,
    so.object_type,
    so.geometry,
    so.area,
    so.buffer_meters,
    so.planted_at,
    so.variety_id,
    v.name AS variety_label,
    v.species_key AS species,
    so.is_active,
    so.is_temporary,
    so.note,
    so.created_at,
    so.updated_at
FROM spatial_objects so
LEFT JOIN sites s ON s.id = so.site_id AND s.tenant_id = so.tenant_id
LEFT JOIN varieties v ON v.id = so.variety_id
WHERE so.object_type = 'olive_tree'
  AND so.is_active;

CREATE UNIQUE INDEX IF NOT EXISTS idx_all_olive_trees_id ON all_olive_trees (id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_tenant ON all_olive_trees (tenant_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_site ON all_olive_trees (site_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_variety ON all_olive_trees (variety_id);
CREATE INDEX IF NOT EXISTS idx_all_olive_trees_geometry ON all_olive_trees USING GIST (geometry);

COMMENT ON MATERIALIZED VIEW all_olive_trees IS
'All olive trees across all sites. Refreshed via pg_cron every 15 minutes.';

-- All parcels/sites that have olive trees
CREATE MATERIALIZED VIEW IF NOT EXISTS parcels_with_olives AS
SELECT DISTINCT ON (s.id)
    s.id AS site_id,
    s.tenant_id,
    s.label AS site_label,
    s.boundary,
    s.area,
    COUNT(so.id) AS olive_tree_count,
    MIN(so.planted_at) AS oldest_planting,
    MAX(so.planted_at) AS newest_planting,
    AVG(EXTRACT(YEAR FROM AGE(NOW(), so.planted_at)))::numeric(4,1) AS avg_tree_age_years
FROM sites s
JOIN spatial_objects so ON so.site_id = s.id AND so.tenant_id = s.tenant_id
WHERE so.object_type = 'olive_tree'
  AND so.is_active
  AND s.is_active
GROUP BY s.id, s.tenant_id, s.label, s.boundary, s.area;

CREATE UNIQUE INDEX IF NOT EXISTS idx_parcels_with_olives_id ON parcels_with_olives (site_id);
CREATE INDEX IF NOT EXISTS idx_parcels_with_olives_tenant ON parcels_with_olives (tenant_id);
CREATE INDEX IF NOT EXISTS idx_parcels_with_olives_boundary ON parcels_with_olives USING GIST (boundary);

COMMENT ON MATERIALIZED VIEW parcels_with_olives IS
'Parcels/sites that contain olive trees, with aggregate stats. Refreshed via pg_cron.';

-- Olive trees grouped by variety
CREATE MATERIALIZED VIEW IF NOT EXISTS olive_trees_by_variety AS
SELECT
    v.id AS variety_id,
    v.tenant_id,
    v.name AS variety_label,
    v.species_key AS species,
    COUNT(so.id) AS tree_count,
    SUM(so.area) AS total_area_m2,
    MIN(so.planted_at) AS oldest_planting,
    MAX(so.planted_at) AS newest_planting,
    COUNT(DISTINCT so.site_id) AS site_count
FROM spatial_objects so
JOIN varieties v ON v.id = so.variety_id
WHERE so.object_type = 'olive_tree'
  AND so.is_active
  AND v.active
GROUP BY v.id, v.tenant_id, v.name, v.species_key;

CREATE UNIQUE INDEX IF NOT EXISTS idx_olive_trees_by_variety_id ON olive_trees_by_variety (variety_id);
CREATE INDEX IF NOT EXISTS idx_olive_trees_by_variety_tenant ON olive_trees_by_variety (tenant_id);

COMMENT ON MATERIALIZED VIEW olive_trees_by_variety IS
'Olive trees grouped by variety with aggregate stats. Refreshed via pg_cron.';

-- ---------------------------------------------------------------------------
-- 2. Operational Views (Regular Views - always current)
-- ---------------------------------------------------------------------------

-- Current worker task overview: which worker is on which task right now
CREATE OR REPLACE VIEW worker_task_overview AS
SELECT
    w.id AS worker_id,
    w.tenant_id,
    u.email AS worker_email,
    u.firstname || ' ' || u.lastname AS worker_name,
    latest_loc.location AS current_location,
    latest_loc.timestamp AS last_position_update,
    t.id AS task_id,
    t.title AS task_label,
    t.status AS task_status,
    o.id AS order_id,
    o.label AS order_label,
    o.status AS order_status,
    twi.started_at AS interval_started_at,
    EXTRACT(EPOCH FROM (NOW() - twi.started_at)) / 60.0 AS current_interval_minutes
FROM workers w
JOIN users u ON u.id = w.user_id AND u.tenant_id = w.tenant_id
LEFT JOIN LATERAL (
    SELECT wl.location, wl.timestamp
    FROM worker_locations wl
    WHERE wl.worker_id = w.id AND wl.tenant_id = w.tenant_id
    ORDER BY wl.timestamp DESC LIMIT 1
) latest_loc ON true
LEFT JOIN task_work_intervals twi ON twi.worker_id = u.id AND twi.tenant_id = w.tenant_id AND twi.stopped_at IS NULL
LEFT JOIN tasks t ON t.id = twi.task_id AND t.tenant_id = w.tenant_id
LEFT JOIN orders o ON o.id = t.order_id AND o.tenant_id = w.tenant_id
WHERE w.is_active;

COMMENT ON VIEW worker_task_overview IS
'Current worker-task assignments with live positions and interval timing. Always current.';

-- Task progress summary with subtask details
CREATE OR REPLACE VIEW task_progress_summary AS
SELECT
    t.id AS task_id,
    t.tenant_id,
    t.title AS task_label,
    t.description,
    t.status AS task_status,
    o.id AS order_id,
    o.label AS order_label,
    t.worker_id,
    u.email AS worker_email,
    u.firstname || ' ' || u.lastname AS worker_name,
    tp.overall_status,
    tp.progress_percent,
    tp.is_overdue,
    tp.sub_task_count,
    tp.sub_tasks_done,
    tp.planned_total,
    tp.completed_total,
    tp.worked_minutes,
    tp.sub_task_count - tp.sub_tasks_done AS remaining_sub_tasks
FROM tasks t
JOIN orders o ON o.id = t.order_id AND o.tenant_id = t.tenant_id
LEFT JOIN users u ON u.id = t.worker_id AND u.tenant_id = t.tenant_id
LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.tenant_id = t.tenant_id;

COMMENT ON VIEW task_progress_summary IS
'Task progress with subtask breakdown and overdue flag. Always current.';

-- Upcoming due tasks (within 7 days) and overdue tasks
CREATE OR REPLACE VIEW upcoming_due_tasks AS
SELECT
    t.id AS task_id,
    t.tenant_id,
    t.title AS task_label,
    t.status,
    o.id AS order_id,
    o.label AS order_label,
    o.deadline_date,
    CASE
        WHEN o.deadline_date IS NOT NULL AND o.deadline_date < CURRENT_DATE THEN 'overdue'
        WHEN o.deadline_date IS NOT NULL AND o.deadline_date <= CURRENT_DATE + 7 THEN 'due_soon'
        ELSE 'upcoming'
    END AS due_category,
    CURRENT_DATE - o.deadline_date AS days_overdue,
    tp.overall_status,
    tp.progress_percent,
    tp.sub_task_count,
    tp.sub_tasks_done
FROM tasks t
JOIN orders o ON o.id = t.order_id AND o.tenant_id = t.tenant_id
LEFT JOIN task_progress tp ON tp.task_id = t.id AND tp.tenant_id = t.tenant_id
WHERE o.deadline_date IS NOT NULL
  AND o.status NOT IN ('completed', 'cancelled')
  AND (o.deadline_date < CURRENT_DATE + 30)  -- look ahead 30 days
ORDER BY
    CASE WHEN o.deadline_date < CURRENT_DATE THEN 0 ELSE 1 END,
    o.deadline_date;

COMMENT ON VIEW upcoming_due_tasks IS
'Tasks due within 30 days or overdue, categorized. Always current.';

-- Worker availability: who is clocked in and available
CREATE OR REPLACE VIEW worker_availability AS
SELECT
    w.id AS worker_id,
    w.tenant_id,
    u.email,
    u.firstname || ' ' || u.lastname AS worker_name,
    ce.id AS active_clock_entry_id,
    ce.entry_type AS last_entry_type,
    ce.timestamp AS last_entry_time,
    CASE
        WHEN ce.entry_type = 'check_in' AND ce.timestamp > NOW() - INTERVAL '12 hours' THEN 'clocked_in'
        WHEN ce.entry_type = 'check_out' THEN 'clocked_out'
        ELSE 'unknown'
    END AS availability_status,
    wl.location AS last_known_location,
    wl.timestamp AS last_location_update
FROM workers w
JOIN users u ON u.id = w.user_id AND u.tenant_id = w.tenant_id
LEFT JOIN LATERAL (
    SELECT * FROM clock_entries ce
    WHERE ce.worker_id = w.id AND ce.tenant_id = w.tenant_id
    ORDER BY ce.timestamp DESC LIMIT 1
) ce ON true
LEFT JOIN LATERAL (
    SELECT * FROM worker_locations wl
    WHERE wl.worker_id = w.id AND wl.tenant_id = w.tenant_id
    ORDER BY wl.timestamp DESC LIMIT 1
) wl ON true
WHERE w.is_active
ORDER BY availability_status DESC, u.lastname;

COMMENT ON VIEW worker_availability IS
'Worker clock-in/out status with last known location. Always current.';

-- Site work summary: total work per site
CREATE OR REPLACE VIEW site_work_summary AS
SELECT
    s.id AS site_id,
    s.tenant_id,
    s.label AS site_label,
    s.boundary,
    s.area AS site_area_m2,
    COUNT(DISTINCT t.id) AS task_count,
    COUNT(DISTINCT t.id) FILTER (WHERE t.status = 'completed') AS completed_task_count,
    COUNT(DISTINCT t.id) FILTER (WHERE t.status = 'in_progress') AS in_progress_task_count,
    COALESCE(SUM(twi.duration_minutes), 0) AS total_work_minutes,
    COALESCE(AVG(twi.duration_minutes), 0) AS avg_task_minutes,
    COUNT(DISTINCT twi.worker_id) AS worker_count,
    MIN(twi.started_at) AS first_work_date,
    MAX(twi.stopped_at) AS last_work_date
FROM sites s
LEFT JOIN tasks t ON t.site_id = s.id AND t.tenant_id = s.tenant_id
LEFT JOIN task_work_intervals twi ON twi.site_id = s.id AND twi.tenant_id = s.tenant_id
WHERE s.is_active
GROUP BY s.id, s.tenant_id, s.label, s.boundary, s.area;

COMMENT ON VIEW site_work_summary IS
'Aggregated work statistics per site. Always current.';

-- Equipment utilization
CREATE OR REPLACE VIEW equipment_utilization AS
SELECT
    e.id AS equipment_id,
    e.tenant_id,
    e.label AS equipment_label,
    e.equipment_type,
    COUNT(twi.id) AS interval_count,
    COALESCE(SUM(twi.duration_minutes), 0) AS total_work_minutes,
    COUNT(DISTINCT twi.task_id) AS task_count,
    COUNT(DISTINCT twi.worker_id) AS operator_count,
    MIN(twi.started_at) AS first_use,
    MAX(twi.stopped_at) AS last_use,
    AVG(twi.duration_minutes)::numeric(10,2) AS avg_interval_minutes
FROM equipments e
LEFT JOIN task_work_intervals twi ON twi.machine_id = e.id AND twi.tenant_id = e.tenant_id
WHERE e.is_active
GROUP BY e.id, e.tenant_id, e.label, e.equipment_type;

COMMENT ON VIEW equipment_utilization IS
'Equipment usage statistics from work intervals. Always current.';

-- Grant access
GRANT SELECT ON all_olive_trees TO agrocore_app;
GRANT SELECT ON parcels_with_olives TO agrocore_app;
GRANT SELECT ON olive_trees_by_variety TO agrocore_app;
GRANT SELECT ON worker_task_overview TO agrocore_app;
GRANT SELECT ON task_progress_summary TO agrocore_app;
GRANT SELECT ON upcoming_due_tasks TO agrocore_app;
GRANT SELECT ON worker_availability TO agrocore_app;
GRANT SELECT ON site_work_summary TO agrocore_app;
GRANT SELECT ON equipment_utilization TO agrocore_app;