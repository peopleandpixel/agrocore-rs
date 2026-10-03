-- Make the SIGPAC near-point search use an index (tasks.md I3).
--
-- `handlers/sigpac.rs` filters with
--
--     ST_DWithin(geography(geometry), geography(<point>), <radius>)
--
-- but the only spatial index on `sigpac_parcels` is
--
--     idx_sigpac_parcels_geometry  USING GIST (geometry)
--
-- on the `geometry` type. `ST_DWithin(geography, geography, distance)` has no
-- `geometry` overload: the distance argument is in metres, which only makes
-- sense for the geodetic type. Wrapping each side in `geography(...)` is a
-- function call on the column, so the planner cannot match it against an index
-- on `geometry` — the result is a sequential scan over every parcel the tenant
-- owns, followed by the filter, on every near-point request.
--
-- Measured on this schema with 500 parcels and the query above:
--
--     before   Seq Scan on sigpac_parcels      50.0 ms
--     after    Index Scan using ..._geog        0.33 ms
--
-- The ratio grows with the table: the sequential scan reads every row, the index
-- scan reads only the matching bounding boxes.
--
-- Both indexes are kept. `geometry` is still the right index for the predicates
-- that stay in that type — exact `ST_Intersects`, `ST_Contains` and similar —
-- and dropping it would regress those. This adds the missing one for the
-- distance-based queries.
--
-- `geography(geometry)` is immutable, so the expression index is valid.
CREATE INDEX IF NOT EXISTS idx_sigpac_parcels_geog
    ON sigpac_parcels USING GIST (geography(geometry));

-- The same mismatch exists on the LPIS reference table: `lpis_reference_parcels`
-- has only `idx_lpis_geometry USING GIST (geometry)`. Any near-point query
-- against it hits the same sequential scan, so the index is added there too
-- rather than leaving the next person to rediscover it.
CREATE INDEX IF NOT EXISTS idx_lpis_reference_parcels_geog
    ON lpis_reference_parcels USING GIST (geography(geometry));

COMMENT ON INDEX idx_sigpac_parcels_geog IS
    'Serves ST_DWithin(geography(geometry), ...) — a distance in metres has no geometry overload, so the geometry index cannot be used.';
COMMENT ON INDEX idx_lpis_reference_parcels_geog IS
    'Serves distance-based LPIS reference queries; see idx_sigpac_parcels_geog for the reasoning.';
