#!/usr/bin/env python3
"""
SIGPAC Reference Data Import Script
Imports official Spanish SIGPAC parcel data from fiboa GeoParquet files
into the sigpac_parcels PostgreSQL table.

Data source: https://source.coop/fiboa/spain (CC-BY-4.0)
~10M parcels from Andalusia, Aragon, Catalonia, Castile and León, Navarre, Basque Country
"""

import os
import sys
import argparse
import logging
from pathlib import Path
from typing import Optional, List, Dict, Any
from dataclasses import dataclass

import geopandas as gpd
import pandas as pd
from sqlalchemy import create_engine, text, Column, String, Integer, Float, DateTime, func
from sqlalchemy.dialects.postgresql import UUID, JSONB
from sqlalchemy.orm import sessionmaker, declarative_base
from sqlalchemy.dialects.postgresql import insert as pg_insert
from geoalchemy2 import Geometry
from geoalchemy2.shape import to_shape
from shapely.geometry import shape, mapping
from shapely.ops import transform
import pyproj
import requests
from tqdm import tqdm

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Database configuration
DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://agrocore:agrocore-secret@localhost:5432/agrocore"
)

# SIGPAC data sources (fiboa GeoParquet files from source.coop)
SIGPAC_SOURCES = {
    "andalusia": "https://data.source.coop/fiboa/es-an/es_an.parquet",
    "aragon": "https://data.source.coop/fiboa/es-ar/es_ar.parquet",
    "catalonia": "https://data.source.coop/fiboa/es-ct/es_ct.parquet",
    "castile_leon": "https://data.source.coop/fiboa/es-cl/es_cl.parquet",
    "navarre": "https://data.source.coop/fiboa/es-nc/es_nc.parquet",
    "basque": "https://data.source.coop/fiboa/es-pv/es_pv.parquet",
    "castile_la_mancha": "https://data.source.coop/fiboa/es-cm/es_cm.parquet",
    "balearic": "https://data.source.coop/fiboa/es-pm/es_pm.parquet",
    "canary": "https://data.source.coop/fiboa/es-cn/es_cn.parquet",
    "cantabria": "https://data.source.coop/fiboa/es-cb/es_cb.parquet",
    "extremadura": "https://data.source.coop/fiboa/es-ex/es_ex_2023.parquet",
    "galicia": "https://data.source.coop/fiboa/es-ga/es_ga.parquet",
    "la_rioja": "https://data.source.coop/fiboa/es-ri/es_ri.parquet",
    "madrid": "https://data.source.coop/fiboa/es-md/es_md.parquet",
    "murcia": "https://data.source.coop/fiboa/es-mc/es_mc.parquet",
    "valencia": "https://data.source.coop/fiboa/es-vc/es_vc_2023.parquet",
}

# Province code mapping (INE codes)
PROVINCE_CODES = {
    # Andalusia
    '04': 'Almería', '11': 'Cádiz', '14': 'Córdoba', '18': 'Granada',
    '21': 'Huelva', '23': 'Jaén', '29': 'Málaga', '41': 'Sevilla',
    # Aragon
    '22': 'Huesca', '44': 'Teruel', '50': 'Zaragoza',
    # Catalonia
    '08': 'Barcelona', '17': 'Girona', '25': 'Lleida', '43': 'Tarragona',
    # Castile and León
    '05': 'Ávila', '09': 'Burgos', '24': 'León', '34': 'Palencia',
    '37': 'Salamanca', '40': 'Segovia', '42': 'Soria', '47': 'Valladolid',
    '49': 'Zamora',
    # Castile-La Mancha
    '02': 'Albacete', '13': 'Ciudad Real', '16': 'Cuenca', '19': 'Guadalajara',
    '45': 'Toledo',
    # Navarre
    '31': 'Navarra',
    # Basque Country
    '20': 'Gipuzkoa', '48': 'Bizkaia', '01': 'Araba/Álava',
    # Extremadura
    '06': 'Badajoz', '10': 'Cáceres',
    # Galicia
    '15': 'A Coruña', '27': 'Lugo', '32': 'Ourense', '36': 'Pontevedra',
    # Madrid
    '28': 'Madrid',
    # Murcia
    '30': 'Murcia',
    # Valencia
    '03': 'Alicante', '12': 'Castellón', '46': 'Valencia',
    # Balearic
    '07': 'Illes Balears',
    # Canary
    '35': 'Las Palmas', '38': 'Santa Cruz de Tenerife',
    # Cantabria
    '39': 'Cantabria',
    # La Rioja
    '26': 'La Rioja',
}

