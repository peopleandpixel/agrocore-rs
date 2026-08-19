#!/usr/bin/env bash
# .dev/dashboard.sh — Interactive dev dashboard (no tmux needed!)
# Shows service status, live logs, and lets you restart individual services

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_DIR="$ROOT_DIR/.dev/pids"
LOG_DIR="$ROOT_DIR/target"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Service definitions
declare -a SERVICES=(
    "api|3000|agrocore-api|api"
    "admin-ui|8080|admin-ui|admin-ui"
    "reporting|3002|agrocore-reporting-service|reporting"
    "weather|3010|agrocore-weather-service|weather"
    "geometry|3003|agrocore-geometry-service|geometry"
    "asset-registry|3004|agrocore-asset-registry|asset-registry"
)

get_service_field() {
    local name="$1" field="$2"
    for entry in "${SERVICES[@]}"; do
        IFS='|' read -r sname sport scrate slog <<< "$entry"
        if [ "$sname" = "$name" ]; then
            case "$field" in
                port) echo "$sport" ;;
                crate) echo "$scrate" ;;
                log) echo "$slog" ;;
            esac
            return 0
        fi
    done
}

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

check_http() {
    local port="$1"
    curl -sf "http://127.0.0.1:${port}/api/v1/health" -o /dev/null 2>/dev/null
    if [ $? -eq 0 ]; then
        return 0
    fi
    curl -sf "http://127.0.0.1:${port}/" -o /dev/null 2>/dev/null
    return $?
}

show_status() {
    clear
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              AGROCORE-RS Development Dashboard              ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Infrastructure
    echo -e "${BLUE}┌── Infrastructure ──┐${NC}"
    if pgrep -f "agrocore-dev-postgres" > /dev/null 2>&1 || docker ps --format '{{.Names}}' | grep -q "agrocore-dev-postgres" 2>/dev/null; then
        echo -e "  ${GREEN}✅ PostgreSQL${NC} (Docker)"
    else
        echo -e "  ${RED}❌ PostgreSQL${NC} (Docker)"
    fi
    if docker ps --format '{{.Names}}' | grep -q "agrocore-dev-nats" 2>/dev/null; then
        echo -e "  ${GREEN}✅ NATS${NC} (Docker)"
    else
        echo -e "  ${RED}❌ NATS${NC} (Docker)"
    fi
    echo ""

    # Services
    echo -e "${BLUE}┌── Services ───────────────────────┬───────┬───────┬───────┐${NC}"
    echo -e "${BLUE}│ Service                           │ Port  │ PID   │ Health│${NC}"
    echo -e "${BLUE}├───────────────────────────────────┼───────┼───────┼───────┤${NC}"

    for entry in "${SERVICES[@]}"; do
        IFS='|' read -r name port crate logname <<< "$entry"
        if is_running "$name"; then
            pid=$(get_pid "$name")
            if check_http "$port"; then
                health="${GREEN}✅ UP${NC}"
            else
                health="${YELLOW}⚠ PID${NC}"
            fi
            printf "│%-33s│${GREEN}%-6s${NC}│${GREEN}%-6s${NC}│%s│\n" "$name" "$port" "$pid" "$health"
        else
            printf "│%-33s│${RED}%-6s${NC}│${RED}%-6s${NC}│${RED}❌ DOWN${NC}│\n" "$name" "$port" "—"
        fi
    done
    echo -e "${BLUE}└───────────────────────────────────┴───────┴───────┴───────┘${NC}"
    echo ""

    # Menu
    echo -e "${YELLOW}Commands:${NC}"
    echo "  r <service>  - Restart a service (e.g. 'r api')"
    echo "  s            - Start all services"
    echo "  x <service>  - Stop a service (e.g. 'x api')"
    echo "  l <service>  - View live logs (Ctrl-C to exit log view)"
    echo "  q            - Quit dashboard (services stay running)"
    echo ""
}

show_logs() {
    local name="$1"
    local logname
    for entry in "${SERVICES[@]}"; do
        IFS='|' read -r sname sport scrate slog <<< "$entry"
        if [ "$sname" = "$name" ]; then
            logname="$slog"
            break
        fi
    done

    local log_file="$LOG_DIR/${logname}.log"
    if [ -f "$log_file" ]; then
        echo -e "${CYAN}=== Live logs for ${name} (Ctrl-C to return to dashboard) ===${NC}"
        echo ""
        tail -f "$log_file"
    else
        echo -e "${RED}No log file found: $log_file${NC}"
        echo "Try starting the service first."
        read -p "Press Enter to return..."
    fi
}

