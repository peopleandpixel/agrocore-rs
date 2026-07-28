#!/usr/bin/env python3
"""
Discover available SIGPAC parquet files on source.coop
"""

import requests
import re
import json

REGIONS = [
    'es-an',  # Andalusia
    'es-ar',  # Aragon
    'es-ct',  # Catalonia
    'es-cl',  # Castile and León
    'es-nc',  # Navarre
    'es-pv',  # Basque Country
    'es-cm',  # Castile-La Mancha
    'es-vc',  # Valencia
    'es-ga',  # Galicia
    'es-ex',  # Extremadura
    'es-md',  # Madrid
    'es-mc',  # Murcia
    'es-ri',  # La Rioja
    'es-pm',  # Balearic Islands
    'es-cn',  # Canary Islands
    'es-cb',  # Cantabria
]

def find_parquet_urls(page_html):
    """Extract parquet URLs from source.coop page HTML"""
    urls = set()
    
    # Pattern 1: Direct data.source.coop URLs
    pattern1 = r'https://data\.source\.coop/fiboa/[a-z-]+/[a-z0-9_\.-]+\.parquet'
    urls.update(re.findall(pattern1, page_html))
    
    # Pattern 2: JSON props in Next.js __NEXT_DATA__ script
    next_data_match = re.search(r'<script id="__NEXT_DATA__" type="application/json">(.*?)</script>', page_html, re.DOTALL)
    if next_data_match:
        try:
            data = json.loads(next_data_match.group(1))
            # Navigate to page props
            props = data.get('props', {}).get('pageProps', {})
            
            # Look for product data
            if 'product' in props:
                product = props['product']
                # Files array
                if 'files' in product:
                    for f in product['files']:
                        if f.get('url', '').endswith('.parquet'):
                            urls.add(f['url'])
                # Direct download URL
                if 'downloadUrl' in product and product['downloadUrl'].endswith('.parquet'):
                    urls.add(product['downloadUrl'])
        except Exception as e:
            pass
    
    # Pattern 3: Look for any .parquet in href attributes
    pattern3 = r'href=["\']([^"\']*\.parquet[^"\']*)["\']'
    urls.update(re.findall(pattern3, page_html))
    
    return list(urls)


def check_region(region):
    """Check a single region for parquet files"""
    url = f"https://source.coop/fiboa/{region}"
    try:
        r = requests.get(url, timeout=10)
        if r.status_code == 200:
            parquet_urls = find_parquet_urls(r.text)
            if parquet_urls:
                return parquet_urls
    except Exception as e:
        pass
    return []


def main():
    print("Discovering SIGPAC parquet files on source.coop...")
    print("=" * 60)
    
    all_urls = {}
    
    for region in REGIONS:
        print(f"Checking {region}...", end=' ', flush=True)
        urls = check_region(region)
        if urls:
            all_urls[region] = urls
            print(f"✓ Found {len(urls)} parquet file(s)")
            for u in urls:
                print(f"  {u}")
        else:
            print("✗ No parquet files found")
    
    print("\n" + "=" * 60)
    print("SUMMARY:")
    for region, urls in all_urls.items():
        print(f"\n{region}:")
        for u in urls:
            print(f"  {u}")
    
    # Save to JSON
    import json
    with open('sigpac_sources.json', 'w') as f:
        json.dump(all_urls, f, indent=2)
    print("\nSaved to sigpac_sources.json")


if __name__ == '__main__':
    main()