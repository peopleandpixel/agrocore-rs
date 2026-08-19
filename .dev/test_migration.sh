#!/usr/bin/env bash
set -euo pipefail
DB_PASSWORD=$(base64 -d /home/jens/RustroverProjects/agrocore-rs/.dev/db_password.env | tr -d '\n')
export DATABASE_URL="postgres://postgres:${DB_PASSWORD}@127.0.0.1:5432/agrocore"
echo "DB_PASSWORD length: ${#DB_PASSWORD}"
echo "DB_PASSWORD hex: $(printf '%s' "$DB_PASSWORD" | xxd -p)"
echo "DATABASE_URL length: ${#DATABASE_URL}"
echo "DATABASE_URL hex: $(printf '%s' "$DATABASE_URL" | xxd -p)"
echo "=== Testing psql ==="
PGPASSWORD="$DB_PASSWORD" psql -h 127.0.0.1 -U postgres -d agrocore -c "SELECT 1;" 2>&1
echo "=== Testing sqlx ==="
sqlx migrate run 2>&1
echo "EXIT_CODE=$?"
