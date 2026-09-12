#!/usr/bin/env bash
# dev.sh ── Start development environment with live dashboard
# Starts PostgreSQL, NATS, MQTT, Redis, API, Notification Service, and Admin UI,
# then launches the Ratatui dashboard.
# The dashboard polls services every 2s and shows build/git/system status.
#
# Usage: ./scripts/dev.sh [--demo] [--no-dashboard] [--stop|--down]
#   --demo           Start with demo data (creates demo tenant, users, sites, equipment, etc.)
#   --no-dashboard   Skip launching the Ratatui dashboard
#   --stop|--down    Stop and remove all dev containers (including volumes)
# Press 'q' in the dashboard to quit (services keep running).

set -euo pipefail

# ════════════════════════════════════════════════════════════════
# Colors
# ════════════════════════════════════════════════════════════════
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ════════════════════════════════════════════════════════════════
# Parse arguments
# ════════════════════════════════════════════════════════════════
DEMO_MODE=false
NO_DASHBOARD=false
STOP_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --demo)    DEMO_MODE=true ;;
        --no-dashboard) NO_DASHBOARD=true ;;
        --stop|--down) STOP_ONLY=true ;;
        *) echo -e "${RED}Unknown argument: $arg${NC}" >&2; exit 1 ;;
    esac
done

# Handle stop command early
if [[ "$STOP_ONLY" == true ]]; then
    echo -e "${BLUE}╔══ AgroCore Dev Environment ══╗${NC}"
    echo -e "${YELLOW}Stopping all dev containers...${NC}"
    if [[ -f docker-compose.dev.yml ]]; then
        docker compose -f docker-compose.dev.yml down -v 2>/dev/null || true
        docker compose -f docker-compose.dev.yml rm -f 2>/dev/null || true
        echo -e "${GREEN}✅ All containers stopped and volumes removed${NC}"
    else
        echo -e "${YELLOW}⚠️  docker-compose.dev.yml not found, nothing to stop${NC}"
    fi
    # Also clean up any orphaned agrocore containers
    docker ps -a --filter "name=agrocore-" --format "{{.Names}}" | xargs -r docker rm -f 2>/dev/null || true
    exit 0
fi

echo -e "${BLUE}╔══ AgroCore Dev Environment ══╗${NC}"
[[ "$DEMO_MODE" == true ]] && echo -e "${CYAN}🎪 Demo Mode aktiviert${NC}"
echo -e "${YELLOW}Starting infrastructure...${NC}"

# ════════════════════════════════════════════════════════════════
# Check if a TCP port is reachable from HOST
# ════════════════════════════════════════════════════════════════
port_reachable() {
    local port=$1
    local host=${2:-127.0.0.1}
    timeout 2 bash -c "cat < /dev/null > /dev/tcp/${host}/${port}" 2>/dev/null
}

# Wait for a TCP port to be reachable from host
wait_for_port() {
    local port=$1
    local name=$2
    local timeout_s=60
    local elapsed=0

    echo -n "    $name (port $port) ... "
    while [[ $elapsed -lt $timeout_s ]]; do
        if port_reachable "$port"; then
            echo -e "${GREEN}ready${NC}"
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    echo -e "${RED}timeout${NC}"
    return 1
}

# Check if a TCP port is in use (on host)
port_in_use() {
    local port=$1
    if command -v ss &>/dev/null; then
        ss -ltn 2>/dev/null | grep -q ":${port} "
    elif command -v netstat &>/dev/null; then
        netstat -ltn 2>/dev/null | grep -q ":${port} "
    else
        port_reachable "$port"
    fi
}

# Find next free port starting from base_port
find_free_port() {
    local base_port=$1
    local port=$base_port
    while port_in_use "$port"; do
        ((port++))
        if [[ $port -gt $((base_port + 100)) ]]; then
            echo -e "${RED}❌ Could not find free port near $base_port${NC}" >&2
            exit 1
        fi
    done
    echo "$port"
}

