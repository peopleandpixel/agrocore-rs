# AgroCore-RS — Technische Dokumentation

> Version: 0.8.3
> Letzte Aktualisierung: 2026-08-11
> Lizenz: GPL-3.0-or-later

---

## 1. Projektübersicht

**AgroCore-RS** ist eine landwirtschaftliche Betriebsleitungsplattform, die von Grund auf in Rust entwickelt wurde. Sie vereint Feld- und Flächenmanagement, Auftrags- und Taskplanung, Wetter- und Phänologiedaten, Tierhaltung, Finanzen, Compliance, Maschinenmanagement, IoT-Integration sowie Multi-Country-LPIS-Unterstützung in einem System.

### 1.1 Stack

| Ebene              | Technologie                        |
|--------------------|------------------------------------|
| Sprache            | Rust 2024 Edition                   |
| HTTP-API           | Actix Web 4                         |
| Frontend           | Leptos 0.8 (SPA, WASM-basiert)     |
| Datenbank          | PostgreSQL 16 + PostGIS 3.4         |
| Messaging          | NATS 2.10 (JetStream) + MQTT 5     |
| IoT-Protokoll      | MQTT via rumqttc 0.25              |
| Container          | Docker + Docker Compose            |
| CI/CD              | GitHub Actions + GitLab CI         |
| OpenAPI            | utoipa 5 + Swagger UI 8            |
| Authentifizierung  | JWT (jsonwebtoken 10) + argon2 0.6 |
| Rate Limiting      | actix-governor                     |
| Caching            | moka (Memory) + Redis              |
| Geo-Bibliotheken   | geo 0.33, geojson 1.0, geozero 0.15 |
| Export             | rust_xlsxwriter 0.95               |

### 1.2 Design-Ziele

- **Lokal-first**: Standardmäßig läuft alles lokal; Cloud ist eine Option, nicht eine Voraussetzung.
- **Multi-Tenant**: Jeder Tenant ist durch PostgreSQL Row-Level Security (RLS) isoliert.
- **Domain-Driven Design**: Klare Trennung von Domain-Logik, Infrastruktur und API.
- **Real-time**: NATS-Event-Bus für lose Kopplung und Webhooks für externe Integration.
- **Erweiterbar**: Plugin-Architektur für LPIS-Provider (SIGPAC, iLPIS, RPG, SIAN, BRP, ...).

---

## 2. Architektur

### 2.1 Hochlevel-Architektur

```
┌─────────────────────────────────────────────────────────┐
│                      Admin UI (Leptos)                   │
│                      WASM / SPA                         │
└──────────────┬──────────────────────────┬───────────────┘
               │ HTTP API                 │ WebSocket/MQTT
               ▼                          ▼
┌──────────────────────┐       ┌──────────────────────┐
│   API (Actix Web)    │◄─────►│   NATS Event Bus     │
│                      │       │   + JetStream        │
└────────┬─────────────┘       └────────┬─────────────┘
         │                              │
         │ JWT Auth                     │ Events
         ▼                              ▼
┌──────────────────────┐       ┌──────────────────────┐
│  Domain Layer        │       │ MQTT Bridge          │
│  (Entities,         │       │ (NATS ↔ MQTT)        │
│   Repositories,      │       │                      │
│   Services)          │       │  HA Auto-Discovery   │
└────────┬─────────────┘       └──────────────────────┘
         │
         ▼
┌──────────────────────┐
│  Infrastructure      │
│  (PostgreSQL /       │
│   PostGIS,            │
│   Repositories)      │
└──────────────────────┘
```

### 2.2 Workspace-Struktur

