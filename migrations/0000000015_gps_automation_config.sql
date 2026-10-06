-- Migration 0015: Extend order_auto_automation_settings for GPS-triggered auto-start/end
-- Adds per-order configuration for automatic task start/stop based on worker position

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- Extend order_auto_automation_settings
-- ---------------------------------------------------------------------------
-- The existing columns (auto_start, auto_stop, min_dwell_minutes, grace_minutes,
-- presence_radius_m, require_inside_plot) control the time-based automation engine.
-- These new columns add GPS-triggered automation with separate radii and
-- second-precision dwell/absence thresholds.

ALTER TABLE order_auto_automation_settings
    ADD COLUMN IF NOT EXISTS auto_start_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS auto_start_trigger_radius_meters DOUBLE PRECISION DEFAULT 50,
    ADD COLUMN IF NOT EXISTS auto_start_min_dwell_seconds INTEGER DEFAULT 300,
    ADD COLUMN IF NOT EXISTS auto_end_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS auto_end_trigger_radius_meters DOUBLE PRECISION DEFAULT 50,
    ADD COLUMN IF NOT EXISTS auto_end_min_absence_seconds INTEGER DEFAULT 600,
    ADD COLUMN IF NOT EXISTS auto_end_grace_seconds INTEGER DEFAULT 120;

COMMENT ON COLUMN order_auto_automation_settings.auto_start_enabled
IS 'When true, a worker entering the trigger radius of a task plot for at least auto_start_min_dwell_seconds automatically opens a work interval.';

COMMENT ON COLUMN order_auto_automation_settings.auto_start_trigger_radius_meters
IS 'Radius around the task plot (site boundary) within which a worker presence counts as "on the plot" for auto-start. Separate from presence_radius_m used by the time-based engine.';

COMMENT ON COLUMN order_auto_automation_settings.auto_start_min_dwell_seconds
IS 'Continuous seconds the worker must remain within the trigger radius before the interval opens. Prevents false starts from GPS jitter or brief boundary crossings.';

COMMENT ON COLUMN order_auto_automation_settings.auto_end_enabled
IS 'When true, a worker leaving the trigger radius for longer than auto_end_min_absence_seconds automatically closes the work interval.';

COMMENT ON COLUMN order_auto_automation_settings.auto_end_trigger_radius_meters
IS 'Radius around the task plot for auto-end. Can differ from auto_start_trigger_radius_meters (e.g., tighter for end to avoid premature stops when turning at field edge).';

COMMENT ON COLUMN order_auto_automation_settings.auto_end_min_absence_seconds
IS 'Continuous seconds the worker must remain outside the trigger radius before the interval closes. Prevents false stops from GPS jitter or brief exits (tractor turns).';

COMMENT ON COLUMN order_auto_automation_settings.auto_end_grace_seconds
IS 'Additional grace period after absence threshold before the interval actually closes. The stop timestamp remains the last position inside the radius, not when grace expires.';

-- ---------------------------------------------------------------------------
-- Index for finding orders with GPS automation enabled
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_order_auto_automation_gps_enabled
    ON order_auto_automation_settings(tenant_id, order_id)
    WHERE auto_start_enabled OR auto_end_enabled;

-- ---------------------------------------------------------------------------
-- Grant
-- ---------------------------------------------------------------------------
GRANT SELECT, INSERT, UPDATE, DELETE ON order_auto_automation_settings TO agrocore_app;