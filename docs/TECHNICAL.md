# AgroCore-RS — Technical Documentation

> Version: 0.49.0
> Last updated: 2026-08-11
> License: GPL-3.0-or-later

---

## 1. Project Overview

**AgroCore-RS** is an agricultural farm-management platform built from the ground up in Rust. It unifies field and parcel management, order and task planning, weather and phenology data, livestock, finance, compliance, machinery management, IoT integration, and multi-country LPIS support in a single system.

### 1.1 Stack

| Layer             | Technology                          |
|-------------------|-------------------------------------|
| Language          | Rust 2024 Edition                   |
| HTTP API          | Actix Web 4                         |
| Frontend          | Leptos 0.8 (SPA, WASM-based)        |
| Database          | PostgreSQL 16 + PostGIS 3.4         |
| Messaging         | NATS 2.10 (JetStream) + MQTT 5      |
| IoT Protocol      | MQTT via rumqttc 0.25               |
| Container         | Docker + Docker Compose             |
| CI/CD             | GitHub Actions + GitLab CI          |
| OpenAPI           | utoipa 5 + Swagger UI 8             |
| Authentication    | JWT (jsonwebtoken 10) + argon2 0.6  |
| Rate Limiting     | actix-governor                      |
| Caching           | moka (memory) + Redis               |
| Geo Libraries     | geo 0.33, geojson 1.0, geozero 0.15 |
| Export            | rust_xlsxwriter 0.95                |

### 1.2 Design Goals

- **Local-first**: everything runs locally by default; the cloud is an option, not a prerequisite.
- **Multi-tenant**: every tenant is isolated via PostgreSQL Row-Level Security (RLS).
- **Domain-Driven Design**: clean separation of domain logic, infrastructure, and API.
- **Real-time**: NATS event bus for loose coupling, webhooks for external integration.
- **Extensible**: plugin architecture for LPIS providers (SIGPAC, iLPIS, RPG, SIAN, BRP, ...).

---

## 2. Architecture

### 2.1 High-Level Architecture

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

### 2.2 Workspace Structure

```
agrocore-rs/
├── Cargo.toml                  # Workspace manifest (Rust 2024)
├── crates/
│   ├── api/                    # Actix Web HTTP API + OpenAPI + Swagger UI
│   │   ├── src/
│   │   │   ├── handlers/       # HTTP handlers (33 modules)
│   │   │   ├── dto/            # API DTOs
│   │   │   ├── services/       # ImportService
│   │   │   ├── middleware.rs   # AuthExtractor, CORS, Security Headers
│   │   │   └── lib.rs          # AppState, OpenApi, Server bootstrap
│   │   └── tests/              # Integration tests
│   ├── admin-ui/               # Leptos 0.8 SPA (WASM)
│   ├── domain/                 # Core business logic, entities, traits
│   │   ├── src/
│   │   │   ├── entities/       # 17 domain modules
│   │   │   ├── repositories.rs # Repository traits
│   │   │   └── services/       # WorkflowService, CalculationService, NutritionService
│   ├── infrastructure/         # PostgreSQL repository implementations
│   ├── messaging/              # NATS + MQTT client, event definitions
│   ├── shared/                 # Shared types (pagination, auth, LPIS)
│   ├── lpis-providers/         # Multi-country LPIS providers (8 countries)
│   ├── reporting-service/      # Excel/GeoJSON/PAC-SIP export worker
│   ├── weather-service/        # Weather data ingestion worker
│   ├── geometry-service/       # Geodata processing
│   └── asset-registry/         # Asset management
├── migrations/                 # 18 SQL migration files
├── docker-compose.yml          # Production compose (Postgres, NATS, Mosquitto, API, Admin UI)
├── scripts/
│   ├── dev.sh                  # Full dev stack (Docker-based)
│   ├── dev-start.sh            # Lightweight dev start (API + Admin UI)
│   └── sigpac_import/          # SIGPAC bulk import (Python)
├── infra/                      # Kubernetes, Prometheus, Loki, Promtail
└── docs/                       # Documentation
```

