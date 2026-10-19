# AgroCore — Startskripte

## Zwei Modi im Überblick

| Skript | Zweck | AdminUI |
|--------|-------|---------|
| `./server.sh` | Kompletter Docker-Stack (Produktion-ähnlich) | Docker-Container |
| `./dev.sh` | Development mit Hot-Reload | Lokal via Trunk |

---

## `./server.sh` — Kompletter Docker-Stack

Startet **alle** Dienste in Docker-Containern: Datenbank, API, AdminUI, MQTT, NATS und Notification-Service.

```bash
./server.sh           # Startet alle Dienste
./server.sh stop      # Stoppt alle Dienste
./server.sh down      # Stoppt und entfernt alle Container
./server.sh status    # Zeigt den Status aller Dienste
```

**Ports:**
- AdminUI: `http://localhost:8081`
- API: `http://localhost:8080`

---

## `./dev.sh` — Development mit Hot-Reload

Startet die **Backend-Dienste** in Docker und die **AdminUI lokal** mit Trunk.
Änderungen an `.rs`, `.js`, `.yml` und `.html` Dateien werden sofort im Browser nachgeladen.

**Wichtig:** Datenbank und alle Daten bleiben erhalten — kein Reset!

### Voraussetzungen

- **Docker** installiert und laufend
- **Trunk** installiert: `cargo install trunk`

### Verwendung

```bash
./dev.sh              # Startet Backend + AdminUI mit Hot-Reload
./dev.sh stop         # Stoppt alle Backend-Dienste
./dev.sh down         # Entfernt alle Container
./dev.sh backend      # Startet nur Backend (ohne AdminUI)
```

Öffne dann **http://localhost:8081** im Browser.

### Wie es funktioniert

1. **Docker** startet die Backend-Dienste: postgres, mqtt, nats, api, notification-service
2. Der Docker-AdminUI-Container wird automatisch gestoppt (Port-Konflikt vermeiden)
3. **Trunk** startet einen lokalen Dev-Server auf Port 8081
4. API-Anfragen werden via Proxy an `localhost:8080` weitergeleitet
5. Bei Dateiänderungen wird die AdminUI automatisch neu kompiliert und der Browser aktualisiert sich

### Zurück zum Docker-Modus

Wenn du wieder die Docker-AdminUI verwenden willst:

```bash
# Trunk beenden (Ctrl+C im Terminal)
# Kompletten Stack mit server.sh starten
./server.sh
```

---

## Konfigurationsdateien

| Datei | Zweck |
|-------|-------|
| `server.sh` | Startet den kompletten Docker-Stack |
| `dev.sh` | Startet Backend + lokale AdminUI mit Hot-Reload |
| `docker-compose.yml` | Definiert alle Docker-Dienste |
| `crates/admin-ui/Trunk.toml` | Trunk-Konfiguration: Port 8081, API-Proxy |

## Tipps

- **Erster Build**: Dauert ~30 Sekunden (vollständige Kompilierung)
- **Folgende Änderungen**: Hot-Reload in 1-5 Sekunden
- **JS-Änderungen** (`maplibre.js`): Sofort ohne Neukompilierung
- **Rust-Änderungen**: Schnelle inkrementelle Kompilierung
