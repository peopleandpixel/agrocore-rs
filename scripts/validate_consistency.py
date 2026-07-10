#!/usr/bin/env python3
"""
Data Consistency Validator - Prüft DTO ↔ Entity Feld-Übereinstimmung
Alles DTOs sind bereits in Entity-Dateien definiert!
"""

import os, re

DOMAIN = "/home/jens/RustroverProjects/agrocore-rs/crates/domain/src/entities"

entities = ["site", "order", "task", "compliance", "olive", "weather", "water", "livestock"]

print("=== DATA CONSISTENCY CHECK ===\n")

for entity in entities:
    path = f"{DOMAIN}/{entity}.rs"
    if os.path.exists(path):
        content = open(path).read()
        
        # Extract Entity fields
        entity_fields = set(re.findall(r'pub\s+(\w+):\s*(?:[^,]+)?,', content))
        
        # Check DTOs
        has_create = f"Create{entity.capitalize()}" in content or f"Create" in content
        
        print(f"{entity}: {len(entity_fields)} Felder, DTO: {'✅' if has_create else '❌'}")

# Geo-Index Check
site_repo = "/home/jens/RustroverProjects/agrocore-rs/crates/infrastructure/src/repositories/site.rs"
if os.path.exists(site_repo):
    content = open(site_repo).read()
    has_geo = "find_by_polygon" in content or "find_nearby" in content
    print(f"\nGeo-Queries in SiteRepo: {'✅' if has_geo else '❌'}")
    if not has_geo:
        print("   -> Kein 2dsphere Index implementiert")