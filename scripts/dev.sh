#!/usr/bin/env bash
# dev.sh — Start development environment with live dashboard
# Starts PostgreSQL + NATS, then launches the Ratatui dashboard.
# The dashboard polls services every 2s and shows build/git/system status.
#
# Usage: ./scripts/dev.sh
# Press 'q' in the dashboard to quit (services keep running).

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 AgroCore Dev Environment${NC}"
echo -e "${YELLOW}Starting infrastructure...${NC}"

# Check if docker compose is available
if command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
    echo "  🐳 Starting PostgreSQL + NATS via docker-compose..."
    docker compose -f docker-compose.dev.yml up -d postgres nats 2>/dev/null || true
else
    echo -e "${YELLOW}  ⚠️  Docker not available — relying on local services${NC}"
fi

# Build dashboard binary (non-blocking if already built)
echo -e "${YELLOW}Building dashboard...${NC}"
if ! cargo build -p agrocore-dashboard 2>&1; then
    echo -e "${RED}❌ Dashboard build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Infrastructure ready${NC}"
echo -e "${BLUE}🎮 Launching Ratatui Dashboard...${NC}"
echo -e "${YELLOW}  Press 'q' to quit${NC}"

# Launch the dashboard
exec cargo run -p agrocore-dashboard