# ════════════════════════════════════════════════════════════════
# Check Docker availability and permissions
# ════════════════════════════════════════════════════════════════
check_docker() {
    if ! command -v docker &>/dev/null; then
        echo -e "${RED}❌ Docker nicht installiert${NC}" >&2
        return 1
    fi
    if ! docker compose version &>/dev/null 2>&1; then
        echo -e "${RED}❌ docker compose plugin nicht verfügbar${NC}" >&2
        return 1
    fi
    # Test Docker daemon access
    if ! docker ps &>/dev/null 2>&1; then
        echo -e "${RED}❌ Kein Zugriff auf Docker Daemon (Socket-Berechtigung?)${NC}" >&2
        echo -e "${YELLOW}   Fix: sudo usermod -aG docker \$USER && newgrp docker${NC}" >&2
        return 1
    fi
    return 0
}

if ! check_docker; then
    exit 1
fi

# Ensure docker-compose.dev.yml exists
if [[ ! -f docker-compose.dev.yml ]]; then
    echo -e "${RED}❌ docker-compose.dev.yml nicht gefunden!${NC}"
    exit 1
fi

# ════════════════════════════════════════════════════════════════
# Stop any existing dev containers to avoid conflicts
# ════════════════════════════════════════════════════════════════
echo -e "${YELLOW}  Stopping existing dev containers...${NC}"
docker compose -f docker-compose.dev.yml down -v 2>/dev/null || true
docker compose -f docker-compose.dev.yml rm -f 2>/dev/null || true

# ════════════════════════════════════════════════════════════════
# Find free ports for each service
# ════════════════════════════════════════════════════════════════
echo -e "${BLUE}  🔍 Suche freie Ports...${NC}"

POSTGRES_PORT=$(find_free_port 5432)
NATS_PORT=$(find_free_port 4222)
NATS_MON_PORT=$(find_free_port 8222)
MQTT_PORT=$(find_free_port 1883)
MQTT_WS_PORT=$(find_free_port 9001)
REDIS_PORT=$(find_free_port 6379)
API_PORT=$(find_free_port 8080)
ADMIN_UI_PORT=$(find_free_port 8081)

# Export for docker-compose
export POSTGRES_PORT
export NATS_PORT
export NATS_MON_PORT
export MQTT_PORT
export MQTT_WS_PORT
export REDIS_PORT
export API_PORT
export ADMIN_UI_PORT

# Write .env.dev for reference
cat > .env.dev <<EOF
# Auto-generated by dev.sh on $(date)
POSTGRES_PORT=$POSTGRES_PORT
NATS_PORT=$NATS_PORT
NATS_MON_PORT=$NATS_MON_PORT
MQTT_PORT=$MQTT_PORT
MQTT_WS_PORT=$MQTT_WS_PORT
REDIS_PORT=$REDIS_PORT
API_PORT=$API_PORT
ADMIN_UI_PORT=$ADMIN_UI_PORT
DEMO_MODE=$DEMO_MODE
EOF

echo -e "${GREEN}  ✅ Ports reserviert:${NC}"
echo -e "    PostgreSQL:  ${CYAN}$POSTGRES_PORT${NC}"
echo -e "    NATS:        ${CYAN}$NATS_PORT${NC} (Mon: ${CYAN}$NATS_MON_PORT${NC})"
echo -e "    MQTT:        ${CYAN}$MQTT_PORT${NC} (WS: ${CYAN}$MQTT_WS_PORT${NC})"
echo -e "    Redis:       ${CYAN}$REDIS_PORT${NC}"
echo -e "    API:         ${CYAN}$API_PORT${NC}"
echo -e "    Admin UI:    ${CYAN}$ADMIN_UI_PORT${NC}"

# ════════════════════════════════════════════════════════════════
# Start infrastructure services (PostgreSQL, NATS, MQTT, Redis)
# ════════════════════════════════════════════════════════════════
echo -e "${BLUE}  🐳 Starting PostgreSQL, NATS, MQTT, Redis via docker-compose...${NC}"
docker compose -f docker-compose.dev.yml up -d postgres nats mqtt redis

# ════════════════════════════════════════════════════════════════
# Wait for healthchecks — robust loop that does NOT treat "none" as ready
# ════════════════════════════════════════════════════════════════
echo -e "${YELLOW}  ⏳ Waiting for infra services to be healthy...${NC}"

