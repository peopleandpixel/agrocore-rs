#!/usr/bin/env bash
#
# scripts/dev.sh — Slim local development environment for agrocore-rs
#
# What's new (v2):
#   - Infrastructure (PostgreSQL + NATS) runs via docker compose with PERSISTENT volume
#   - No more `docker rm -f` — database survives dev.sh restarts
#   - No tmux! Interactive dashboard via .dev/dashboard.sh
#   - Services started/stopped individually via PID tracking
#   - Restart a single service without touching the DB or other services
#
# Workflow:
#   1. ./scripts/dev.sh          → starts infra + launches interactive dashboard
#   2. In dashboard: 's'         → starts all services
#   3. In dashboard: 'r api'     → restarts only the API (hot reload workflow)
#   4. In dashboard: 'l api'      → view live API logs
#   5. In dashboard: 'q'          → exit dashboard (services stay running)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ── Configuration ─────────────────────────────────────────────────────────────
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
NATS_PORT="${NATS_PORT:-4222}"
NATS_MONITOR_PORT="${NATS_MONITOR_PORT:-8222}"
DATABASE_NAME="${DATABASE_NAME:-agrocore}"
# Load DB password from env file (avoids terminal password masking issues)
# The password is base64-encoded to prevent the terminal from stripping/masking it
if [ -f "$ROOT_DIR/.dev/db_password.env" ]; then
    DB_PASSWORD=$(base64 -d "$ROOT_DIR/.dev/db_password.env" | tr -d '\n')
else
    echo "  ❌ Missing .dev/db_password.env — run 'echo cG9zdGdyZXM= > .dev/db_password.env' first" >&2
    exit 1
fi
export DATABASE_URL="postgres://postgres:${DB_PASSWORD}@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}"

# ── Helpers ───────────────────────────────────────────────────────────────────

wait_for_pg_isready() {
    local retries=30
    until docker exec agrocore-dev-postgres pg_isready -U postgres >/dev/null 2>&1; do
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            echo "  ❌ PostgreSQL did not become ready in time" >&2
            return 1
        fi
        sleep 1
    done
    echo "  ✅ PostgreSQL ready"
}

wait_for_port() {
    local host="$1" port="$2" label="$3"
    for _ in $(seq 1 30); do
        if timeout 1 bash -c "cat < /dev/null > /dev/tcp/${host}/${port}" 2>/dev/null; then
            echo "  ✅ ${label} listening on ${host}:${port}"
            return 0
        fi
        sleep 1
    done
    echo "  ❌ Timed out waiting for ${label} on ${host}:${port}" >&2
    return 1
}

wait_for_http() {
    local url="$1" label="$2" timeout_secs="${3:-120}"
    for _ in $(seq 1 "$timeout_secs"); do
        if curl -sf "$url" > /dev/null 2>&1; then
            echo "  ✅ ${label} healthy at ${url}"
            return 0
        fi
        sleep 1
    done
    echo "  ❌ Timed out waiting for ${label} at ${url}" >&2
    return 1
}

cleanup() {
    echo ""
    echo "Shutting down services..."
    # Stop tracked service PIDs but NOT the DB containers
    local pid_dir="$ROOT_DIR/.dev/pids"
    if [ -d "$pid_dir" ]; then
        for pidfile in "$pid_dir"/*.pid; do
            [ -f "$pidfile" ] || continue
            local pid
            pid=$(cat "$pidfile" 2>/dev/null) || continue
            kill "$pid" 2>/dev/null || true
        done
        for pidfile in "$pid_dir"/*.pid; do
            [ -f "$pidfile" ] || continue
            local pid
            pid=$(cat "$pidfile" 2>/dev/null) || continue
            kill -9 "$pid" 2>/dev/null || true
        done
        rm -f "$pid_dir"/*.pid 2>/dev/null || true
    fi
    echo "Services stopped. Database containers left running."
    docker stop agrocore-dev-postgres agrocore-dev-nats >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ── Start ─────────────────────────────────────────────────────────────────────

echo "=== AGROCORE-RS Local Development (v2) ==="
echo ""

# Setup directories
mkdir -p "$ROOT_DIR/.dev/pids" "$ROOT_DIR/target"

# Start infrastructure via docker compose (persistent volume)
echo "Starting infrastructure (PostgreSQL + NATS)..."
cd "$ROOT_DIR/.dev"
docker compose up -d postgres nats 2>&1 | tail -5
cd "$ROOT_DIR"
wait_for_port 127.0.0.1 "$POSTGRES_PORT" "PostgreSQL" || exit 1
# pg_isready checks if PostgreSQL is accepting connections, but password auth
# may not be ready yet. Use a real connection test:
for i in $(seq 1 30); do
    if PGPASSWORD="$DB_PASSWORD" psql -h 127.0.0.1 -U postgres -d "$DATABASE_NAME" -c "SELECT 1" >/dev/null 2>&1; then
        echo "  ✅ PostgreSQL ready (auth OK)"
        sleep 2  # Extra grace period for role/auth cache to stabilize
        break
    fi
    [ $i -eq 30 ] && { echo "  ❌ PostgreSQL did not become ready in time" >&2; exit 1; }
    sleep 1
done
wait_for_port 127.0.0.1 "$NATS_PORT" "NATS" || exit 1
echo "  ✅ NATS ready"

# Run database migrations (retry for auth readiness)
echo ""
echo "Running database migrations..."
if ! command -v sqlx &> /dev/null; then
    echo "  ❌ sqlx-cli not found — required for compile-time query verification" >&2
    exit 1
fi

for attempt in 1 2 3 4 5; do
    if DATABASE_URL="$DATABASE_URL" sqlx migrate run 2>&1 | tail -1; then
        echo "  ✅ Migrations applied"
        break
    fi
    echo "  ⚠ Migration attempt $attempt failed, retrying in 3s..."
    [ $attempt -eq 5 ] && { echo "  ❌ Failed to apply migrations after 5 attempts" >&2; exit 1; }
    sleep 3
done

# Pre-compile services
echo ""
echo "Pre-compiling services..."
cargo build -p agrocore-api -p agrocore-weather-service -p agrocore-reporting-service -p agrocore-geometry-service -p agrocore-asset-registry 2>&1 | tail -3
echo "  ✅ Pre-compilation done"

echo ""
echo "=== Ready! Starting interactive dashboard ==="
echo ""

# Launch the interactive dashboard (no tmux!)
# Dashboard provides:
#   - Real-time service status
#   - 'r <service>' to restart individual services (hot reload)
#   - 'l <service>' to view live logs
#   - 's' to start all services
#   - 'q' to quit (services stay running)
exec bash "$ROOT_DIR/.dev/dashboard.sh"