start_all_services() {
    echo -e "${BLUE}Starting all services...${NC}"
    for entry in "${SERVICES[@]}"; do
        IFS='|' read -r name port crate logname <<< "$entry"
        if is_running "$name"; then
            echo "  ⚠ $name already running"
            continue
        fi
        local pid
        local log_file="$LOG_DIR/${logname}.log"

        # Load DB password from env file (base64-encoded to avoid terminal masking)
        local db_pass
        db_pass=$(base64 -d "$ROOT_DIR/.dev/db_password.env" 2>/dev/null | tr -d '\n') || true
        # Set env vars
        local env_args="DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/agrocore NATS_URL=nats://127.0.0.1:4222 JWT_SECRET=***"

        if [ "$crate" = "admin-ui" ]; then
            env_args="DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/agrocore NATS_URL=nats://127.0.0.1:4222 JWT_SECRET=*** AGROCORE_API_BASE_URL=http://localhost:3000"
            (cd "$ROOT_DIR/crates/admin-ui" && exec env $env_args trunk serve --port "$port" --address 0.0.0.0) > "$log_file" 2>&1 &
        else
            env_args="DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/agrocore NATS_URL=nats://127.0.0.1:4222 JWT_SECRET=*** LISTEN_ADDR=0.0.0.0:$port"
            (cd "$ROOT_DIR" && exec env $env_args cargo run -p "$crate" --offline) > "$log_file" 2>&1 &
        fi
        pid=$!
        echo "$pid" > "$PID_DIR/${name}.pid"
        echo "  ✅ $name started (PID $pid, port $port)"
    done
    echo ""
    echo "Waiting for services to become healthy..."
    sleep 20
}

restart_service() {
    local name="$1"
    local port crate logname
    for entry in "${SERVICES[@]}"; do
        IFS='|' read -r sname sport scrate slog <<< "$entry"
        if [ "$sname" = "$name" ]; then
            port="$sport"
            crate="$scrate"
            logname="$slog"
            break
        fi
    done

    if [ -z "$name" ]; then
        echo -e "${RED}Unknown service: $1${NC}"
        echo "Available: api, admin-ui, reporting, weather, geometry, asset-registry"
        read -p "Press Enter to return..."
        return
    fi

    # Stop if running
    if is_running "$name"; then
        local pid
        pid=$(get_pid "$name")
        kill "$pid" 2>/dev/null || true
        sleep 2
        kill -9 "$pid" 2>/dev/null || true
        rm -f "$PID_DIR/${name}.pid"
        echo "  🛑 $name stopped"
    fi

    # Start
    local log_file="$LOG_DIR/${logname}.log"
    local env_args="DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/agrocore NATS_URL=nats://127.0.0.1:4222 JWT_SECRET=${JWT_SECRET:-dev-secret}"

    if [ "$crate" = "admin-ui" ]; then
        env_args="$env_args AGROCORE_API_BASE_URL=http://localhost:3000"
        (cd "$ROOT_DIR/crates/admin-ui" && exec env $env_args trunk serve --port "$port" --address 0.0.0.0) > "$log_file" 2>&1 &
    else
        env_args="$env_args LISTEN_ADDR=0.0.0.0:$port"
        (cd "$ROOT_DIR" && exec env $env_args cargo run -p "$crate" --offline) > "$log_file" 2>&1 &
    fi
    local pid=$!
    echo "$pid" > "$PID_DIR/${name}.pid"
    echo "  ✅ $name restarted (PID $pid, port $port)"
}

stop_service() {
    local name="$1"
    if is_running "$name"; then
        local pid
        pid=$(get_pid "$name")
        kill "$pid" 2>/dev/null || true
        sleep 2
        kill -9 "$pid" 2>/dev/null || true
        rm -f "$PID_DIR/${name}.pid"
        echo "  🛑 $name stopped"
    else
        echo "  ⚠ $name is not running"
    fi
}

# ─── Main loop ─────────────────────────────────────────────────────────────────

# Handle Ctrl-C by showing a fresh status screen (don't exit)
trap 'show_status' INT
trap 'exit 0' TERM

show_status

while true; do
    echo ""
    read -r -p "$(echo -e "${YELLOW}agrocore> ${NC}")" input
    if [ -z "$input" ]; then
        show_status
        continue
    fi

    cmd=$(echo "$input" | awk '{print $1}')
    arg=$(echo "$input" | awk '{print $2}')

    case "$cmd" in
        r)
            if [ -n "$arg" ]; then
                restart_service "$arg"
            else
                echo -e "${RED}Usage: r <service-name>${NC}"
            fi
            ;;
        s)
            start_all_services
            ;;
        x)
            if [ -n "$arg" ]; then
                stop_service "$arg"
            else
                echo -e "${RED}Usage: x <service-name>${NC}"
            fi
            ;;
        l)
            if [ -n "$arg" ]; then
                show_logs "$arg"
            else
                echo -e "${RED}Usage: l <service-name>${NC}"
            fi
            ;;
        q)
            echo -e "${GREEN}Dashboard quit. Services are still running.${NC}"
            echo "To stop all: kill \$(cat .dev/pids/*.pid) or run 'make stop'"
            exit 0
            ;;
        "")
            ;;
        *)
            echo -e "${RED}Unknown command: $cmd${NC}"
            echo "Commands: r (restart), s (start all), x (stop), l (logs), q (quit)"
            ;;
    esac

    show_status
done