```
agrocore-rs/
├── Cargo.toml                  # Workspace-Manifest (Rust 2024)
├── crates/
│   ├── api/                    # Actix Web HTTP API + OpenAPI + Swagger UI
│   │   ├── src/
│   │   │   ├── handlers/       # HTTP-Handler (27 Module)
│   │   │   ├── dto/            # API-DTOs
│   │   │   ├── services/       # ImportService
│   │   │   ├── middleware.rs   # AuthExtractor, CORS, Security Headers
│   │   │   └── lib.rs          # AppState, OpenApi, Server-Bootstrap
│   │   └── tests/              # Integration-Tests
│   ├── admin-ui/               # Leptos 0.8 SPA (WASM)
│   ├── domain/                 # Kern-Geschäftslogik, Entitäten, Traits
│   │   ├── src/
│   │   │   ├── entities/       # 17 Domänen-Module
│   │   │   ├── repositories.rs # Repository-Traits
│   │   │   └── services/       # WorkflowService, CalculationService, NutritionService
│   ├── infrastructure/         # PostgreSQL-Repository-Implementierungen
│   ├── messaging/              # NATS + MQTT Client, Event-Definitionen
│   ├── shared/                 # Gemeinsame Typen (Pagination, Auth, LPIS)
│   ├── lpis-providers/         # Multi-Country LPIS Provider (8 Länder)
│   ├── reporting-service/      # Excel/GeoJSON/PAC-SIP Export-Worker
│   ├── weather-service/        # Wetterdaten-Ingestion-Worker
│   ├── geometry-service/       # Geodaten-Verarbeitung
│   └── asset-registry/         # Asset-Management
├── migrations/                 # 18 SQL-Migrationsdateien
├── docker-compose.yml          # Produktions-Compose (Postgres, NATS, Mosquitto, API, Admin UI)
├── scripts/
│   ├── dev.sh                  # Vollständiger Dev-Stack (Docker-basiert)
│   ├── dev-start.sh            # Leichter Dev-Start (API + Admin UI)
│   └── sigpac_import/          # SIGPAC Bulk-Import (Python)
├── infra/                      # Kubernetes, Prometheus, Loki, Promtail
└── docs/                       # Dokumentation
```

### 2.3 Domain-Driven Design: Entity + Repository Pattern

Jede Domäne folgt einem konsistenten Mustern:

```rust
// 1. Entity (domain/src/entities/{entity}.rs)
pub struct Site {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub label: String,
    // ... weitere Felder
}

// 2. DTOs für Create/Update
pub struct CreateSiteDto { pub label: String, /* ... */ }
pub struct UpdateSiteDto { pub label: Option<String>, /* ... */ }

// 3. Repository-Trait (domain/src/repositories.rs)
pub trait SiteRepository: Send + Sync {
    async fn find_all_visible(&self, tenant_id: TenantId, pagination: Pagination,
        user_id: Uuid, roles: &[UserRole]) -> Result<PaginatedResponse<Site>>;
    async fn find_by_id_visible(&self, tenant_id: TenantId, id: Uuid,
        user_id: Uuid, roles: &[UserRole]) -> Result<Option<Site>>;
    async fn create(&self, tenant_id: TenantId, dto: CreateSiteDto,
        created_by: Uuid) -> Result<Site>;
    async fn update(&self, tenant_id: TenantId, id: Uuid, dto: UpdateSiteDto,
        updated_by: Uuid) -> Result<Option<Site>>;
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<bool>;
}

// 4. PostgreSQL-Implementierung (infrastructure/src/postgres/site.rs)
pub struct PgSiteRepo { pool: PgPool }
impl SiteRepository for PgSiteRepo { ... }

// 5. Factory-Methode (infrastructure/src/postgres/database.rs)
pub fn site_repo(&self) -> Arc<dyn SiteRepository> { ... }
```

### 2.4 Tenant-Isolation (RLS)

Alle Tabellen enthalten eine `tenant_id UUID`-Spalte mit `ON DELETE CASCADE`.

```sql
-- RLS Policy Beispiel
CREATE POLICY sites_tenant_isolation ON sites
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Anwendungsseitig auf jeder Anfrage:
SET LOCAL app.current_tenant_id = $1;
SET LOCAL app.is_superadmin = $1;
```

Ein `TenantId`-Wrapper-Typ (`pub struct TenantId(pub Uuid)`) in der Domain-Schicht stellt compile-time-Sicherheit gegen UUID/TenantId-Mischungen sicher.

### 2.5 PaginatedResponse

Einheitliches Antwortformat für alle paginierbaren Endpunkte:

