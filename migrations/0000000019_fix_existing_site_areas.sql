-- Fix: Recalculate area for all existing sites that have a boundary but area = 0 or NULL.
-- This addresses sites created before the trigger fix (migration 0018) where the trigger
-- was storing area in hectares instead of square meters, resulting in area ≈ 0 after display division.
--
-- The trigger only recalculates area when the boundary changes, so existing sites with
-- incorrect area values need this one-time fix.

UPDATE sites
SET area = calculate_area_hectares(boundary) * 10000
WHERE boundary IS NOT NULL
  AND (area IS NULL OR area = 0);
