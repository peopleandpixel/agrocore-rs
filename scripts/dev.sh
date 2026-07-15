#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

API_PORT="${API_PORT:-3000}"
WEATHER_PORT="${WEATHER_PORT:-3001}"
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

cleanup() {
    local exit_code=$?

    trap - EXIT INT TERM

    for pid in "${service_pids[@]:-}"; do
        kill "$pid" >/dev/null 2>&1 || true
    done

    for pid in "${service_pids[@]:-}"; do
        wait "$pid" >/dev/null 2>&1 || true
    done

    docker stop "$POSTGRES_CONTAINER" "$NATS_CONTAINER" >/dev/null 2>&1 || true

    exit "$exit_code"
}

wait_for_port() {
    local host="$1"
    local port="$2"
    local label="$3"

    for _ in $(seq 1 30); do
        if (echo >"/dev/tcp/${host}/${port}") >/dev/null 2>&1; then
            return 0
        fi

        sleep 1
    done

    echo "Timed out waiting for ${label} on ${host}:${port}" >&2
    exit 1
}

start_service() {
    local label="$1"
    local workdir="$2"
    shift 2

    echo "Starting ${label}..."
    (
        cd "$workdir"
        exec "$@"
    ) &
    service_pids+=("$!")
}

trap cleanup EXIT INT TERM

docker rm -f "$POSTGRES_CONTAINER" "$NATS_CONTAINER" >/dev/null 2>&1 || true

echo "Starting PostgreSQL (PostGIS)..."
docker run -d --rm \
    --name "$POSTGRES_CONTAINER" \
    -p "${POSTGRES_PORT}:5432" \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB="$DATABASE_NAME" \
    postgis/postgis:latest >/dev/null

echo "Starting NATS..."
docker run -d --rm \
    --name "$NATS_CONTAINER" \
    -p "${NATS_PORT}:4222" \
    -p "${NATS_MONITOR_PORT}:8222" \
    nats:latest >/dev/null

wait_for_port 127.0.0.1 "$POSTGRES_PORT" "PostgreSQL"
wait_for_port 127.0.0.1 "$NATS_PORT" "NATS"

start_service \
    "API" \
    "$ROOT_DIR" \
    env \
        DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
        LISTEN_ADDR="0.0.0.0:${API_PORT}" \
        NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
        JWT_SECRET="$JWT_SECRET" \
        cargo run -p agrocore-api

start_service \
    "Reporting Service" \
    "$ROOT_DIR" \
    env \
        DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
        NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
        LISTEN_ADDR="0.0.0.0:${REPORTING_PORT}" \
        cargo run -p agrocore-reporting-service

start_service \
    "Weather Service" \
    "$ROOT_DIR" \
    env \
        DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
        NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
        LISTEN_ADDR="0.0.0.0:${WEATHER_PORT}" \
        cargo run -p agrocore-weather-service

start_service \
    "Geometry Service" \
    "$ROOT_DIR" \
    env \
        DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
        NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
        LISTEN_ADDR="0.0.0.0:${GEOMETRY_PORT}" \
        cargo run -p agrocore-geometry-service

start_service \
    "Asset Registry" \
    "$ROOT_DIR" \
    env \
        DATABASE_URL="postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT}/${DATABASE_NAME}" \
        NATS_URL="nats://127.0.0.1:${NATS_PORT}" \
        LISTEN_ADDR="0.0.0.0:${ASSET_REGISTRY_PORT}" \
        cargo run -p agrocore-asset-registry

start_service \
    "Admin UI" \
    "$ROOT_DIR/crates/admin-ui" \
    env \
        AGROCORE_API_BASE_URL="http://127.0.0.1:${API_PORT}" \
        trunk serve --port "$ADMIN_UI_PORT"

echo
echo "Stack is up."
echo "API:      http://localhost:${API_PORT}"
echo "Admin UI: http://localhost:${ADMIN_UI_PORT}"
echo "Stop it with Ctrl-C."

wait -n "${service_pids[@]}"
exit $?