| Feld         | Typ    | Beschreibung                |
|--------------|--------|-----------------------------|
| data         | Vec<T> | Ergebnis-Elemente           |
| total        | u64    | Gesamtanzahl der Datensätze |
| page         | u64    | Aktuelle Seite              |
| per_page     | u64    | Elemente pro Seite          |
| total_pages  | u64    | Gesamtzahl der Seiten       |

---

## 3. Domänenmodule

AgroCore-RS umfasst 17 integrierte Domänenmodule:

| # | Modul                | Kernfunktionalität                                        | Status      |
|---|----------------------|-----------------------------------------------------------|-------------|
| 1 | **Sites/Parcels**    | Flächen, Boundary, GeoPoint, Plots, SigpacData, Custom Fields | Produktiv  |
| 2 | **Orders/Tasks**     | Arbeitsaufträge, Rekurrenz, Execution Policies, Automation | Produktiv  |
| 3 | **Workforce**        | Worker, Work Logs, Standort-Reporting, Task-Status        | Produktiv  |
| 4 | **Equipment**        | Maschinen, Service-Intervalle, Kosten                     | Produktiv  |
| 5 | **Weather/Phenology**| Wetterstationen, Wetterdaten, Phänologische Phase (BBCH)  | Produktiv  |
| 6 | **Compliance**       | Checklisten, Audit-Trail, Pflanzenschutz, Düngemittel     | Produktiv  |
| 7 | **Vineyard**         | Weinberge, Kelterrechnung, Qualitätsklassen               | Produktiv  |
| 8 | **Olive**            | Olivenhaine, Olivenöl-Rekorde, Qualitätsklassen          | Produktiv  |
| 9 | **Water**            | Wasserquellen, Nutzung, Quotas, Bewässerungsmethoden      | Produktiv  |
|10 | **Harvest**          | Ernte-Saisons, Lose, Ablieferung, Kaltluft-Tracking       | Produktiv  |
|11 | **Livestock**        | Tiere, Arten, Behandlungen, Weide, Status                 | Produktiv  |
|12 | **Finance**          | PAC-Anträge, Kostenstellen, Finanzbuchungen               | Produktiv  |
|13 | **Nutrition**        | Düngungsplanung, Nährstoffbilanz                          | Produktiv  |
|14 | **Import Service**   | GeoJSON, Shapefile, REGEPAC Import mit LPIS-Validierung   | Produktiv  |
|15 | **LPIS Providers**   | Multi-Country LPIS (8 Länder) mit Caching + Rate Limiting | Produktiv  |
|16 | **IoT/Devices**      | Geräteregistrierung, Telemetrie, Commands, HA-Discovery   | Produktiv  |
|17 | **System**           | Health-Status, Initial-Setup, Tenant-Management           | Produktiv  |

### 3.1 LPIS Multi-Country Provider

Unterstützte Länder mit jeweiligen Providern:

| Land | Code | Provider | Datenquelle          |
|------|------|----------|----------------------|
| Spanien | ES | SIGPAC | Sistema de Información Geográfica de Parcelas Agrícolas |
| Portugal | PT | iLPIS | Integrated Land Parcel Identification System |
| Frankreich | FR | RPG | Registre Parcellaire Graphique |
| Italien | IT | SIAN | Sistema Informativo Agricolo Nazionale |
| Niederlande | NL | BRP | Basisregistratie Percelen (PDOK WFS) |
| Deutschland | DE | LPIS | Länder-spezifische LPIS-Portale |
| Polen | PL | LPIS | ARiMR Geoportal WFS |
| Österreich | AT | INVEKOS | AMA/data.gv.at WFS |

#### Provider-Architektur

- **`BaseClient`** (`lpis-providers/src/base.rs`): Gemeinsame HTTP-Client-Logik mit:
  - Konfigurierbares Timeout
  - Caching via `LpisCache` (Memory-Moka / Redis-Backend)
  - Rate Limiting via `governor`
  - Retry-Logik mit exponentiellem Backoff (ohne externes `retry`-Crate)
  - Konfigurierbar über `ProviderConfig`
- **`LpisRegistry`**: Zentrale Registry für alle Provider
- **`LpisCache`**: Einheitliche Caching-Schicht mit TTL und Max-Entries

