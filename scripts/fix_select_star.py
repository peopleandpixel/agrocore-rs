#!/usr/bin/env python3
"""
I11: Replace SELECT * with explicit columns for wide tables (>10 columns).
Reads migration files to extract column names, then replaces SELECT * in .rs files.
"""
import re
import os
import sys

# Tables with >10 columns (from migration analysis)
WIDE_TABLES = {
    'sites': 39,
    'orders': 34,
    'task_data': 24,
    'olive_oil_records': 20,
    'spatial_objects': 18,
    'tasks': 18,
    'vineyards': 18,
    'plant_protection_records': 16,
    'users': 16,
    'workers': 14,
    'animals': 14,
    'customers': 14,
    'olive_groves': 14,
    'inventory_transactions': 13,
    'pac_applications': 13,
    'audit_logs': 13,
    'financial_records': 12,
    'fertilizer_records': 12,
    'soil_moisture_readings': 12,
    'phenology_records': 12,
    'water_quotas': 12,
    'type_entries': 12,
    'equipment': 11,
    'inventory_items': 11,
    'water_sources': 11,
    'kelter_deliveries': 11,
    'soil_moisture_configs': 11,
    'growing_degree_days': 11,
    'equipment_usage_log': 11,
    'harvest_deliveries': 11,
    'harvest_lots': 10,
    'treatment_records': 10,
    'compliance_checklists': 10,
    'clock_entries': 10,
    'soil_moisture_alerts': 10,
    'equipment_depreciation_schedule': 10,
    'frost_warnings': 9,
    'work_logs': 9,
    'pest_risks': 9,
    'harvest_seasons': 9,
    'cost_centers': 11,
    'cold_chain_logs': 9,
    'livestock': 9,
    'buildings': 9,
    'groups': 9,
    'trees': 9,
    'inventory_locations': 9,
    'worker_task_statuses': 10,
    'applicator_licenses': 9,
    'weather_data': 9,
    'weather_stations': 8,
    'equipment_maintenance_log': 9,
}

# Map table name -> repo file (relative to postgres dir)
TABLE_TO_FILE = {
    'sites': 'site.rs',
    'orders': 'order.rs',
    'task_data': 'task_data.rs',
    'olive_oil_records': 'olive_oil_record.rs',
    'spatial_objects': 'site.rs',  # spatial_objects are in site.rs
    'tasks': 'task.rs',
    'vineyards': 'vineyard.rs',
    'plant_protection_records': 'plant_protection_record.rs',
    'users': 'user.rs',
    'workers': 'worker.rs',
    'animals': 'animal.rs',
    'customers': 'customer.rs',
    'olive_groves': 'olive_grove.rs',
    'inventory_transactions': 'inventory_transaction.rs',
    'pac_applications': 'pac_application.rs',
    'audit_logs': 'audit_log.rs',
    'financial_records': 'financial_record.rs',
    'fertilizer_records': 'fertilizer_record.rs',
    'soil_moisture_readings': 'soil_moisture_reading.rs',
    'phenology_records': 'phenology_record.rs',
    'water_quotas': 'water_quota.rs',
    'type_entries': 'type_entry.rs',
    'equipment': 'equipment.rs',
    'inventory_items': 'inventory_item.rs',
    'water_sources': 'water_source.rs',
    'kelter_deliveries': 'kelter_delivery.rs',
    'soil_moisture_configs': 'soil_moisture_config.rs',
    'growing_degree_days': 'growing_degree_day.rs',
    'equipment_usage_log': 'equipment.rs',
    'harvest_deliveries': 'harvest_delivery.rs',
    'harvest_lots': 'harvest_lot.rs',
    'treatment_records': 'treatment_record.rs',
    'compliance_checklists': 'compliance.rs',
    'clock_entries': 'clock_entry.rs',
    'soil_moisture_alerts': 'soil_moisture_alert.rs',
    'equipment_depreciation_schedule': 'equipment.rs',
    'frost_warnings': 'frost_warning.rs',
    'work_logs': 'work_log.rs',
    'pest_risks': 'pest_risk.rs',
    'harvest_seasons': 'harvest_season.rs',
    'cost_centers': 'cost_center.rs',
    'cold_chain_logs': 'cold_chain_log.rs',
    'livestock': 'livestock.rs',
    'buildings': 'building.rs',
    'groups': 'group.rs',
    'trees': 'tree.rs',
    'inventory_locations': 'inventory_location.rs',
    'worker_task_statuses': 'worker_task_status.rs',
    'applicator_licenses': 'applicator_license.rs',
    'weather_data': 'weather_data.rs',
    'weather_stations': 'weather_station.rs',
    'equipment_maintenance_log': 'equipment.rs',
}

