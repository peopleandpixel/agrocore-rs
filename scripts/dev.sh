#!/usr/bin/env bash
#
# scripts/dev.sh — Local development environment for agrocore-rs
#
# Starts PostgreSQL (PostGIS), NATS, and all Rust microservices, waits for each
# to become healthy, then opens an interactive tmux dashboard showing live
# service status and logs.
#
# Press Ctrl-C to stop everything cleanly.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ── Ports ──────────────────────────────────────────────────────────────────
API_PORT="${API_PORT:-3000}"
WEATHER_PORT="${WEATHER_PORT:-3010}"
REPORTING_PORT="${REPORTING_PORT:-3002}"
GEOMETRY_PORT="${GEOMETRY_PORT:-3003}"
ASSET_REGISTRY_PORT="${ASSET_REGISTRY_PORT:-3004}"
ADMIN_UI_PORT="${ADMIN_UI_PORT:-8080}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
NATS_PORT="${NATS_PORT:-4222}"
NATS_MONITOR_PORT="${NATS_MONITOR_PORT:-8222}"
DATABASE_NAME="${DATABASE_NAME:-agrocore}"
JWT_SECRET="${JWT_SECRET:-dev-secret}"

POSTGRES_CONTAINER="agrocore-dev-postgres"
NATS_CONTAINER="agrocore-dev-nats"

service_pids=()
TMUX_SESSION="agrocore-dev"

# ── Helpers ────────────────────────────────────────────────────────────────

mkdir -p "$ROOT_DIR/target"

wait_for_port() {
    local host="$1"
    local port="$2"
    local label="$3"

    for _ in $(seq 1 60); do
        if timeout 1 bash -c "cat < /dev/null > /dev/tcp/${host}/${port}" 2>/dev/null; then
            echo "  ✅ ${label} listening on ${host}:${port}"
            return 0
        fi
        sleep 1
    done

    echo "  ❌ Timed out waiting for ${label} on ${host}:${port}" >&2
    exit 1
}

wait_for_http() {
    local url="$1"
    local label="$2"
    local timeout_secs="${3:-60}"

    for _ in $(seq 1 "$((timeout_secs))"); do
        if curl -sf "$url" > /dev/null 2>&1; then
            echo "  ✅ ${label} healthy at ${url}"
            return 0
        fi
        sleep 1
    done

    echo "  ❌ Timed out waiting for ${label} at ${url}" >&2
    exit 1
}

check_port_free() {
    local port="$1"
    local label="$2"
    if timeout 1 bash -c "cat < /dev/null > /dev/tcp/127.0.0.1/${port}" 2>/dev/null; then
        echo "Warning: Port $port ($label) is already in use — will try to stop conflicting containers." >&2
        # If it's our container, stop it
        case "$port" in
            "$POSTGRES_PORT"|"$NATS_PORT"|"$NATS_MONITOR_PORT")
                docker ps --format '{{.Names}}' | grep -q "^${POSTGRES_CONTAINER}$" && docker stop "$POSTGRES_CONTAINER" >/dev/null 2>&1 || true
                docker ps --format '{{.Names}}' | grep -q "^${NATS_CONTAINER}$" && docker stop "$NATS_CONTAINER" >/dev/null 2>&1 || true
                sleep 2
                if timeout 1 bash -c "cat < /dev/null > /dev/tcp/127.0.0.1/${port}" 2>/dev/null; then
                    echo "Error: Port $port ($label) still in use after cleanup." >&2
                    return 1
                fi
                ;;
            *)
                echo "Error: Port $port ($label) is in use by another process." >&2
                return 1
                ;;
        esac
    fi
    return 0
}

start_service() {
    local label="$1"
    local workdir="$2"
    shift 2

    echo "Starting ${label}..."
    (
        cd "$workdir"
        exec env "$@"
    ) > "$ROOT_DIR/target/${label// /-}.log" 2>&1 &
    service_pids+=("$!:$label")
}

cleanup() {
    local exit_code=$?

    trap - EXIT INT TERM

    echo ""
    echo "Shutting down services..."

    for entry in "${service_pids[@]:-}"; do
        pid="${entry%%:*}"
        label="${entry##*:}"
        kill "$pid" >/dev/null 2>&1 || true
    done

    for entry in "${service_pids[@]:-}"; do
        pid="${entry%%:*}"
        wait "$pid" >/dev/null 2>&1 || true
    done

    docker stop "$POSTGRES_CONTAINER" "$NATS_CONTAINER" >/dev/null 2>&1 || true
    tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true

    echo "All services stopped."
    exit "$exit_code"
}