wait_for_health() {
    local container_name=$1
    local display_name=$2
    local timeout_s=60
    local elapsed=0

    echo -n "    $display_name ... "

    while [[ $elapsed -lt $timeout_s ]]; do
        local health
        health=$(docker inspect --format='{{.State.Health.Status}}' "$container_name" 2>/dev/null || echo "none")

        case "$health" in
            healthy)
                echo -e "${GREEN}ready${NC}"
                return 0
                ;;
            unhealthy)
                echo -e "${RED}unhealthy${NC}"
                return 1
                ;;
            starting|none)
                # service may not have a healthcheck yet or is still starting
                sleep 2
                elapsed=$((elapsed + 2))
                ;;
            *)
                sleep 2
                elapsed=$((elapsed + 2))
                ;;
        esac
    done

    echo -e "${YELLOW}timeout (continuing anyway)${NC}"
    return 0
}

# Wait for each infra service - verify both Docker healthcheck AND host port reachability
echo -e "${YELLOW}  ⏳ Waiting for infra services to be reachable...${NC}"

# PostgreSQL
if ! wait_for_port "$POSTGRES_PORT" "PostgreSQL"; then
    echo -e "${RED}❌ PostgreSQL port not reachable${NC}"
    exit 1
fi
# Also verify Docker healthcheck
if ! wait_for_health "agrocore-postgres" "PostgreSQL (healthcheck)"; then
    echo -e "${RED}❌ PostgreSQL failed health check${NC}"
    exit 1
fi

# NATS
if ! wait_for_port "$NATS_PORT" "NATS"; then
    echo -e "${YELLOW}⚠️  NATS port not reachable (may still work)${NC}"
fi
wait_for_health "agrocore-nats" "NATS (healthcheck)" || true

# MQTT
if ! wait_for_port "$MQTT_PORT" "MQTT"; then
    echo -e "${YELLOW}⚠️  MQTT port not reachable (may still work)${NC}"
fi
wait_for_health "agrocore-mqtt" "MQTT (healthcheck)" || true

# Redis
if ! wait_for_port "$REDIS_PORT" "Redis"; then
    echo -e "${YELLOW}⚠️  Redis port not reachable (may still work)${NC}"
fi
wait_for_health "agrocore-redis" "Redis (healthcheck)" || true

# ════════════════════════════════════════════════════════════════
# Start API server
# ════════════════════════════════════════════════════════════════
echo -e "${BLUE}  🐳 Starting API server...${NC}"

# Export dynamic port info into the container via env
# docker-compose.dev.yml already maps ${API_PORT:-8080}:8080
docker compose -f docker-compose.dev.yml up -d api 2>/dev/null || true

# Wait for API port to be reachable from host
if ! wait_for_port "$API_PORT" "API"; then
    echo -e "${RED}❌ API port not reachable${NC}"
    echo -e "${YELLOW}  Showing API logs for debugging:${NC}"
    docker compose -f docker-compose.dev.yml logs api --tail=30
    exit 1
fi
# Also verify Docker healthcheck
if ! wait_for_health "agrocore-api" "API (healthcheck)"; then
    echo -e "${RED}❌ API failed health check${NC}"
    echo -e "${YELLOW}  Showing API logs for debugging:${NC}"
    docker compose -f docker-compose.dev.yml logs api --tail=30
    exit 1
fi

# ════════════════════════════════════════════════════════════════
# Start Notification Service
# ════════════════════════════════════════════════════════════════
echo -e "${BLUE}  🐳 Starting Notification Service...${NC}"
docker compose -f docker-compose.dev.yml up -d notification-service 2>/dev/null || true
wait_for_health "agrocore-notification-service" "Notification Service" || true

# ════════════════════════════════════════════════════════════════
# Start Admin UI
# ════════════════════════════════════════════════════════════════
echo -e "${BLUE}  🐳 Starting Admin UI...${NC}"
docker compose -f docker-compose.dev.yml up -d admin-ui 2>/dev/null || true

# Wait for Admin UI port to be reachable from host
if ! wait_for_port "$ADMIN_UI_PORT" "Admin UI"; then
    echo -e "${RED}❌ Admin UI port not reachable${NC}"
    echo -e "${YELLOW}  Showing Admin UI logs for debugging:${NC}"
    docker compose -f docker-compose.dev.yml logs admin-ui --tail=30
    exit 1
fi
wait_for_health "agrocore-admin-ui" "Admin UI (healthcheck)" || true

