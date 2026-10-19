#!/usr/bin/env bash
#
# Startet AgroCore im Development-Modus mit Hot-Reload.
#
# Backend-Dienste (API, Datenbank, MQTT, NATS, Notification) laufen in Docker.
# Die AdminUI wird lokal mit Trunk gestartet — Änderungen an .rs, .js, .yml
# und .html Dateien werden sofort im Browser nachgeladen.
#
# Datenbank und alle Daten bleiben erhalten — kein Reset!
#
# Voraussetzungen:
#   - Docker installiert und laufend
#   - Trunk installiert: cargo install trunk
#
# Verwendung:
#   ./dev.sh              # Startet Development-Modus
#   ./dev.sh stop         # Stoppt alle Dienste
#   ./dev.sh down         # Stoppt und entfernt alle Container
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

ADMIN_UI_DIR="$SCRIPT_DIR/crates/admin-ui"
ACTION="${1:-up}"

# ---------------------------------------------------------------
# Health-Check: wartet bis ein Container healthy ist
# ---------------------------------------------------------------
wait_for_healthy() {
    local container_name="$1"
    local timeout="${2:-60}"
    local elapsed=0

    echo "⏳ Warte auf '$container_name'..."
    while [ $elapsed -lt $timeout ]; do
        local status
        status=$(docker inspect --format='{{.State.Health.Status}}' "$container_name" 2>/dev/null || echo "not_found")
        if [ "$status" = "healthy" ]; then
            echo "   ✅ '$container_name' ist healthy."
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    echo "   ❌ Timeout — '$container_name' ist nach ${timeout}s nicht healthy."
    return 1
}

# ---------------------------------------------------------------
# Backend-Dienste starten (ohne AdminUI)
# ---------------------------------------------------------------
start_backend() {
    echo "🌱 AgroCore — Development-Modus starten"
    echo "=============================================="

    # AdminUI Container stoppen, falls vorhanden
    if docker ps --format '{{.Names}}' | grep -q 'agrocore-admin-ui'; then
        echo "🔧 Stoppe Docker AdminUI-Container (Trunk übernimmt)..."
        docker stop agrocore-admin-ui >/dev/null 2>&1 || true
    fi

    # Backend-Dienste starten (ohne admin-ui)
    echo "🐳 Starte Backend-Dienste..."
    docker compose up -d --build postgres mqtt nats api notification-service

    # Auf API warten
    wait_for_healthy agrocore-api 90

    echo ""
    echo "✅ Backend-Dienste sind bereit."
    echo "   API: http://localhost:8080"
}

# ---------------------------------------------------------------
# Trunk Dev-Server starten
# ---------------------------------------------------------------
start_trunk() {
    # Prüfen ob API erreichbar ist
    local retries=0
    local max_retries=15
    while ! curl -sf http://localhost:8080/api/v1/health >/dev/null 2>&1; do
        retries=$((retries + 1))
        if [ $retries -ge $max_retries ]; then
            echo "❌ API auf Port 8080 ist nicht erreichbar."
            echo "   Stelle sicher dass der API-Container läuft und healthy ist."
            exit 1
        fi
        sleep 2
    done

    # Prüfen ob Trunk installiert ist
    if ! command -v trunk &>/dev/null; then
        echo "❌ Trunk ist nicht installiert."
        echo "   Installiere mit: cargo install trunk"
        exit 1
    fi

    cd "$ADMIN_UI_DIR"

    echo ""
    echo "🚀 Starte Trunk Dev-Server..."
    echo "   AdminUI:    http://localhost:8081"
    echo "   API:        http://localhost:8080 (via Proxy)"
    echo "   Hot-Reload: Aktiv"
    echo ""
    echo "Drücke Ctrl+C zum Beenden."
    echo "=============================================="

    trunk serve --config Trunk.toml
}

# ---------------------------------------------------------------
# Hauptlogik
# ---------------------------------------------------------------
case "$ACTION" in
    up)
        start_backend
        start_trunk
        ;;
    stop)
        echo "🛑 AgroCore — Development-Modus stoppen"
        docker compose stop postgres mqtt nats api notification-service
        echo "✅ Backend-Dienste gestoppt."
        ;;
    down)
        echo "🗑️  AgroCore — Alle Container entfernen"
        docker compose down
        echo "✅ Alle Container entfernt."
        ;;
    backend)
        # Nur Backend starten, ohne Trunk (für Debugging)
        start_backend
        echo ""
        echo "💡 Um die AdminUI mit Hot-Reload zu starten:"
        echo "   cd crates/admin-ui && trunk serve --config Trunk.toml"
        ;;
    *)
        echo "Verwendung: $0 [up|stop|down|backend]"
        echo ""
        echo "  up       — Startet Backend + AdminUI mit Hot-Reload (Standard)"
        echo "  stop     — Stoppt alle Backend-Dienste"
        echo "  down     — Entfernt alle Container"
        echo "  backend  — Startet nur Backend-Dienste (ohne AdminUI)"
        exit 1
        ;;
esac