trap cleanup EXIT INT TERM

# ── Check prerequisites ────────────────────────────────────────────────────

echo "=== AGROCORE-RS Local Development ==="
echo ""
echo "Checking prerequisites..."

for port in $API_PORT $WEATHER_PORT $REPORTING_PORT $GEOMETRY_PORT $ASSET_REGISTRY_PORT $ADMIN_UI_PORT $POSTGRES_PORT $NATS_PORT $NATS_MONITOR_PORT; do
    check_port_free "$port" "port" || true
done

# ── Docker infra ──────────────────────────────────────────────────────────

echo ""
echo "Starting infrastructure containers..."

docker rm -f "$POSTGRES_CONTAINER" "$NATS_CONTAINER" >/dev/null 2>&1 || true

docker run -d --rm \
    --name "$POSTGRES_CONTAINER" \
    -p "${POSTGRES_PORT}:5432" \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB="$DATABASE_NAME" \
    postgis/postgis:latest > /dev/null

docker run -d --rm \
    --name "$NATS_CONTAINER" \
    -p "${NATS_PORT}:4222" \
    -p "${NATS_MONITOR_PORT}:8222" \
    nats:latest > /dev/null

wait_for_port 127.0.0.1 "$POSTGRES_PORT" "PostgreSQL"
echo "Waiting for PostgreSQL to be fully ready..."
until docker exec "$POSTGRES_CONTAINER" pg_isready -U postgres >/dev/null 2>&1; do
    sleep 1
done
echo "  ✅ PostgreSQL ready"

wait_for_port 127.0.0.1 "$NATS_PORT" "NATS"
echo "  ✅ NATS ready"

# ── Run database migrations ────────────────────────────────────────────────

echo ""
echo "Running database migrations..."

export DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}"
# Apply migrations BEFORE cargo build so sqlx::query! macros can verify against the schema
if command -v sqlx &> /dev/null; then
    sqlx migrate run 2>&1 | tail -1
else
    echo "  (sqlx-cli not found — API will auto-migrate on startup)"
    # API auto-migrates, but cargo build needs the schema compiled at compile time
    # so we need sqlx-cli to set up the database first
    echo "  ⚠ sqlx-cli required for compile-time query verification"
    exit 1
fi
echo "  ✅ Migrations applied"

# ── Start services ─────────────────────────────────────────────────────────

echo "Starting microservices..."

# Pre-compile all services so cargo run startup is fast
echo "Pre-compiling services (this may take a while on first run)..."
export DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}"
cargo build -p agrocore-api -p agrocore-weather-service -p agrocore-reporting-service -p agrocore-geometry-service -p agrocore-asset-registry 2>&1 | tail -3
echo "  ✅ Pre-compilation done"

start_service \
    "API" \
    "$ROOT_DIR" \
    DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
    LISTEN_ADDR="0.0.0.0:${API_PORT}" \
    NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
    JWT_SECRET="$JWT_SECRET" \
    cargo run -p agrocore-api --offline 2>>"$ROOT_DIR/target/api.stderr.log"

wait_for_http "http://127.0.0.1:${API_PORT}/api/v1/health" "API" 120

start_service \
    "Reporting Service" \
    "$ROOT_DIR" \
    DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
    NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
    LISTEN_ADDR="0.0.0.0:${REPORTING_PORT}" \
    JWT_SECRET="$JWT_SECRET" \
    cargo run -p agrocore-reporting-service

wait_for_http "http://127.0.0.1:${REPORTING_PORT}/health" "Reporting Service" 120

start_service \
    "Weather Service" \
    "$ROOT_DIR" \
    DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
    NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
    LISTEN_ADDR="0.0.0.0:${WEATHER_PORT}" \
    JWT_SECRET="$JWT_SECRET" \
    cargo run -p agrocore-weather-service

wait_for_http "http://127.0.0.1:${WEATHER_PORT}/health" "Weather Service" 120

start_service \
    "Geometry Service" \
    "$ROOT_DIR" \
    DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
    NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
    LISTEN_ADDR="0.0.0.0:${GEOMETRY_PORT}" \
    JWT_SECRET="$JWT_SECRET" \
    cargo run -p agrocore-geometry-service

wait_for_http "http://127.0.0.1:${GEOMETRY_PORT}/health" "Geometry Service" 60