### 3.2 Event-System (NATS)

Der NATS-Event-Bus verbindet alle Services:

```
events.sites       → SiteCreated, SiteUpdated, SiteDeleted
events.orders      → OrderCreated, OrderUpdated, OrderDeleted
events.users       → UserCreated, UserUpdated, UserDeleted
events.spatial     → SpatialPolygonEntered, SpatialPolygonIn, SpatialPolygonExited
events.iot.*       → IoTTelemetry, IoTDeviceStatus, IoTCommand
events.harvest.*   → HarvestSeasonCreated, LotDelivered, ColdChainAlert
```

NATS JetStream wird für persistente Nachrichten verwendet.

---

## 4. Datenbank-Schema

### 4.1 Migrationen

18 Migration-Dateien in `/migrations/`:

| Datei | Beschreibung |
|-------|-------------|
| `20240101_init.sql` | Kern-Schema (Tenants, Users, Sites, Equipment, Orders, Weather, Animals, Tasks) |
| `2024071501_harvest.sql` | Ernte-Logistik (Seasons, Lots, Deliveries, Cold Chain) |
| `2024071502_optimization.sql` | Junction Tables, Triggers, Indizes |
| `2026071601_site_schema_update.sql` | Erweiterte Site-Felder |
| `2026071701_rename_sites_polygon_to_boundary.sql` | Polygon → Boundary Rename |
| `2026071702_fix_enum_types.sql` | Enum-Typen-Korrektur |
| `2026071801_applicator_licenses.sql` | Pflanzenschutz-Ausbringungs-Lizenzen |
| `2026072401_olive_tables.sql` | Olivenhaine + Olivenöl-Rekorde |
| `2026072402_vineyard_tables.sql` | Weinberge + Kelterrechnung |
| `2026072403_water_tables.sql` | Wasserquellen, Nutzung, Quotas |
| `2026072404_fertilizer_compliance_tables.sql` | Düngemittel-Compliance |
| `2026072405_livestock_tables.sql` | Tierhaltung (Tiere, Behandlungen, Weide) |
| `2026072407_task_data_update.sql` | Task-Daten-Schema-Update |
| `2026072501_core_additional_tables.sql` | Audit, Cost Centers, Financial, PAC |
| `2026072502_plant_protection_worker_status.sql` | Pflanzenschutz-Worker-Status |
| `2026072503_tasks_table.sql` | Tasks-Tabelle |
| `2026072601_rls_policies.sql` | Kern-RLS-Policies |
| `2026072602_rls_policies_extended.sql` | Erweiterte RLS-Policies |
| `2026072801_lpis_sigpac_reference.sql` | SIGPAC-Referenzdaten + Validierungsfunktionen |
| `2026073101_lpis_multi_country.sql` | Multi-Country LPIS (lpis_country, lpis_data JSONB) |
| `2026080901_iot_devices.sql` | IoT-Geräteregistrierung |
| `2026080902_task_data_complete.sql` | Vollständige Task-Daten |
| `2026080903_vineyard_active.sql` | Vineyard-Aktiv-Flag |

Migrations werden über `sqlx::migrate!("./migrations")` beim Server-Start automatisch angewendet.

### 4.2 Schlüsseltabellen

#### tenants
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug VARCHAR(50) UNIQUE,
    config JSONB,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at/updated_at TIMESTAMPTZ
);
```

#### users
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,       -- argon2
    roles JSONB NOT NULL DEFAULT '[]',
    refresh_token TEXT,
    refresh_token_expires_at TIMESTAMPTZ,
    ... weitere Felder (Kosten, Sprache, Farbe, ...)
);
```

#### sites
```sql
CREATE TABLE sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL,
    label TEXT NOT NULL,
    site_type VARCHAR(30),
    crop_type VARCHAR(30),
    area NUMERIC(15,2),                 -- Hektar
    polygon GEOMETRY(POLYGON, 4326),   -- PostGIS
    center GEOMETRY(POINT, 4326),
    lpis_country VARCHAR(2),           -- LPIS-Country-Code
    lpis_data JSONB,                   -- LPIS-spezifische Daten
    ... weitere Felder
);
```