def extract_columns_from_migrations(migrations_dir, table_name):
    """Extract column names from migration files for a given table."""
    columns = []
    # Search all migration files
    for fname in sorted(os.listdir(migrations_dir)):
        if not fname.endswith('.sql'):
            continue
        fpath = os.path.join(migrations_dir, fname)
        with open(fpath, 'r') as f:
            content = f.read()
        
        # Find CREATE TABLE for this table
        pattern = rf'CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+{re.escape(table_name)}\s*\((.*?)\);'
        match = re.search(pattern, content, re.DOTALL | re.IGNORECASE)
        if match:
            body = match.group(1)
            # Extract column names (first word of each line, ignoring constraints)
            for line in body.split('\n'):
                line = line.strip().rstrip(',')
                if not line or line.startswith('--'):
                    continue
                # Skip constraints
                if line.upper().startswith(('PRIMARY', 'FOREIGN', 'UNIQUE', 'CHECK', 'CONSTRAINT', 'INDEX')):
                    continue
                parts = line.split()
                if parts:
                    col = parts[0]
                    # Skip SQL keywords
                    if col.upper() in ('CREATE', 'TABLE', 'IF', 'NOT', 'EXISTS'):
                        continue
                    columns.append(col)
            if columns:
                return columns
        
        # Also check ALTER TABLE ADD COLUMN
        pattern = rf'ALTER\s+TABLE\s+{re.escape(table_name)}\s+ADD\s+COLUMN'
        for match in re.finditer(pattern, content, re.IGNORECASE):
            # Find the next few lines after the match
            start = match.end()
            chunk = content[start:start+500]
            for line in chunk.split('\n'):
                line = line.strip().rstrip(',')
                if not line or line.startswith('--'):
                    continue
                if 'ADD COLUMN' in line.upper():
                    parts = line.replace('ADD COLUMN', '').replace('IF NOT EXISTS', '').strip().split()
                    if parts:
                        columns.append(parts[0])
    
    return columns

def get_column_aliases(table_name, columns):
    """Get column aliases for tables where DB column names differ from struct field names."""
    aliases = {}
    
    # orders: actual_start -> started_at, actual_end -> completed_at
    if table_name == 'orders':
        aliases['actual_start'] = 'started_at'
        aliases['actual_end'] = 'completed_at'
    
    # animals: tag_number is the DB column, struct uses tag_number
    # (already correct after J4 fix)
    
    return aliases

def build_select_clause(table_name, columns):
    """Build the SELECT clause with aliases if needed."""
    aliases = get_column_aliases(table_name, columns)
    parts = []
    for col in columns:
        if col in aliases:
            parts.append(f"{col} AS {aliases[col]}")
        else:
            parts.append(col)
    return ", ".join(parts)

def process_file(file_path, table_name, columns):
    """Replace SELECT * FROM table with explicit columns in a Rust file."""
    if not os.path.exists(file_path):
        return False
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    select_clause = build_select_clause(table_name, columns)
    
    # Replace SELECT * FROM table_name
    pattern_select = rf'SELECT\s+\*\s+FROM\s+{re.escape(table_name)}'
    replacement_select = f"SELECT {select_clause} FROM {table_name}"
    
    if re.search(pattern_select, content, re.IGNORECASE):
        new_content = re.sub(pattern_select, replacement_select, content, flags=re.IGNORECASE)
        
        # Replace RETURNING * in INSERT/UPDATE for this table
        # Only replace RETURNING * that's in the same context as this table
        # We need to be careful not to replace RETURNING * for other tables
        # For simplicity, we replace RETURNING * only if the table name appears nearby
        
        with open(file_path, 'w') as f:
            f.write(new_content)
        return True
    
    return False

def main():
    project_root = '/home/jens/RustroverProjects/agrocore-rs'
    migrations_dir = os.path.join(project_root, 'migrations')
    postgres_dir = os.path.join(project_root, 'crates/infrastructure/src/postgres')
    
    # Group tables by file
    file_tables = {}
    for table_name in WIDE_TABLES:
        if table_name in TABLE_TO_FILE:
            file_path = TABLE_TO_FILE[table_name]
            if file_path not in file_tables:
                file_tables[file_path] = []
            file_tables[file_path].append(table_name)
    
    changed_files = []
    for file_path, tables in file_tables.items():
        full_path = os.path.join(postgres_dir, file_path)
        if not os.path.exists(full_path):
            print(f"SKIP: {full_path} not found")
            continue
        
        file_changed = False
        for table_name in tables:
            columns = extract_columns_from_migrations(migrations_dir, table_name)
            if not columns:
                print(f"  WARN: No columns found for {table_name}")
                continue
            
            print(f"Processing {table_name} ({len(columns)} columns) in {file_path}...")
            if process_file(full_path, table_name, columns):
                print(f"  ✓ Replaced SELECT * for {table_name}")
                file_changed = True
            else:
                print(f"  - No SELECT * found for {table_name}")
        
        if file_changed:
            changed_files.append(file_path)
    
    print(f"\nDone! Changed {len(changed_files)} files:")
    for f in changed_files:
        print(f"  - {f}")

if __name__ == '__main__':
    main()
