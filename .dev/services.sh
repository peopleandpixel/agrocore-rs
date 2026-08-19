#!/usr/bin/env bash
# .dev/services.sh — Start/stop individual agrocore services as background processes
# Each service writes to its own PID file in .dev/pids/ and log in target/

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_DIR="$ROOT_DIR/.dev/pids"
LOG_DIR="$ROOT_DIR/target"
mkdir -p "$PID_DIR" "$LOG_DIR"

# Database connection — keep consistent with docker-compose.yml
DB_PASSWORD="postgres"
export DATABASE_URL="postgres://postgres:${DB_PASSWORD}@127.0.0.1:5432/agrocore"
export NATS_URL="nats://127.0.0.1:4222"
export JWT_SECRET="${JWT_SECRET:-dev-secret}"

# ─── Service definitions ─────────────────────────────────────────────────────
# Format: NAME|PORT|CRATE|EXTRA_ENV|LOG_NAME

services_list() {
    cat <<'EOF'
api|3000|agrocore-api|LISTEN_ADDR=0.0.0.0:3000|api
admin-ui|8080|admin-ui|AGROCORE_API_BASE_URL=http://localhost:3000|admin-ui
reporting|3002|agrocore-reporting-service|LISTEN_ADDR=0.0.0.0:3002|reporting
weather|3010|agrocore-weather-service|LISTEN_ADDR=0.0.0.0:3010|weather
geometry|3003|agrocore-geometry-service|LISTEN_ADDR=0.0.0.0:3003|geometry
asset-registry|3004|agrocore-asset-registry|LISTEN_ADDR=0.0.0.0:3004|asset-registry
EOF
}

# ─── Helpers ─────────────────────────────────────────────────────────────────

is_running() {
    local name="$1"
    local pidfile="$PID_DIR/${name}.pid"
    [ -f "$pidfile" ] || return 1
    local pid
    pid=$(cat "$pidfile" 2>/dev/null) || return 1
    kill -0 "$pid" 2>/dev/null
}

get_pid() {
    cat "$PID_DIR/$1.pid" 2>/dev/null || echo ""
}

start_service() {
    local name="$1"
    local port="$2"
    local crate="$3"
    local extra_env="$4"
    local log_name="$5"

    if is_running "$name"; then
        echo "⚠ $name already running (PID $(get_pid "$name"))"
        return 0
    fi

    local log_file="$LOG_DIR/${log_name}.log"
    local env_vars="$DATABASE_URL $NATS_URL $JWT_SECRET $extra_env"

    echo "Starting $name (port $port)..."

    # Build env array
    local -a env_array=()
    env_array+=("DATABASE_URL=$DATABASE_URL")
    env_array+=("NATS_URL=$NATS_URL")
    env_array+=("JWT_SECRET=$JWT_SECRET")

    # Parse EXTRA_ENV
    if [ -n "$extra_env" ]; then
        IFS=';' read -ra PAIRS <<< "$extra_env"
        for pair in "${PAIRS[@]}"; do
            env_array+=("$pair")
        done
    fi

    if [ "$crate" = "admin-ui" ]; then
        (
            cd "$ROOT_DIR/crates/admin-ui"
            exec env "${env_array[@]}" trunk serve --port "$port" --address 0.0.0.0
        ) > "$log_file" 2>&1 &
    else
        (
            cd "$ROOT_DIR"
            exec env "${env_array[@]}" cargo run -p "$crate" --offline
        ) > "$log_file" 2>&1 &
    fi

    local pid=$!
    echo "$pid" > "$PID_DIR/${name}.pid"
    echo "✅ $name started (PID $pid)"
}

stop_service() {
    local name="$1"
    local pidfile="$PID_DIR/${name}.pid"

    if ! is_running "$name"; then
        echo "⚠ $name is not running"
        rm -f "$pidfile"
        return 0
    fi

    local pid
    pid=$(get_pid "$name")
    kill "$pid" 2>/dev/null || true
    sleep 2
    # Force kill if still alive
    if kill -0 "$pid" 2>/dev/null; then
        kill -9 "$pid" 2>/dev/null || true
    fi
    rm -f "$pidfile"
    echo "🛑 $name stopped"
}

status_service() {
    local name="$1"
    local port="$2"

    if is_running "$name"; then
        local pid
        pid=$(get_pid "$name")
        printf '  ✅ %-20s RUNNING (PID %s) port %s\n' "$name" "$pid" "$port"
    else
        printf '  ❌ %-20s STOPPED port %s\n' "$name" "$port"
    fi
}

case "${1:-}" in
    start)
        line=$(services_list | grep "^${2}|") || { echo "Unknown service: $2"; exit 1; }
        IFS='|' read -r name port crate extra_env log_name <<< "$line"
        start_service "$name" "$port" "$crate" "$extra_env" "$log_name"
        ;;
    stop)
        stop_service "$2"
        ;;
    restart)
        line=$(services_list | grep "^${2}|") || { echo "Unknown service: $2"; exit 1; }
        IFS='|' read -r name port crate extra_env log_name <<< "$line"
        stop_service "$2"
        sleep 1
        start_service "$name" "$port" "$crate" "$extra_env" "$log_name"
        ;;
    status)
        echo "=== Agrocore Service Status ==="
        while IFS='|' read -r name port crate extra_env log_name; do
            status_service "$name" "$port"
        done <<< "$(services_list)"
        echo ""
        echo "Infrastructure:"
        docker ps --filter "name=agrocore-dev" --format "  🐳 {{.Names}} {{.Status}}"
        ;;
    logs)
        line=$(services_list | grep "^${2}|") || { echo "Unknown service: $2"; exit 1; }
        IFS='|' read -r name port crate extra_env log_name <<< "$line"
        local log_file="$LOG_DIR/${log_name}.log"
        if [ -f "$log_file" ]; then
            tail -f "$log_file"
        else
            echo "No log file at $log_file"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs} [service-name]"
        echo ""
        echo "Available services:"
        while IFS='|' read -r name port crate extra_env log_name; do
            echo "  $name (port $port)"
        done <<< "$(services_list)"
        exit 1
        ;;
esac