#### iot_devices
```sql
CREATE TABLE iot_devices (
    device_id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    device JSONB NOT NULL,              -- Vollständiges Gerät-Objekt
    created_at/updated_at TIMESTAMPTZ
);
```

#### sigpac_parcels
```sql
CREATE TABLE sigpac_parcels (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL,
    province/smallint, municipality/smallint,
    aggregate/smallint, zone/smallint,
    polygon/smallint, parcel/smallint, enclosure/smallint,
    sigpac_reference VARCHAR(20) GENERATED ALWAYS AS (...) STORED,
    usage_code/usage_description, geometry GEOMETRY(POLYGON, 4326),
    official_area_ha NUMERIC(10,4),
    source_year/smallint, ...
);
```

### 4.3 Indizes

- GIST-Indizes auf allen PostGIS-geometrischen Spalten
- Hash-Indizes auf `tenant_id` in allen Tabellen
- GIN-Indizes auf JSONB-Spalten (`roles`, `lpis_data`, `device`)
- Unique-Indizes auf `sigpac_reference`, `email`, `device_id`

---

## 5. Authentifizierung & Autorisierung

### 5.1 JWT-Auth

- **Signing**: `jsonwebtoken` (RS256/HS256)
- **Expiry**: 30 Minuten für Access-Token, 7 Tage für Refresh-Token
- **Password Hashing**: argon2 (RFC 9106)
- **Middleware**: `AuthExtractor` in `middleware.rs` extrahiert und validiert den Bearer-Token

### 5.2 Rollenbasierte Zugriffskontrolle (RBAC)

| Rolle       | Berechtigungen                                          |
|-------------|---------------------------------------------------------|
| admin       | Vollzugriff auf alle Ressourcen, Tenant-Management      |
| manager     | CRUD auf Sites, Orders, Workers, Equipment, Compliance  |
| worker      | Nur eigene Tasks, Work-Logs, Standort-Reporting         |
| viewer      | Nur Lesen (Sites, Orders, Workers, Reports)             |

Jede API-Route verwendet `auth.require_manager()`, `auth.require_admin()`, `auth.require_any_role(...)` etc.

### 5.3 Sicherheitsheader

```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```

### 5.4 Rate Limiting

- **Standard**: 120 Requests/Minute pro IP (via `actix-governor`)
- **Burst**: 120
- **Für Auth-Endpunkte**: Strengere Limits empfohlen (siehe `docs/optimizations.md` §2.3)

---

## 6. IoT-Integration (MQTT + Home Assistant)

### 6.1 MQTT Client

Implementiert in `crates/messaging/src/lib.rs` mit `rumqttc 0.25`:

- **Konfiguration**: `MqttConfig` (broker_host, broker_port, client_id, TLS, credentials)
- **Event-Typen**:
  - `IoTTelemetryEvent` — Messwerte (Temperatur, Humidity, SoilMoisture, ...)
  - `IoTDeviceStatusEvent` — Gerätestatus (Online/Offline/Maintenance)
  - `IoTCommandEvent` — Befehle an Geräte
  - `IoTMeasurement` — Messwerte mit Timestamp und Wert

### 6.2 UnifiedMessagingClient

Ermöglicht dualen Betrieb (NATS + MQTT):
- `publish_event()` veröffentlicht gleichzeitig auf beiden Backends
- `publish_telemetry()` und `publish_device_status()` sind MQTT-optimiert (retained)
- `subscribe_device_commands()` für bidirektionale Kommunikation

### 6.3 Home Assistant Auto-Discovery

Vollständige Implementierung:
- `HaSensorConfig` für alle Capabilities (Temperature, Humidity, SoilMoisture, Light, GPS, BatteryLevel, SignalStrength)
- `HaBinarySensorConfig` für Geräte-Verfügbarkeit (connectivity)
- `HaButtonConfig` für Aktuator-Befehle
- `HaNumberConfig` für numerische Einstellungen
- `HaDeviceInfo` mit Identifiers, Manufacturer, Model, Version, via_device
- Topics: `homeassistant/sensor/.../config`, `homeassistant/binary_sensor/.../config`, `agrocore/telemetry/...`, `agrocore/status/...`, `agrocore/commands/...`

