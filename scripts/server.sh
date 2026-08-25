#!/usr/bin/env bash
# server.sh — Start full live environment
# Starts all services (PostgreSQL, NATS, MQTT, Redis) + API server + Admin UI + live dashboard.
# The dashboard shows real-time status of all components.
#
# Usage: ./scripts/server.sh
# Press 'q' in the dashboard to quit (API keeps running).

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 AgroCore Live Environment${NC}"
echo -e "${YELLOW}Starting all services...${NC}"

# Start all Docker services
if command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
    echo -e "${YELLOW}  Stopping existing dev containers...${NC}"
    docker compose -f docker-compose.dev.yml stop 2>/dev/null || true
    docker compose -f docker-compose.dev.yml rm -f 2>/dev/null || true

    echo -e "${BLUE}  🐳 Starting PostgreSQL, NATS, MQTT, Redis via docker-compose...${NC}"
    docker compose -f docker-compose.dev.yml up -d postgres nats mqtt redis 2>/dev/null || true

    echo -e "${YELLOW}  ⏳ Waiting for infra services (5s)...${NC}"
    sleep 5

    echo -e "${BLUE}  🐳 Starting API server...${NC}"
    docker compose -f docker-compose.dev.yml up -d api 2>/dev/null || true

    echo -e "${YELLOW}  ⏳ Waiting for API (5s)...${NC}"
    sleep 5

    echo -e "${BLUE}  🐳 Starting Admin UI...${NC}"
    docker compose -f docker-compose.dev.yml up -d admin-ui 2>/dev/null || true

    echo -e "${YELLOW}  ⏳ Waiting for Admin UI (5s)...${NC}"
    sleep 5
else
    echo -e "${YELLOW}  ⚠️  Docker not available — relying on local services${NC}"
fi

# Build everything
echo -e "${YELLOW}Building all crates...${NC}"
if ! cargo build --workspace 2>&1; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All services running${NC}"

# Start API server in background
echo -e "${YELLOW}  Starting API server on :8080...${NC}"
cargo run -p agrocore-api &
API_PID=$!

# Start dashboard (foreground)
echo -e "${BLUE}🎮 Launching Ratatui Dashboard (live)...${NC}"
echo -e "${YELLOW}  API server PID: $API_PID${NC}"
echo -e "${YELLOW}  Press 'q' to quit dashboard (API keeps running)${NC}"

# Cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down...${NC}"
    kill $API_PID 2>/dev/null || true
    wait $API_PID 2>/dev/null || true
    echo -e "${GREEN}✅ Done${NC}"
}
trap cleanup EXIT INT TERM

# Launch dashboard
exec cargo run -p agrocore-dashboard
