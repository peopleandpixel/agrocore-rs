-- Fix site area type: change from NUMERIC to DOUBLE PRECISION to match Rust f64
ALTER TABLE public.sites 
    ALTER COLUMN area TYPE DOUBLE PRECISION USING area::double precision,
    ALTER COLUMN gross_area TYPE DOUBLE PRECISION USING gross_area::double precision;