### 6.4 Mosquitto-Broker

In `docker-compose.yml`:
- `eclipse-mosquitto:2.0` auf Port 1883 (MQTT), 9001 (WebSockets)
- TLS-Zertifikate generiert (CA, Server, Client, PKCS12)
- Authentifizierung via Passwort-Datei
- Health-Checks via `mosquitto_sub`

---

## 7. Multi-Country LPIS Integration

### 7.1 Provider-Architektur

```
LpisRegistry (crates/shared/src/lpis.rs)
  └── LpisProvider<T> (trait)
       ├── SigpacProvider (ES)  ← BaseClient + Caching + Rate Limiting
       ├── IlpisProvider (PT)
       ├── RpgProvider (FR)
       ├── SianProvider (IT)
       ├── BrpProvider (NL)  ← bereits migriert auf BaseClient
       ├── LpisDeProvider (DE)
       ├── LpisPlProvider (PL)
       └── InvekosProvider (AT)
```

### 7.2 BaseClient Pattern

Jeder Provider nutzt `BaseClient` aus `lpis-providers/src/base.rs`:
- HTTP-Client mit konfigurierbarem Timeout
- Caching via `LpisCache` (Memory-Moka / Redis)
- Rate Limiting via `governor` (Requests/Sekunde + Burst)
- Retry-Logik mit exponentiellem Backoff
- Konfiguration über `ProviderConfig` (base_url, timeout, cache_ttl, rate_limits, enabled)

### 7.3 LPIS Provider Settings API

```
GET  /api/v1/settings/lpis          — Laden der Provider-Konfiguration
PUT  /api/v1/settings/lpis         — Speichern (mit restart_required-Flag)
GET  /api/v1/settings/lpis/providers — Liste aller Provider mit Defaults
```

### 7.4 Import Service LPIS-Integration

`ImportService` verwendet `LpisRegistry` für Multi-Country-Validierung:
- Alle Import-DTOs (`GeoJsonImportRequest`, `ShapefileImportRequest`, `ImportSitesRequest`) unterstützen `lpis_country: Option<LpisCountry>`
- Rückwärtskompatibel: Standard ist ES (SIGPAC) wenn nicht angegeben
- Fallback zu SIGPAC-SQL-Funktion wenn Provider nicht verfügbar

### 7.5 SIGPAC-Import

Python-Skripte in `scripts/sigpac_import/`:
- `import_sigpac.py` — Hauptimport (15 spanische Regionen, ~25M Parcelen)
- `discover_sources.py` — Quell-URL-Entdeckung
- `test_setup.py` — Umwelt-Verifikation

---

## 8. Reporting & Export

### 8.1 Export-Worker

`crates/reporting-service/` läuft als separater Worker:
- **Excel-Export**: `rust_xlsxwriter` fürOrders, Sites, Tasks
- **GeoJSON-Export**: Flächen- und Boundary-Daten
- **PAC-SIP-Export**: Spezifischer Export für EU-Agrarförderung (PlanTEIL-Importformat)

### 8.2 Export-Endpunkte

```
POST /api/v1/reports/orders-excel
POST /api/v1/reports/sites-geojson
POST /api/v1/reports/pac-sip
```

---

## 9. Entwicklung & Deployment

### 9.1 Lokale Entwicklung

```bash
# Vollständiger Stack (Docker-Infrastruktur)
./scripts/dev.sh

# Leichter Start (API + Admin UI, kein Docker)
./scripts/dev-start.sh

# Direkter API-Start
cargo run -p agrocore-api

# Admin UI lokal
cd crates/admin-ui && trunk serve
```

### 9.2 Docker Compose (Produktiv)

```yaml
services:
  postgres:    # postgis/postgis:16-3.4
  nats:        # nats:2.10-alpine (mit JetStream)
  mosquitto:   # eclipse-mosquitto:2.0 (MQTT + WebSockets + TLS)
  api:         # Actix Web API (Port 8080)
  reporting-service:
  weather-service:
  admin-ui:    # Leptos SPA (Port 3000)
```

