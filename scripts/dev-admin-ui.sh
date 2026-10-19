#!/usr/bin/env bash
#
# Startet die AdminUI im lokalen Development-Modus mit Hot-Reload.
#
# Die Docker-Dienste (API, Datenbank, MQTT, NATS) müssen bereits laufen.
# Trunk überwacht alle Dateien und lädt die Änderungen sofort im Browser nach.
#
# Verwendung:
#   ./scripts/dev-admin-ui.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ADMIN_UI_DIR="$PROJECT_ROOT/crates/admin-ui"

echo "🌱 AgroCore AdminUI Development Mode"
echo "======================================"

# Prüfen ob Docker-Dienste laufen
if ! docker ps --format '{{.Names}}' | grep -q 'agrocore-api'; then
    echo "❌ API-Container 'agrocore-api' läuft nicht."
    echo "   Starte zuerst: docker compose up -d"
    exit 1
fi

# Prüfen ob API erreichbar ist
if ! curl -sf http://localhost:3000/api/v1/health >/dev/null 2>&1; then
    echo "⚠️  API auf Port 3000 ist nicht erreichbar."
    echo "   Stelle sicher dass der API-Container läuft und healthy ist."
    echo "   Versuche: docker compose up -d && sleep 10"
    exit 1
fi

# Prüfen ob Trunk installiert ist
if ! command -v trunk &>/dev/null; then
    echo "❌ Trunk ist nicht installiert."
    echo "   Installiere mit: cargo install trunk"
    exit 1
fi

# AdminUI Container stoppen (konfligiert mit Trunk Port 8080)
if docker ps --format '{{.Names}}' | grep -q 'agrocore-admin-ui'; then
    echo "🔧 Stoppe Docker AdminUI-Container (Trunk übernimmt)..."
    docker stop agrocore-admin-ui >/dev/null 2>&1 || true
fi

cd "$ADMIN_UI_DIR"

echo ""
echo "🚀 Starte Trunk Dev-Server..."
echo "   AdminUI:  http://127.0.0.1:8080"
echo "   API:      http://localhost:3000 (via Proxy)"
echo "   Hot-Reload: Aktiv"
echo ""
echo "Drücke Ctrl+C zum Beenden."
echo "======================================"

trunk serve --config Trunk.toml