start_service \
    "Asset Registry" \
    "$ROOT_DIR" \
    DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
    NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
    LISTEN_ADDR="0.0.0.0:${ASSET_REGISTRY_PORT}" \
    JWT_SECRET="$JWT_SECRET" \
    cargo run -p agrocore-asset-registry

wait_for_http "http://127.0.0.1:${ASSET_REGISTRY_PORT}/health" "Asset Registry" 60

start_service \
    "Admin UI" \
    "$ROOT_DIR/crates/admin-ui" \
    AGROCORE_API_BASE_URL="http://localhost:${API_PORT}" \
    trunk serve --port "$ADMIN_UI_PORT" --address 0.0.0.0

wait_for_http "http://127.0.0.1:${ADMIN_UI_PORT}" "Admin UI" 120

echo ""
echo "All services are up!"

# ── Dashboard ──────────────────────────────────────────────────────────────

echo ""
echo "=== AGROCORE-RS DASHBOARD ==="
echo "  API:        http://localhost:${API_PORT}"
echo "  Admin UI:   http://localhost:${ADMIN_UI_PORT}"
echo "  NATS Mon:   http://localhost:${NATS_MONITOR_PORT}"
echo "  Weather:    http://localhost:${WEATHER_PORT}"
echo "  Reporting:  http://localhost:${REPORTING_PORT}"
echo "  Geometry:   http://localhost:${GEOMETRY_PORT}"
echo "  Asset Reg:  http://localhost:${ASSET_REGISTRY_PORT}"
echo "  Logs:       $ROOT_DIR/target/*.log"
echo "  Stop:       Ctrl-C"
echo ""

if command -v tmux &> /dev/null && [ -z "${NO_TMUX:-}" ]; then
    echo "Starting tmux dashboard..."

    tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true
    tmux new-session -d -s "$TMUX_SESSION" -x "$(tput cols 2>/dev/null || echo 120)" -y 40

    # Pane 1: Dashboard status
    tmux new-window -t "$TMUX_SESSION" -n "dashboard"
    tmux send-keys -t "$TMUX_SESSION:0" "
        echo '=== AGROCORE-RS SERVICE STATUS ==='
        while true; do
            clear
            echo '=== AGROCORE-RS SERVICE STATUS ==='
            echo ''
            for entry in ${service_pids[*]}; do
                pid=\${entry%%:*}
                label=\${entry##*:}
                if kill -0 \$pid 2>/dev/null; then
                    printf '  ✅ %-20s RUNNING (PID %s)\n' \"\$label\" \"\$pid\"
                else
                    printf '  ❌ %-20s STOPPED\n' \"\$label\"
                fi
            done
            echo ''
            echo 'Endpoints:'
            echo '  API:        http://localhost:${API_PORT}'
            echo '  Admin UI:   http://localhost:${ADMIN_UI_PORT}'
            echo '  NATS:       nats://localhost:${NATS_PORT}'
            echo ''
            echo 'Press Ctrl-C in the status window or close tmux to exit.'
            echo 'Refresh every 5s...'
            sleep 5
        done
    " C-m

    # Pane 2: API logs
    tmux new-window -t "$TMUX_SESSION" -n "api"
    tmux send-keys -t "$TMUX_SESSION:1" "tail -f $ROOT_DIR/target/API.log" C-m

    # Pane 3: Weather logs
    tmux new-window -t "$TMUX_SESSION" -n "weather"
    tmux send-keys -t "$TMUX_SESSION:2" "tail -f $ROOT_DIR/target/Weather-Service.log" C-m

    # Pane 4: All service PIDs
    tmux new-window -t "$TMUX_SESSION" -n "all-logs"
    tmux send-keys -t "$TMUX_SESSION:3" "tail -f $ROOT_DIR/target/*.log" C-m

    tmux select-window -t "$TMUX_SESSION:0"
    tmux attach-session -t "$TMUX_SESSION"
else
    echo "tmux not found. Running in foreground mode."
    echo "Press Ctrl-C to stop."
    while true; do
        for entry in "${service_pids[@]:-}"; do
            pid="${entry%%:*}"
            label="${entry##*:}"
            if ! kill -0 "$pid" 2>/dev/null; then
                wait "$pid" 2>/dev/null || true
                exit_code=$?
                echo "Service ${label} (PID ${pid}) exited with code ${exit_code}"
                exit "$exit_code"
            fi
        done
        sleep 5
    done
fi