### 9.3 Kubernetes

Helm-Chart-Anpassungen in `infra/k8s/`:
- `api-deployment.yaml`
- `worker-deployments.yaml`
- `ingress.yaml`

### 9.4 Monitoring

- **Prometheus**: Metriken-Endpunkt auf `/metrics`
- **Grafana**: Vordefinierte Dashboards (siehe `infra/prometheus/`)
- **Loki/Promtail**: Log-Aggregation (siehe `infra/loki/`, `infra/promtail/`)
- **OpenTelemetry**: Tracing via `tracing-actix-web`

### 9.5 Quality Gates (MANDATORY)

Vor jedem Commit:

```bash
cargo fmt
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -D warnings
```

Bekannte nicht-blockierende Warnings:
- `proc-macro-error2 v2.0.1` — zukünftige Inkompatibilität (Leptos 0.8 → Upgrade auf 0.9+)
- `Cargo.toml: unused manifest key: profile.test.edition` (workspace-level)
- Edition 2015 cached warnings (harmlos)

---

## 10. Umgebungsvariablen

| Variable | Verwendung |
|----------|-----------|
| `DATABASE_URL` | PostgreSQL-Verbindungsstring |
| `NATS_URL` | NATS-Verbindungs-URL |
| `JWT_SECRET` | JWT Signier-Schlüssel |
| `POSTGRES_PASSWORD` | Datenbank-Passwort |
| `MQTT_BROKER_HOST` | MQTT Broker Host |
| `MQTT_BROKER_PORT` | MQTT Broker Port |
| `MQTT_CLIENT_ID` | MQTT Client ID |
| `MQTT_TOPIC_PREFIX` |MQTT-Themen-Präfix |
| `DATABASE_MAX_CONNECTIONS` | Max. Pool-Größe |
| `DATABASE_MIN_CONNECTIONS` | Min. Pool-Größe |
| `DATABASE_IDLE_TIMEOUT_SECS` | Idle Timeout |
| `DATABASE_MAX_LIFETIME_SECS` | Max. Verbindungslebensdauer |

---

## 11. Performance-Optimierungen (Quick Wins)

Alle 5 Quick Wins implementiert (siehe `docs/optimizations.md`):

1. **DecodingKey Caching** — `OnceLock` in AuthExtractor, eliminiert pro-Request-Allocation
2. **PgPoolOptions Konfiguration** — Umgebungsvariablen für Pool-Tuning
3. **Repository Factory Macro** — `repo!` Macro reduziert Boilerplate auf One-Liner
4. **Messaging Topic Precomputation** — Statische `&'static str` Konstanten für NATS-Subjects
5. **Role Mapping Optimization** — `Claims` speichert `Vec<UserRole>` direkt, zero-allocation `roles()`

---

## 12. Bekannte Issues & Pitfalls

| Issue | Lösung |
|-------|--------|
| `proc-macro-error2 v2.0.1` future incompatibility | Leptos auf 0.9+ upgraden |
| `Cargo.toml: unused manifest key: profile.test.edition` | Harmlos, workspace-level |
| Edition 2015 cached warnings | Harmlos, ignorieren |
| `dev.sh` failt beim Start | Docker-PostgreSQL muss laufen; alternativ `dev-start.sh` nutzen |
| `trunk` nicht gefunden | `cargo install trunk` |
| SQLx Offline-Mode fehlt | `cargo sqlx prepare --workspace` ausführen |
| `dev.sh` Spinner blockiert / Skript endet früh | `SPINNER_RUNNING=0` ist nicht sichtbar (Subshell); `kill $SPINNER_PID` verwenden |
| CORS zu permissiv | In Produktivumgebung Whitelist konfigurieren |
| argon2 in RC-Version | Auf stabile Version wechseln |

---

## 13. Lizenz

Dieses Projekt ist lizenziert unter der GNU GPL v3.0 oder späterer Lizenz.

```
AgroCore-RS — Copyright (c) 2024-2026 Jens Reinemuth
Lizenz: GNU General Public License v3.0 or later
```