### 2.3 Domain-Driven Design: Entity + Repository Pattern

Every domain follows a consistent pattern:

```rust
// 1. Entity (domain/src/entities/{entity}.rs)
pub struct Site {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub label: String,
    // ... further fields
}

// 2. DTOs for Create/Update
pub struct CreateSiteDto { pub label: String, /* ... */ }
pub struct UpdateSiteDto { pub label: Option<String>, /* ... */ }

// 3. Repository trait (domain/src/repositories.rs)
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

// 4. PostgreSQL implementation (infrastructure/src/postgres/site.rs)
pub struct PgSiteRepo { pool: PgPool }
impl SiteRepository for PgSiteRepo { ... }

// 5. Factory method (infrastructure/src/postgres/database.rs)
pub fn site_repo(&self) -> Arc<dyn SiteRepository> { ... }
```

### 2.4 Tenant Isolation (RLS)

All tables contain a `tenant_id UUID` column with `ON DELETE CASCADE`.

```sql
-- RLS policy example
CREATE POLICY sites_tenant_isolation ON sites
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Application-side on every request:
SET LOCAL app.current_tenant_id = $1;
SET LOCAL app.is_superadmin = $1;
```

A `TenantId` wrapper type (`pub struct TenantId(pub Uuid)`) in the domain layer ensures compile-time safety against mixing up `Uuid` and `TenantId`.

### 2.5 PaginatedResponse

Uniform response format for all paginated endpoints:

| Field        | Type    | Description               |
|--------------|---------|---------------------------|
| data         | Vec<T> | Result items              |
| total        | u64    | Total number of records   |
| page         | u64    | Current page              |
| per_page     | u64    | Items per page            |
| total_pages  | u64    | Total number of pages     |

---

## 3. Domain Modules

AgroCore-RS covers 17 integrated domain modules:

| # | Module                | Core functionality                                          | Status     |
|---|-----------------------|-------------------------------------------------------------|------------|
| 1 | **Sites/Parcels**     | Areas, boundary, GeoPoint, plots, SigpacData, custom fields | Production |
| 2 | **Orders/Tasks**      | Work orders, recurrence, execution policies, automation     | Production |
| 3 | **Workforce**         | Workers, work logs, site reporting, task status             | Production |
| 4 | **Equipment**         | Machinery, service intervals, costs                        | Production |
| 5 | **Weather/Phenology** | Weather stations, weather data, phenological stage (BBCH)   | Production |
| 6 | **Compliance**        | Checklists, audit trail, plant protection, fertilizers      | Production |
| 7 | **Vineyard**          | Vineyards, must calculation, quality classes                | Production |
| 8 | **Olive**             | Olive groves, olive oil records, quality classes            | Production |
| 9 | **Water**             | Water sources, usage, quotas, irrigation methods            | Production |
|10 | **Harvest**           | Harvest seasons, lots, delivery, cold chain tracking        | Production |
|11 | **Livestock**         | Animals, species, treatments, pasture, status               | Production |
|12 | **Finance**           | PAC applications, cost centers, financial bookings          | Production |
|13 | **Nutrition**         | Fertilizer planning, nutrient balance                      | Production |
|14 | **Import Service**    | GeoJSON, Shapefile, REGEPAC import with LPIS validation    | Production |
|15 | **LPIS Providers**    | Multi-country LPIS (8 countries) with caching + rate limiting | Production |
|16 | **IoT/Devices**       | Device registry, telemetry, commands, HA discovery         | Production |
|17 | **System**            | Health status, initial setup, tenant management            | Production |

### 3.1 LPIS Multi-Country Provider

Supported countries with their respective providers:

| Country   | Code | Provider | Data source                     |
|-----------|------|----------|---------------------------------|
| Spain     | ES | SIGPAC | Sistema de Información Geográfica de Parcelas Agrícolas |
| Portugal  | PT | iLPIS | Integrated Land Parcel Identification System |
| France   | FR | RPG | Registre Parcellaire Graphique |
| Italy    | IT | SIAN | Sistema Informativo Agricolo Nazionale |
| Netherlands | NL | BRP | Basisregistratie Percelen (PDOK WFS) |
| Germany  | DE | LPIS | State-specific LPIS portals |
| Poland   | PL | LPIS | ARiMR Geoportal WFS |
| Austria  | AT | INVEKOS | AMA/data.gv.at WFS |

#### Provider Architecture

- **`BaseClient`** (`lpis-providers/src/base.rs`): shared HTTP client logic with:
  - Configurable timeout
  - Caching via `LpisCache` (in-memory Moka / Redis backend)
  - Rate limiting via `governor`
  - Retry logic with exponential backoff (no external `retry` crate)
  - Configurable via `ProviderConfig`
- **`LpisRegistry`**: central registry for all providers
- **`LpisCache`**: uniform caching layer with TTL and max entries

### 3.2 Event System (NATS)

The NATS event bus connects all services:

```
events.sites       → SiteCreated, SiteUpdated, SiteDeleted
events.orders      → OrderCreated, OrderUpdated, OrderDeleted
events.users       → UserCreated, UserUpdated, UserDeleted
events.spatial     → SpatialPolygonEntered, SpatialPolygonIn, SpatialPolygonExited
events.iot.*       → IoTTelemetry, IoTDeviceStatus, IoTCommand
events.harvest.*   → HarvestSeasonCreated, LotDelivered, ColdChainAlert
```

NATS JetStream is used for persistent messages.

---

## 4. Database Schema

### 4.1 Migrations

11 migration files in `/migrations/`, numbered so that `sqlx::migrate!`
applies them in order:

| File | Description |
|------|-------------|
| `0000000000_consolidated_init.sql` | Consolidated initial schema — tenants, users, sites, equipment, orders, weather, animals, tasks, orders, ~190 row-level-security policies |
| `0000000001_fix_site_area_type.sql` | Corrects the `sites.area` column type |
| `0000000002_fix_sites_insert_defaults.sql` | Insert defaults required by the sites repository |
| `0000000003_missing_domain_tables.sql` | Eight domain tables the repositories queried but no migration created (spatial_objects, groups, trees, buildings, livestock, water_usages, animal treatments/grazing) |
| `0000000004_force_rls.sql` | Makes row-level security effective: `agrocore_app` role without BYPASSRLS, FORCE on every RLS table, grants, tenant INSERT policy |
| `0000000005_system_settings.sql` | Typed key/value settings with tenant overrides and inherited system defaults, plus the shipped defaults and the LPIS provider endpoints |
| `0000000006_workforce_schema_alignment.sql` | Aligns `workers` and `worker_task_statuses` with their entities — the repositories read columns the tables never had |
| `0000000007_geography_spatial_indexes.sql` | GIST indexes for the geography and spatial columns |
| `0000000008_tenant_delete_cascade.sql` | Cascades foreign keys and audit rows on tenant deletion, which failed with `audit_logs_tenant_id_fkey` |
| `0000000009_varieties_and_breeds_catalogue.sql` | Tenant scope for the cultivar and breed catalogues (`tenant_id`, `active`, `species_key`, `source`), RLS policies, and `planted_at` / `variety_id` on `spatial_objects` |
| `0000000010_crop_species_and_breed_performance.sql` | `crop_species` split out of `varieties`, wool micron ranges, strain-scoped egg figures, and EU registration scope per cultivar |

Two rules the layout depends on:

- **Every file in `migrations/` must carry a numeric prefix.** `sqlx::migrate!`
  rejects an unnumbered file outright, which is why the demo seed lives in
  `scripts/demo_seed.sql` rather than here — demo data is opt-in via `--demo`,
  migrations are not.
