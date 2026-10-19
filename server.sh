#!/usr/bin/env bash
#
# Startet den kompletten AgroCore Docker-Stack.
# Alle Dienste (Datenbank, API, AdminUI, MQTT, NATS, Notification)
# werden in Containern gestartet.
#
# Verwendung:
#   ./server.sh              # Startet alle Dienste
#   ./server.sh stop         # Stoppt alle Dienste
#   ./server.sh down         # Stoppt und entfernt alle Container
#   ./server.sh status       # Zeigt den Status aller Dienste
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

ACTION="${1:-up}"

case "$ACTION" in
    up)
        echo "🌱 AgroCore — Kompletter Docker-Stack starten"
        echo "=============================================="
        docker compose up -d --build
        echo ""
        echo "✅ Alle Dienste gestartet."
        echo "   AdminUI:  http://localhost:8081"
        echo "   API:      http://localhost:8080"
        echo ""
        docker compose ps
        ;;
    stop)
        echo "🛑 AgroCore — Alle Dienste stoppen"
        docker compose stop
        echo "✅ Alle Dienste gestoppt."
        ;;
    down)
        echo "🗑️  AgroCore — Alle Container entfernen"
        docker compose down
        echo "✅ Alle Container entfernt."
        ;;
    status)
        docker compose ps
        ;;
    *)
        echo "Verwendung: $0 [up|stop|down|status]"
        exit 1
        ;;
esac
