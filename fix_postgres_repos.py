#!/usr/bin/env python3
"""Fix PostgreSQL Repositories - PaginatedResponse field names"""
import re
import os

POSTGRES_DIR = "/home/jens/RustroverProjects/agrocore-rs/crates/infrastructure/src/postgres"

def fix_repo(filepath: str):
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Fix PaginatedResponse fields
    content = content.replace('Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })', '''Ok(PaginatedResponse {
                data: items,
                total: total as u64,
                page: p.page.unwrap_or(0),
                per_page: p.per_page.unwrap_or(20),
                total_pages: if total == 0 { 0 } else { (total as f64 / (p.per_page.unwrap_or(20) as f64)).ceil() as u64 },
            })''')
    
    # Fix .limit und .per_page falsche Queries
    content = re.sub(r'\.bind\(p\.limit as i32\)', '.bind(per_page as i32)', content)
    content = re.sub(r'\.bind\(\(p\.page - 1\) \* p\.limit\)', '.bind(offset as i32)', content)
    
    with open(filepath, 'w') as f:
        f.write(content)
    
    print(f"Fixed: {filepath}")

if __name__ == "__main__":
    for fname in os.listdir(POSTGRES_DIR):
        if fname.endswith('.rs'):
            fix_repo(os.path.join(POSTGRES_DIR, fname))
    print("Done!")