# Reverse mapping
PROVINCE_TO_CODE = {v: k for k, v in PROVINCE_CODES.items()}

Base = declarative_base()


class SigpacParcel(Base):
    """SQLAlchemy model for sigpac_parcels table"""
    __tablename__ = 'sigpac_parcels'

    id = Column(UUID(as_uuid=True), primary_key=True, server_default=func.uuid_generate_v4())
    tenant_id = Column(UUID(as_uuid=True), nullable=False)
    province = Column(Integer, nullable=False)
    municipality = Column(Integer, nullable=False)
    aggregate = Column(Integer, nullable=False)
    zone = Column(Integer, nullable=False)
    polygon = Column(Integer, nullable=False)
    parcel = Column(Integer, nullable=False)
    enclosure = Column(Integer, nullable=False)
    sigpac_reference = Column(String(20))
    usage_code = Column(String(10))
    usage_description = Column(String(255))
    official_area_ha = Column(Float)
    geometry = Column(Geometry('POLYGON', srid=4326), nullable=False)
    source_year = Column(Integer)
    source_dataset = Column(String(100))
    created_at = Column(DateTime(timezone=True), server_default=func.now())

    def __repr__(self):
        return f"<SigpacParcel({self.sigpac_reference})>"


def get_province_code_from_fiboa(region: str, properties: Dict) -> int:
    """Extract province code from fiboa properties or region mapping"""
    # Try to get from properties first
    if 'province' in properties:
        prov = properties['province']
        if isinstance(prov, str) and prov.isdigit():
            return int(prov)
        if isinstance(prov, int):
            return prov

    # Try name mapping
    if 'province_name' in properties:
        name = properties['province_name']
        if name in PROVINCE_TO_CODE:
            return int(PROVINCE_TO_CODE[name])

    # Region-based mapping
    region_province_map = {
        'andalusia': list(range(4, 30))[::7],  # 04,11,14,18,21,23,29,41
        'aragon': [22, 44, 50],
        'catalonia': [8, 17, 25, 43],
        'castile_leon': [5, 9, 24, 34, 37, 40, 42, 47, 49],
        'castile_la_mancha': [2, 13, 16, 19, 45],
        'navarre': [31],
        'basque': [20, 48, 1],
        'extremadura': [6, 10],
        'galicia': [15, 27, 32, 36],
        'madrid': [28],
        'murcia': [30],
        'valencia': [3, 12, 46],
        'balearic': [7],
        'canary': [35, 38],
        'cantabria': [39],
        'la_rioja': [26],
    }

    provinces = region_province_map.get(region.lower(), [1])
    return provinces[0] if provinces else 1