- **Demo data is not a migration.** Seeding belongs in `scripts/demo_seed.sql`
  and is loaded explicitly.

Migrations are applied automatically on server startup via
`sqlx::migrate!("./migrations")`.

### 4.2 Key Tables

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
    ... further fields (costs, language, color, ...)
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
    area NUMERIC(15,2),                 -- hectares
    polygon GEOMETRY(POLYGON, 4326),   -- PostGIS
    center GEOMETRY(POINT, 4326),
    lpis_country VARCHAR(2),           -- LPIS country code
    lpis_data JSONB,                   -- LPIS-specific data
    ... further fields
);
```

#### iot_devices
```sql
CREATE TABLE iot_devices (
    device_id TEXT PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    site_id UUID REFERENCES sites(id) ON DELETE SET NULL,
    device JSONB NOT NULL,              -- full device object
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

### 4.3 Indices

- GIST indices on all PostGIS geometry columns
- Hash indices on `tenant_id` in all tables
- GIN indices on JSONB columns (`roles`, `lpis_data`, `device`)
- Unique indices on `sigpac_reference`, `email`, `device_id`

---

### 4.2 Reference catalogue

Three tables, and the split between them is what makes grouping possible.

`crop_species` holds species with a stable `species_key` (`olea_europaea`,
`ovis_aries`, `vitis_vinifera`). `species_key` is **globally unique** and a tenant cannot
shadow it: it is the join target of every grouping query, so a tenant-local duplicate
would split one species into two in every aggregate. This is the only closed reference
value in the system.

`varieties` holds cultivars. `registration_countries` is the set of countries whose
national catalogue or official statistics list the cultivar — 825 grapevine cultivars
across 27 countries, most registered in more than one. That is what answers "may I plant
this where I operate"; origin alone is not a usable proxy. `vivc_no`, `berry_colour` and
`use_kind` come from the GrapeGen06 European Catalogue / VIVC, with the national lists
below merged in:

| Country | Source | Cultivars |
|---------|--------|----------|
| EU (21 states) | GrapeGen06 European Catalogue, Annex 1A (INRA/VIVC) | 699 |
| Australia | Wikipedia, VIVC number per cultivar | 179 |
| United States | USDA NASS California Grape Acreage Report 2024 | 80 |
| New Zealand | Wikipedia, planted area per cultivar | 43 |
| Argentina | INV Resoluciones 18/2022 and 20/2024 | 19 |
| Chile | SAG Catastro Vitícola Nacional | 8 |
| South Africa | Wikipedia, share of national crush | 5 |

A cultivar is stored once with every market it is registered in, so "which of these can I
grow in Germany, California or Australia" is one row and one predicate. Records are matched
by VIVC accession number, then by name, then by synonym — the VIVC prime name is often not
the everyday name (Chardonnay is listed as *Chardonnay Blanc*, Syrah carries *Shiraz*), and
name-only matching produced two rows for one cultivar.

88 olive cultivars have no `registration_countries`: no country publishes an olive national
catalogue, so their origin is recorded instead.

`breeds` holds breeds with reference performance values. Two conventions matter:

- **Wool is micron, not kilograms.** Fibre diameter decides market value, kilograms decide
  quantity. Merino grades 17.70–19.14, Suffolk 36.20–38.09. Both are kept; conflating
  them produced one misleading number.
- **Egg figures name their strain.** Published values for one breed disagree by up to 100
  eggs/year because they describe different strains. `egg_production_strain` records which
  population a figure describes, so a number is never read as universal.

Scope on all three tables: `tenant_id = 00000000-0000-0000-0000-000000000000` is the
global scope, readable by every tenant and writable only by a platform operator. That
combination cannot be expressed in RLS, because every policy is evaluated against a
tenant id; the sentinel keeps `tenant_id = $t OR tenant_id = $GLOBAL` an ordinary indexed
predicate. A tenant cannot shadow a global entry — it adds its own row.

Seed data: `scripts/catalogue_seed.sql`, generated by `scripts/build_catalogue_seed.py`
from the sources named per row. Idempotent — re-running changes nothing. 41 species,
732 cultivars, 110 breeds.


### 4.3 PostgreSQL version and upgrading an existing database

The project targets **PostgreSQL 18** with **PostGIS 3.6**, pinned as `postgis/postgis:18-3.6`
in every compose file, both CI pipelines and the test fixture. Tags `18-3.4` and `18-3.5`
do not exist, so a guessed tag fails only when the suite runs.

`18-3.6` is the only 18.x PostGIS tag that exists; `16-3.4` was the previous pin.

### Upgrading an existing cluster

PostgreSQL supports skipping intermediate major versions, so 16 → 18 goes directly — read
the 17 and 18 release notes regardless, in particular the Migration sections.

Two things to know before starting:

- **A dump taken on 18 cannot be restored into 16.** Verified:
  `pg_restore: error: unsupported version (1.16) in file header`. Once the database is
  upgraded, rolling back to 16 is no longer an option. Take a `pg_dumpall` in the custom
  format *before* the upgrade if a fallback matters.
- **`pg_dump` 18 against a 16 server works**, and the backup service already tolerates the
  unknown-parameter warnings a newer client produces on an older server.

`pg_upgrade` cannot move a Docker volume between two images, so upgrading the container
means a logical dump and restore (`pg_dumpall`, or `pg_dump` per database plus
`pg_upgrade`-style role restoration). Plan for a maintenance window.

The backup service shells out to `pg_dump` and `pg_restore`. Those binaries are **not
installed by any Dockerfile**, so a container built from this repository cannot run a
backup; it needs `postgresql-client` matching the server version.

## 5. Authentication & Authorization

### 5.1 JWT Auth

- **Signing**: `jsonwebtoken` (RS256/HS256)
- **Expiry**: 30 minutes for access tokens, 7 days for refresh tokens
- **Password Hashing**: argon2 (RFC 9106)
- **Middleware**: `AuthExtractor` in `middleware.rs` extracts and validates the bearer token

### 5.2 Role-Based Access Control (RBAC)

| Role       | Permissions                                         |
|------------|-----------------------------------------------------|
| admin       | Full access to all resources, tenant management      |
| manager     | CRUD on sites, orders, workers, equipment, compliance |
| worker      | Own tasks, work logs, site reporting only             |
| viewer      | Read-only (sites, orders, workers, reports)           |

Every API route uses `auth.require_manager()`, `auth.require_admin()`, `auth.require_any_role(...)`, etc.

### 5.3 Security Headers

```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```

### 5.4 Rate Limiting

- **Default**: 120 requests/minute per IP (via `actix-governor`)
- **Burst**: 120
- **For auth endpoints**: stricter limits recommended (see `docs/optimizations.md` §2.3)

---

## 6. IoT Integration (MQTT + Home Assistant)

### 6.1 MQTT Client

Implemented in `crates/messaging/src/lib.rs` with `rumqttc 0.25`:

- **Configuration**: `MqttConfig` (broker_host, broker_port, client_id, TLS, credentials)
- **Event types**:
  - `IoTTelemetryEvent` — measurements (temperature, humidity, SoilMoisture, ...)
  - `IoTDeviceStatusEvent` — device status (Online/Offline/Maintenance)
  - `IoTCommandEvent` — commands to devices
  - `IoTMeasurement` — measurement with timestamp and value

### 6.2 UnifiedMessagingClient

Enables dual operation (NATS + MQTT):
- `publish_event()` publishes to both backends simultaneously
- `publish_telemetry()` and `publish_device_status()` are MQTT-optimized (retained)
- `subscribe_device_commands()` for bidirectional communication

### 6.3 Home Assistant Auto-Discovery

Full implementation:
- `HaSensorConfig` for all capabilities (Temperature, Humidity, SoilMoisture, Light, GPS, BatteryLevel, SignalStrength)
- `HaBinarySensorConfig` for device availability (connectivity)
- `HaButtonConfig` for actuator commands
- `HaNumberConfig` for numeric settings
- `HaDeviceInfo` with identifiers, manufacturer, model, version, via_device
- Topics: `homeassistant/sensor/.../config`, `homeassistant/binary_sensor/.../config`, `agrocore/telemetry/...`, `agrocore/status/...`, `agrocore/commands/...`

### 6.4 Mosquitto Broker

In `docker-compose.yml`:
- `eclipse-mosquitto:2.0` on port 1883 (MQTT), 9001 (WebSockets)
- TLS certificates generated (CA, server, client, PKCS12)
- Authentication via password file
- Health checks via `mosquitto_sub`

---

## 7. Multi-Country LPIS Integration

### 7.1 Provider Architecture

```
LpisRegistry (crates/shared/src/lpis.rs)
  └── LpisProvider<T> (trait)
       ├── SigpacProvider (ES)  ← BaseClient + Caching + Rate Limiting
       ├── IlpisProvider (PT)
       ├── RpgProvider (FR)
       ├── SianProvider (IT)
       ├── BrpProvider (NL)  ← already migrated to BaseClient
       ├── LpisDeProvider (DE)
       ├── LpisPlProvider (PL)
       └── InvekosProvider (AT)
```

### 7.2 BaseClient Pattern

Every provider uses `BaseClient` from `lpis-providers/src/base.rs`:
- HTTP client with configurable timeout
- Caching via `LpisCache` (in-memory Moka / Redis)
- Rate limiting via `governor` (requests/second + burst)
- Retry logic with exponential backoff
- Configuration via `ProviderConfig` (base_url, timeout, cache_ttl, rate_limits, enabled)

### 7.3 LPIS Provider Settings API

```
GET  /api/v1/settings/lpis          — Load provider configuration
PUT  /api/v1/settings/lpis          — Save (with restart_required flag)
GET  /api/v1/settings/lpis/providers — List all providers with defaults
```

### 7.4 Import Service LPIS Integration

`ImportService` uses `LpisRegistry` for multi-country validation:
- All import DTOs (`GeoJsonImportRequest`, `ShapefileImportRequest`, `ImportSitesRequest`) support `lpis_country: Option<LpisCountry>`
- Backwards compatible: default is ES (SIGPAC) when not specified
- Fallback to the SIGPAC SQL function when the provider is unavailable

### 7.5 SIGPAC Import

Python scripts in `scripts/sigpac_import/`:
- `import_sigpac.py` — main import (15 Spanish regions, ~25M parcels)
- `discover_sources.py` — source URL discovery
- `test_setup.py` — environment verification

---

## 8. Reporting & Export

### 8.1 Export Worker

`crates/reporting-service/` runs as a separate worker:
- **Excel export**: `rust_xlsxwriter` for orders, sites, tasks
- **GeoJSON export**: area and boundary data
- **PAC-SIP export**: dedicated export for EU agricultural subsidies (PlanTEIL import format)

### 8.2 Export Endpoints

```
POST /api/v1/reports/orders-excel
POST /api/v1/reports/sites-geojson
POST /api/v1/reports/pac-sip
```

---

## 9. Development & Deployment

### 9.1 Local Development

```bash
# Full stack (Docker infrastructure)
./scripts/dev.sh

# Lightweight start (API + Admin UI, no Docker)
./scripts/dev-start.sh

# Direct API start
cargo run -p agrocore-api

# Admin UI locally
cd crates/admin-ui && trunk serve
```

### 9.2 Docker Compose (Production)

```yaml
services:
  postgres:    # postgis/postgis:18-3.6  (PostgreSQL 18.6, PostGIS 3.6.4)
  nats:        # nats:2.10-alpine (with JetStream)
  mosquitto:   # eclipse-mosquitto:2.0 (MQTT + WebSockets + TLS)
  api:         # Actix Web API (port 8080)
  reporting-service:
  weather-service:
  admin-ui:    # Leptos SPA (port 3000)
```

### 9.3 Kubernetes

Helm chart adaptations in `infra/k8s/`:
- `api-deployment.yaml`
- `worker-deployments.yaml`
- `ingress.yaml`

### 9.4 Monitoring

- **Prometheus**: metrics endpoint at `/metrics`
- **Grafana**: predefined dashboards (see `infra/prometheus/`)
- **Loki/Promtail**: log aggregation (see `infra/loki/`, `infra/promtail/`)
- **OpenTelemetry**: tracing via `tracing-actix-web`

### 9.5 Quality Gates (MANDATORY)

Before every commit:

```bash
cargo fmt
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -D warnings
```

Known non-blocking warnings:
- `proc-macro-error2 v2.0.1` — future incompatibility (Leptos 0.8 → upgrade to 0.9+)
- `Cargo.toml: unused manifest key: profile.test.edition` (workspace level)
- Edition 2015 cached warnings (harmless)

---

## 10. Environment Variables

| Variable | Usage |
|----------|-------|
| `DATABASE_URL` | PostgreSQL connection string |
| `NATS_URL` | NATS connection URL |
| `JWT_SECRET` | JWT signing key |
| `POSTGRES_PASSWORD` | Database password |
| `MQTT_BROKER_HOST` | MQTT broker host |
| `MQTT_BROKER_PORT` | MQTT broker port |
| `MQTT_CLIENT_ID` | MQTT client ID |
| `MQTT_TOPIC_PREFIX` | MQTT topic prefix |
| `DATABASE_MAX_CONNECTIONS` | Max pool size |
| `DATABASE_MIN_CONNECTIONS` | Min pool size |
| `DATABASE_IDLE_TIMEOUT_SECS` | Idle timeout |
| `DATABASE_MAX_LIFETIME_SECS` | Max connection lifetime |

---

## 11. Performance Optimizations (Quick Wins)

All 5 quick wins implemented (see `docs/optimizations.md`):

1. **DecodingKey Caching** — `OnceLock` in AuthExtractor eliminates per-request allocation
2. **PgPoolOptions Configuration** — environment variables for pool tuning
3. **Repository Factory Macro** — `repo!` macro reduces boilerplate to a one-liner
4. **Messaging Topic Precomputation** — static `&'static str` constants for NATS subjects
5. **Role Mapping Optimization** — `Claims` stores `Vec<UserRole>` directly, zero-allocation `roles()`

---

## 12. Known Issues & Pitfalls

| Issue | Solution |
|-------|----------|
| `proc-macro-error2 v2.0.1` future incompatibility | Upgrade Leptos to 0.9+ |
| `Cargo.toml: unused manifest key: profile.test.edition` | Harmless, workspace level |
| Edition 2015 cached warnings | Harmless, ignore |
| `dev.sh` fails on startup | Docker PostgreSQL must be running; alternatively use `dev-start.sh` |
| `trunk` not found | `cargo install trunk` |
| SQLx offline mode missing | Run `cargo sqlx prepare --workspace` |
| `dev.sh` spinner blocks / script ends early | `SPINNER_RUNNING=0` is not visible (subshell); use `kill $SPINNER_PID` |
| CORS too permissive | Configure a whitelist in production |
| argon2 in RC version | Switch to a stable version |

---

## 13. License

This project is licensed under the GNU GPL v3.0 or later license.

```
AgroCore-RS — Copyright (c) 2024-2026 Jens Reinemuth
License: GNU General Public License v3.0 or later
```
