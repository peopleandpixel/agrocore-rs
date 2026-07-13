#!/bin/bash
# Fix all PostgreSQL repositories - correct PaginatedResponse field names
# Usage: ./fix_all_pg_repos.sh

set -e

POSTGRES_DIR="/home/jens/RustroverProjects/agrocore-rs/crates/infrastructure/src/postgres"

for file in "$POSTGRES_DIR"/*.rs; do
    echo "Fixing $file..."
    
    # Fix PaginatedResponse with wrong fields
    # Pattern 1: items/total/page/limit -> data/total/page/per_page/total_pages
    sed -i 's/Ok(PaginatedResponse { items, total, page: p.page, limit: p.limit })/Ok(PaginatedResponse { data: items, total: total as u64, page: p.page.unwrap_or(0), per_page: p.per_page.unwrap_or(20), total_pages: (total as f64 \/ (p.per_page.unwrap_or(20) as f64)).ceil() as u64 })/g' "$file"
    
    sed -i 's/\.bind(p.limit as i32)/.bind(per_page as i32)/g' "$file"
    sed -i 's/\.bind(((p.page - 1) \* p.limit) as i32)/.bind(offset as i32)/g' "$file"
    
    # Add total_pages calculation where missing
    sed -i 's/total: total as u64,/total: total as u64,\n                total_pages: if total == 0 { 0 } else { (total as f64 \/ per_page as f64).ceil() as u64 },/g' "$file"
done

echo "Done!"