#!/usr/bin/env python3
"""
Download and import a small SIGPAC dataset for testing
"""

import os
import sys
import argparse
import tempfile
from pathlib import Path
import requests
from tqdm import tqdm

# Add venv to path
sys.path.insert(0, str(Path(__file__).parent / "venv" / "lib" / "python3.14" / "site-packages"))

import geopandas as gpd
from sqlalchemy import create_engine, text
from geoalchemy2 import Geometry

# SIGPAC sources - using smaller regions first
SIGPAC_SOURCES = {
    "andalusia": "https://data.source.coop/fiboa/es-an/es_an.parquet",
    "aragon": "https://data.source.coop/fiboa/es-ar/es_ar.parquet",
    "catalonia": "https://data.source.coop/fiboa/es-ct/es_ct.parquet",
    "castile_leon": "https://data.source.coop/fiboa/es-cl/es_cl.parquet",
    "navarre": "https://data.source.coop/fiboa/es-nc/es_nc.parquet",
    "basque": "https://data.source.coop/fiboa/es-pv/es_pv.parquet",
    "castile_la_mancha": "https://data.source.coop/fiboa/es-cm/es_cm.parquet",
    "valencia": "https://data.source.coop/fiboa/es-vc/es_vc_2023.parquet",
    "galicia": "https://data.source.coop/fiboa/es-ga/es_ga.parquet",
    "extremadura": "https://data.source.coop/fiboa/es-ex/es_ex_2023.parquet",
    "madrid": "https://data.source.coop/fiboa/es-md/es_md.parquet",
    "murcia": "https://data.source.coop/fiboa/es-mc/es_mc.parquet",
    "balearic": "https://data.source.coop/fiboa/es-pm/es_pm.parquet",
    "canary": "https://data.source.coop/fiboa/es-cn/es_cn.parquet",
    "cantabria": "https://data.source.coop/fiboa/es-cb/es_cb_2023.parquet",
    "la_rioja": "https://data.source.coop/fiboa/es-ri/es_ri.parquet",
}

DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://agrocore:agrocore-secret@localhost:5432/agrocore")


def download_file(url: str, dest: Path, chunk_size: int = 8192) -> bool:
    """Download file with progress bar"""
    try:
        response = requests.get(url, stream=True, timeout=30)
        response.raise_for_status()
        total = int(response.headers.get('content-length', 0))
        
        with open(dest, 'wb') as f, tqdm(
            total=total, unit='B', unit_scale=True, unit_divisor=1024,
            desc=dest.name
        ) as pbar:
            for chunk in response.iter_content(chunk_size=chunk_size):
                if chunk:
                    f.write(chunk)
                    pbar.update(len(chunk))
        return True
    except Exception as e:
        print(f"Download failed: {e}")
        if dest.exists():
            dest.unlink()
        return False