# ════════════════════════════════════════════════════════════════
# Demo mode: seed database
# ════════════════════════════════════════════════════════════════
if [[ "$DEMO_MODE" == true ]]; then
    echo -e "${CYAN}  🌱 Seeding demo data...${NC}"
    sleep 3  # Give API a moment to be fully ready

    # Try API-based seeding first (preferred)
    api_url="http://localhost:${API_PORT}"
    if curl -sf -X POST "${api_url}/api/v1/demo/seed" \
        -H "Content-Type: application/json" \
        -d '{"tenant": "demo", "user": "admin"}' 2>/dev/null
    then
        echo -e "${GREEN}    Demo data seeded via API${NC}"
    else
        echo -e "${YELLOW}    API seed failed, trying SQL fallback...${NC}"
        # SQL fallback: run demo_seed.sql directly against the DB container
        if [[ -f migrations/demo_seed.sql ]]; then
            docker exec -i agrocore-postgres psql -U agrocore -d agrocore \
                < migrations/demo_seed.sql 2>/dev/null && \
                echo -e "${GREEN}    Demo data seeded via SQL${NC}" || \
                echo -e "${YELLOW}    SQL seed also failed (maybe already seeded)${NC}"
        else
            echo -e "${YELLOW}    No demo_seed.sql found${NC}"
        fi
    fi
fi

# ════════════════════════════════════════════════════════════════
# Build dashboard binary
# ════════════════════════════════════════════════════════════════
echo -e "${YELLOW}Building dashboard...${NC}"
if ! cargo build -p agrocore-dashboard 2>&1; then
    echo -e "${RED}❌ Dashboard build failed${NC}"
    exit 1
fi

# ════════════════════════════════════════════════════════════════
# Summary
# ════════════════════════════════════════════════════════════════
echo -e "${GREEN}✅ Infrastructure ready${NC}"
echo -e "${CYAN}  Services running on:${NC}"
echo -e "    API:        http://localhost:${API_PORT}"
echo -e "    Admin UI:   http://localhost:${ADMIN_UI_PORT}"
echo -e "    PostgreSQL: localhost:${POSTGRES_PORT}"
echo -e "    NATS:       localhost:${NATS_PORT} (Mon: ${NATS_MON_PORT})"
echo -e "    MQTT:       localhost:${MQTT_PORT} (WS: ${MQTT_WS_PORT})"
echo -e "    Redis:      localhost:${REDIS_PORT}"

# Demo credentials box (only in demo mode)
if [[ "$DEMO_MODE" == true ]]; then
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}                    ${GREEN}🎪 DEMO ZUGANGSDATEN${NC}                      ${CYAN}║${NC}"
    echo -e "${CYAN}╠══════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC} ${YELLOW}Admin UI:${NC}      http://localhost:${ADMIN_UI_PORT}                       ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} ${YELLOW}API Base:${NC}      http://localhost:${API_PORT}                           ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} ${YELLOW}Health Check:${NC}  http://localhost:${API_PORT}/api/v1/health                  ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}                                                                   ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} ${GREEN}Login:${NC}                                                            ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   ${YELLOW}E-Mail:${NC}    admin@demo.local                                         ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   ${YELLOW}Passwort:${NC}  demo123                                                  ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}                                                                   ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC} ${GREEN}Demo-Inhalt:${NC}                                                     ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 1 Tenant (demo) + 1 Admin-User                                 ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 3 Sites (Weizen, Mais, Weide)                                  ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 3 Equipment (Traktor, Mähdrescher, Spritze)                    ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 2 Worker (Hans, Maria)                                         ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 2 Orders (Aussaat, Gülle)                                      ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 3 Inventory Items + Lagerort                                   ${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}   • 3 Tiere (Bella, Lotte, Bruno)                                  ${CYAN}║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
fi

# ════════════════════════════════════════════════════════════════
# Launch dashboard (or skip if --no-dashboard)
# ════════════════════════════════════════════════════════════════
if [[ "$NO_DASHBOARD" == true ]]; then
    echo -e "${YELLOW}  Dashboard skipped (--no-dashboard)${NC}"
    echo -e "${YELLOW}  Press Ctrl+C to stop all services.${NC}"
    # Keep script running so containers don't get killed by trap
    trap 'docker compose -f docker-compose.dev.yml down 2>/dev/null; exit 0' INT TERM
    while true; do sleep 3600; done
else
    echo -e "${BLUE}🎮 Launching Ratatui Dashboard...${NC}"
    echo -e "${YELLOW}  Press 'q' to quit${NC}"
    exec cargo run -p agrocore-dashboard
fi
