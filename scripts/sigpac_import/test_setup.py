#!/usr/bin/env python3
"""
Quick test script to verify SIGPAC import setup
"""

import os
import sys

def test_imports():
    """Test all required imports"""
    try:
        import geopandas as gpd
        print("✓ geopandas")
    except ImportError as e:
        print(f"✗ geopandas: {e}")
        return False

    try:
        import pandas as pd
        print("✓ pandas")
    except ImportError as e:
        print(f"✗ pandas: {e}")
        return False

    try:
        from sqlalchemy import create_engine
        print("✓ sqlalchemy")
    except ImportError as e:
        print(f"✗ sqlalchemy: {e}")
        return False

    try:
        from geoalchemy2 import Geometry
        print("✓ geoalchemy2")
    except ImportError as e:
        print(f"✗ geoalchemy2: {e}")
        return False

    try:
        import psycopg2
        print("✓ psycopg2")
    except ImportError as e:
        print(f"✗ psycopg2: {e}")
        return False

    try:
        import pyproj
        print("✓ pyproj")
    except ImportError as e:
        print(f"✗ pyproj: {e}")
        return False

    try:
        import requests
        print("✓ requests")
    except ImportError as e:
        print(f"✗ requests: {e}")
        return False

    try:
        from tqdm import tqdm
        print("✓ tqdm")
    except ImportError as e:
        print(f"✗ tqdm: {e}")
        return False

    return True


def test_database_connection():
    """Test database connection"""
    try:
        from sqlalchemy import create_engine, text
        db_url = os.getenv("DATABASE_URL", "postgresql://agrocore:agrocore-secret@localhost:5432/agrocore")
        engine = create_engine(db_url)
        with engine.connect() as conn:
            result = conn.execute(text("SELECT 1"))
            result.fetchone()
        print("✓ Database connection")
        return True
    except Exception as e:
        print(f"✗ Database connection: {e}")
        return False


def test_postgis():
    """Test PostGIS extension"""
    try:
        from sqlalchemy import create_engine, text
        db_url = os.getenv("DATABASE_URL", "postgresql://agrocore:agrocore-secret@localhost:5432/agrocore")
        engine = create_engine(db_url)
        with engine.connect() as conn:
            result = conn.execute(text("SELECT PostGIS_Version()"))
            version = result.fetchone()[0]
        print(f"✓ PostGIS: {version}")
        return True
    except Exception as e:
        print(f"✗ PostGIS: {e}")
        return False


if __name__ == "__main__":
    print("Testing SIGPAC Import Setup\n")
    
    all_ok = True
    all_ok &= test_imports()
    print()
    all_ok &= test_database_connection()
    all_ok &= test_postgis()
    
    print()
    if all_ok:
        print("✓ All tests passed!")
        sys.exit(0)
    else:
        print("✗ Some tests failed")
        sys.exit(1)