def import_sigpac(
    region: str,
    year: int = 2023,
    limit: int = None,
    batch_size: int = 5000,
    download_dir: str = None,
    skip_download: bool = False
):
    """Import SIGPAC data for a region"""

    if region not in SIGPAC_SOURCES:
        print(f"Unknown region: {region}")
        print(f"Available: {', '.join(SIGPAC_SOURCES.keys())}")
        return False

    url = SIGPAC_SOURCES[region]
    download_path = Path(download_dir or tempfile.gettempdir()) / f"{region}_{year}.parquet"

    # Download
    if not skip_download:
        print(f"Downloading {region} from {url}...")
        if not download_file(url, download_path):
            return False
        print(f"Downloaded to {download_path}")
    else:
        print(f"Using existing file: {download_path}")

    # Read parquet
    print(f"Reading {region} data...")
    gdf = gpd.read_parquet(download_path)
    print(f"Loaded {len(gdf)} features, CRS: {gdf.crs}")

    if limit:
        gdf = gdf.head(limit)
        print(f"Limited to {limit} features")

    # Ensure WGS84
    if gdf.crs != "EPSG:4326":
        print(f"Reprojecting from {gdf.crs} to EPSG:4326")
        gdf = gdf.to_crs("EPSG:4326")

    # Database connection
    engine = create_engine(DATABASE_URL)

    # Create table if not exists
    with engine.connect() as conn:
        conn.execute(text("""
            CREATE TABLE IF NOT EXISTS sigpac_parcels_test (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                tenant_id UUID NOT NULL,
                province SMALLINT NOT NULL,
                municipality SMALLINT NOT NULL,
                aggregate SMALLINT NOT NULL,
                zone SMALLINT NOT NULL,
                polygon SMALLINT NOT NULL,
                parcel SMALLINT NOT NULL,
                enclosure SMALLINT NOT NULL,
                sigpac_reference VARCHAR(20),
                usage_code VARCHAR(10),
                usage_description TEXT,
                official_area_ha DOUBLE PRECISION,
                geometry GEOMETRY(POLYGON, 4326) NOT NULL,
                source_year SMALLINT,
                source_dataset VARCHAR(100),
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        """))
        conn.execute(text("CREATE INDEX IF NOT EXISTS idx_sigpac_geometry_test ON sigpac_parcels_test USING GIST(geometry)"))
        conn.commit()

    # Import in batches
    tenant_id = "00000000-0000-0000-0000-000000000001"  # demo tenant
    
    total = len(gdf)
    inserted = 0
    
    for i in tqdm(range(0, total, batch_size), desc="Importing"):
        batch = gdf.iloc[i:i+batch_size]
        
        # Prepare data for batch insert
        records = []
        for idx, row in batch.iterrows():
            props = row.drop('geometry').to_dict() if 'geometry' in row else row.to_dict()
            geom = row.geometry
            
            if geom is None or geom.is_empty:
                continue
            
            # Extract SIGPAC components
            fid = props.get('id', '')
            province = props.get('province', 0)
            municipality = props.get('municipality', 0)
            aggregate = props.get('aggregate', 0)
            zone = props.get('zone', 0)
            polygon = props.get('polygon', 0)
            parcel = props.get('parcel', 0)
            enclosure = props.get('enclosure', 0)
            
            # Build reference
            ref = f"{province:02d}{municipality:03d}{aggregate:03d}{zone:03d}{polygon:03d}{parcel:03d}{enclosure:03d}"
            
            usage_code = props.get('crop:code', props.get('cultivo', props.get('usage_code', '')))
            if usage_code and not isinstance(usage_code, str):
                usage_code = str(usage_code)
            
            area = props.get('area_ha', props.get('hectareas', props.get('official_area_ha')))
            if area is not None:
                try:
                    area = float(area)
                except:
                    area = None
            
            records.append({
                'tenant_id': tenant_id,
                'province': int(province),
                'municipality': int(municipality),
                'aggregate': int(aggregate),
                'zone': int(zone),
                'polygon': int(polygon),
                'parcel': int(parcel),
                'enclosure': int(enclosure),
                'sigpac_reference': ref,
                'usage_code': str(usage_code) if usage_code else None,
                'official_area_ha': area,
                'geometry': geom.wkt,
                'source_year': year,
                'source_dataset': f'fiboa_{region}',
            })
        
        if records:
            # Batch insert
            with engine.connect() as conn:
                for rec in records:
                    conn.execute(text("""
                        INSERT INTO sigpac_parcels_test (
                            tenant_id, province, municipality, aggregate, zone, polygon, parcel, enclosure,
                            sigpac_reference, usage_code, official_area_ha, geometry, source_year, source_dataset
                        ) VALUES (
                            :tenant_id, :province, :municipality, :aggregate, :zone, :polygon, :parcel, :enclosure,
                            :sigpac_reference, :usage_code, :official_area_ha, ST_GeomFromText(:geometry, 4326), :source_year, :source_dataset
                        )
                    """), rec)
                conn.commit()
            inserted += len(records)
    
    print(f"\n✓ Imported {inserted} parcels for {region}")
    return True


def main():
    parser = argparse.ArgumentParser(description="Import SIGPAC reference data")
    parser.add_argument("region", choices=list(SIGPAC_SOURCES.keys()) + ["all"], help="Region to import")
    parser.add_argument("--year", type=int, default=2023, help="Data year")
    parser.add_argument("--limit", type=int, help="Limit number of features")
    parser.add_argument("--batch-size", type=int, default=5000, help="Batch size for inserts")
    parser.add_argument("--download-dir", type=str, help="Directory for downloaded files")
    parser.add_argument("--skip-download", action="store_true", help="Skip download, use existing file")
    
    args = parser.parse_args()
    
    if args.region == "all":
        for region in SIGPAC_SOURCES:
            import_sigpac(region, args.year, args.limit, args.batch_size, args.download_dir, args.skip_download)
    else:
        import_sigpac(args.region, args.year, args.limit, args.batch_size, args.download_dir, args.skip_download)


if __name__ == "__main__":
    main()