def extract_sigpac_components(fid: str, properties: Dict) -> Dict[str, int]:
    """Extract SIGPAC components from feature ID or properties"""
    # SIGPAC reference format: PPMMMAAAZZZPPPPEEE (20 digits)
    # PP=province, MMM=municipality, AAA=aggregate, ZZZ=zone, PPPP=parcel, EEE=enclosure
    # In fiboa, we have: admin_province_code, admin_municipality_code
    # The numeric id can be used for the parcel/enclosure parts
    
    components = {}

    # Try to extract from fid first (should contain SIGPAC reference)
    if fid and len(fid) >= 20:
        try:
            components['province'] = int(fid[0:2])
            components['municipality'] = int(fid[2:5])
            components['aggregate'] = int(fid[5:8])
            components['zone'] = int(fid[8:11])
            components['polygon'] = int(fid[11:14])
            components['parcel'] = int(fid[14:17])
            components['enclosure'] = int(fid[17:20])
            return components
        except (ValueError, IndexError):
            pass

    # Fallback to properties - fiboa uses admin_province_code, admin_municipality_code
    # Use numeric id for parcel/enclosure to ensure uniqueness
    numeric_id = properties.get('id', '0')
    try:
        numeric_id_int = int(numeric_id)
    except (ValueError, TypeError):
        numeric_id_int = 0

    # Extract available fields
    province = properties.get('admin_province_code', '0')
    municipality = properties.get('admin_municipality_code', '0')

    try:
        components['province'] = int(province)
    except (ValueError, TypeError):
        components['province'] = 0

    try:
        components['municipality'] = int(municipality)
    except (ValueError, TypeError):
        components['municipality'] = 0

    # For the remaining fields, derive from numeric_id
    # Use last 10 digits of numeric_id for aggregate+zone+polygon+parcel+enclosure
    # If numeric_id is not available, use 0
    if numeric_id_int > 0:
        # Distribute numeric_id across the remaining fields
        components['aggregate'] = (numeric_id_int // 1000000) % 1000
        components['zone'] = (numeric_id_int // 1000) % 1000
        components['polygon'] = (numeric_id_int // 100) % 10
        components['parcel'] = (numeric_id_int // 10) % 10
        components['enclosure'] = numeric_id_int % 10
    else:
        components['aggregate'] = 0
        components['zone'] = 0
        components['polygon'] = 0
        components['parcel'] = 0
        components['enclosure'] = 0

    # Ensure all components exist
    for field in ['province', 'municipality', 'aggregate', 'zone', 'polygon', 'parcel', 'enclosure']:
        if field not in components:
            components[field] = 0

    return components


def process_geoparquet_file(
    file_path: str,
    region: str,
    year: int,
    engine,
    batch_size: int = 10000,
    limit: Optional[int] = None
) -> Dict[str, int]:
    """Process a single GeoParquet file and import to database"""

    logger.info(f"Reading {file_path}...")
    gdf = gpd.read_parquet(file_path)

    if limit:
        gdf = gdf.head(limit)

    logger.info(f"Loaded {len(gdf)} features from {region}")

    # Ensure CRS is WGS84
    if gdf.crs != "EPSG:4326":
        logger.info(f"Reprojecting from {gdf.crs} to EPSG:4326")
        gdf = gdf.to_crs("EPSG:4326")

    stats = {'inserted': 0, 'skipped': 0, 'errors': 0}

    Session = sessionmaker(bind=engine)
    session = Session()

    try:
        # Process in batches
        for i in tqdm(range(0, len(gdf), batch_size), desc=f"Importing {region}"):
            batch = gdf.iloc[i:i + batch_size]

            records = []
            for _, row in batch.iterrows():
                try:
                    props = row.drop('geometry').to_dict() if 'geometry' in row else row.to_dict()
                    geom = row.geometry

                    if geom is None or geom.is_empty:
                        stats['skipped'] += 1
                        continue

                    # Extract SIGPAC components
                    fid = props.get('id', '')
                    components = extract_sigpac_components(fid, props)

                    # Get province
                    province = components.get('province', 0)
                    if province == 0:
                        province = get_province_code_from_fiboa(region, props)

                    # Usage: str, props)

                    # Usage code
                    usage_code = props.get('crop:code', props.get('cultivo', props.get('usage_code', '')))
                    if usage_code and not isinstance(usage_code, str):
                        usage_code = str(usage_code)

                    # Area
                    area_ha = props.get('area_ha', props.get('hectareas', None))
                    if area_ha is not None:
                        try:
                            area_ha = float(area_ha)
                        except (ValueError, TypeError):
                            area_ha = geom.area * 111320 * 111320 / 10000  # rough conversion
                    else:
                        area_ha = None

                    record = {
                        'tenant_id': '00000000-0000-0000-0000-000000000000',  # global reference data
                        'province': province,
                        'municipality': components.get('municipality', 0),
                        'aggregate': components.get('aggregate', 0),
                        'zone': components.get('zone', 0),
                        'polygon': components.get('polygon', 0),
                        'parcel': components.get('parcel', 0),
                        'enclosure': components.get('enclosure', 0),
                        'usage_code': usage_code[:10] if usage_code else None,
                        'usage_description': props.get('crop:name', props.get('cultivo_nombre', None)),
                        'official_area_ha': area_ha,
                        'geometry': f'SRID=4326;{geom.wkt}',
                        'source_year': year,
                    }
                    records.append(record)

                except Exception as e:
                    logger.warning(f"Error processing feature: {e}")
                    stats['errors'] += 1
                    continue

            if records:
                try:
                    # Use upsert (ON CONFLICT DO NOTHING)
                    # Note: sigpac_reference is a generated column, conflict on unique constraint on sigpac_reference
                    stmt = pg_insert(SigpacParcel).values(records)
                    stmt = stmt.on_conflict_do_nothing(index_elements=['sigpac_reference'])
                    result = session.execute(stmt)
                    stats['inserted'] += result.rowcount
                    session.commit()
                except Exception as e:
                    logger.error(f"Batch insert error: {e}")
                    session.rollback()
                    stats['errors'] += len(records)

    finally:
        session.close()

    return stats


def download_geoparquet(url: str, output_dir: Path) -> Optional[Path]:
    """Download a GeoParquet file from URL"""
    filename = url.split('/')[-1]
    output_path = output_dir / filename

    if output_path.exists():
        logger.info(f"File already exists: {output_path}")
        return output_path

    logger.info(f"Downloading {url}...")
    try:
        response = requests.get(url, stream=True, timeout=300)
        response.raise_for_status()

        total_size = int(response.headers.get('content-length', 0))
        with open(output_path, 'wb') as f, tqdm(
            desc=filename,
            total=total_size,
            unit='B',
            unit_scale=True,
            unit_divisor=1024,
        ) as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    pbar.update(len(chunk))

        logger.info(f"Downloaded to {output_path}")
        return output_path

    except Exception as e:
        logger.error(f"Download failed: {e}")
        if output_path.exists():
            output_path.unlink()
        return None


def main():
    parser = argparse.ArgumentParser(description='Import SIGPAC reference data')
    parser.add_argument('--region', choices=list(SIGPAC_SOURCES.keys()) + ['all'],
                        default='all', help='Region to import')
    parser.add_argument('--year', type=int, default=2023, help='Data year')
    parser.add_argument('--limit', type=int, help='Limit number of features (for testing)')
    parser.add_argument('--batch-size', type=int, default=10000, help='Batch size for inserts')
    parser.add_argument('--download-dir', default='./data/sigpac', help='Directory for downloaded files')
    parser.add_argument('--skip-download', action='store_true', help='Skip download, use existing files')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be done without importing')
    args = parser.parse_args()

    # Setup
    download_dir = Path(args.download_dir)
    download_dir.mkdir(parents=True, exist_ok=True)

    engine = create_engine(DATABASE_URL, pool_size=5, max_overflow=10)

    if args.dry_run:
        logger.info("DRY RUN - no data will be imported")
        regions = list(SIGPAC_SOURCES.keys()) if args.region == 'all' else [args.region]
        for region in regions:
            url = SIGPAC_SOURCES[region]
            logger.info(f"Would download: {url}")
            logger.info(f"Would import region: {region} (year: {args.year})")
        return

    # Determine regions to process
    if args.region == 'all':
        regions = list(SIGPAC_SOURCES.keys())
    else:
        regions = [args.region]

    total_stats = {'inserted': 0, 'skipped': 0, 'errors': 0}

    for region in regions:
        url = SIGPAC_SOURCES.get(region)
        if not url:
            logger.warning(f"Unknown region: {region}")
            continue

        # Adjust URL for year if needed
        if args.year != 2023:
            url = url.replace('2023', str(args.year))

        # Download
        if not args.skip_download:
            file_path = download_geoparquet(url, download_dir)
            if not file_path:
                logger.error(f"Failed to download {region}, skipping")
                continue
        else:
            file_path = download_dir / url.split('/')[-1]
            if not file_path.exists():
                logger.error(f"File not found: {file_path}, use --skip-download only if files exist")
                continue

        # Import
        logger.info(f"Importing {region} from {file_path}...")
        stats = process_geoparquet_file(
            str(file_path),
            region,
            args.year,
            engine,
            batch_size=args.batch_size,
            limit=args.limit
        )

        logger.info(f"{region}: inserted={stats['inserted']}, skipped={stats['skipped']}, errors={stats['errors']}")
        total_stats['inserted'] += stats['inserted']
        total_stats['skipped'] += stats['skipped']
        total_stats['errors'] += stats['errors']

    logger.info(f"Total: inserted={total_stats['inserted']}, skipped={total_stats['skipped']}, errors={total_stats['errors']}")


if __name__ == '__main__':
    main()