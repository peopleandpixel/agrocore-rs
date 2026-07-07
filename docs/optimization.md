# agrocore-rs Optimierungsplan

> Erstellt: 2025-07-04  
> Basis: Code-Review aller Crates (domain, infrastructure, api, shared, messaging, weather-service, reporting-service, admin-ui)

---

## Inhaltsverzeichnis

1. [Testing (Kritisch)](#1-testing-kritisch)
2. [Code-Qualität & Linting](#2-code-qualität--linting)
3. [API & Serialisierung](#3-api--serialisierung)
4. [Domain-Modelling](#4-domain-modelling)
5. [Sicherheit](#5-sicherheit)
6. [Observability](#6-observability)
7. [Performance & Skalierung](#7-performance--skalierung)
8. [Frontend (Admin-UI)](#8-frontend-admin-ui)
9. [Infrastructure / DevOps](#9-infrastructure--devops)
10. [Domain-spezifisch (Landwirtschaft)](#10-domain-spezifisch-landwirtschaft)
11. [Priorisierung & Roadmap](#11-priorisierung--roadmap)

---

## 1. Testing (Kritisch)

### Status Quo
- **Unit Tests**: Nur Inline in Domain (`#[cfg(test)]` Module)
- **Integration Tests**: 3 API-Tests (health, metrics, swagger) – keine DB
- **Property-based Tests**: Fehlend
- **Contract Tests**: Fehlend (API ↔ Frontend Types)
- **E2E Tests**: Fehlend

### Anforderungen (aus Memory)
> Tests IMMER in separaten Verzeichnissen (`crates/<name>/tests/`), niemals inline. `cargo check` muss OHNE Warnings durchlaufen.

### Aufgaben

| ID | Aufgabe | Crate | Aufwand |
|----|---------|-------|---------|
| T-01 | `tests/` Verzeichnisse pro Crate anlegen | alle | 1h |
| T-02 | Unit Tests für `calculation.rs` (Property-based mit `proptest`) | domain | 4h |
| T-03 | Repository Tests mit Testcontainers (MongoDB) | infrastructure | 8h |
| T-04 | API Handler Tests (Actix test::TestServer) | api | 8h |
| T-05 | Messaging Tests (NATS JetStream Mock) | messaging | 4h |
| T-06 | Frontend Component Tests (Leptos testing-library) | admin-ui | 8h |
| T-07 | E2E Tests (Playwright) – Login, Sites CRUD, Map | admin-ui + api | 16h |
| T-08 | Contract Tests (OpenAPI Schema ↔ TypeScript Types) | api + admin-ui | 4h |
| T-09 | CI Pipeline: `cargo test --workspace` muss grün | all | 2h |

### Test-Struktur pro Crate
```
crates/<name>/
├── src/
├── tests/
│   ├── unit/           # Pure logic, no DB
│   ├── integration/    # With Testcontainers MongoDB
│   └── fixtures/       # Test data builders
```

---

## 2. Code-Qualität & Linting

### Probleme
- `write_file` Linter erkennt Edition 2024 nicht → **immer `cargo check` nutzen**
- Keine einheitliche Clippy-Konfiguration
- Kein `rustfmt.toml` im Workspace

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| L-01 | `clippy.toml` mit Workspace-Regeln erstellen | 1h |
| L-02 | `rustfmt.toml` (indent 4, max_width 100) | 30min |
| L-03 | CI: `cargo clippy --workspace -- -D warnings` | 30min |
| L-04 | CI: `cargo fmt --check --workspace` | 30min |
| L-05 | `cargo check --workspace` als Pre-Commit Hook (husky/lefthook) | 1h |
| L-06 | Edition 2024 Migration prüfen (aktuell 2021) | 4h |

### Clippy-Regeln (Vorschlag)
```toml
# clippy.toml
avoid-breaking-exported-api = true
cargo = true
complexity = "warn"
correctness = "deny"
maintainability = "warn"
nursery = "warn"
pedantic = "warn"
perf = "warn"
style = "warn"
suspicious = "deny"

# Allow spezifisch
allow = [
    "module_name_repetitions",    # Domain-Driven Design
    "missing_docs_in_private_items",
    "too_many_arguments",         # DTOs haben viele Felder
]
```

---

## 3. API & Serialisierung

### Probleme

| Problem | Details |
|---------|---------|
| **OpenAPI Duplikate** | `PACApplication`, `CostCenter`, `FinancialRecord` 2x in `components.schemas` |
| **ID-Inkonsistenz** | `TenantId` Newtype, aber `User::id` = `Uuid`, `Site::id` = `Uuid` |
| **Enum-Serialisierung** | `SiteType::Other(String)`, `CropType::Other(String)` – nicht typsicher |
| **API Versioning** | `/api/v1/` ok, aber keine Deprecation-Strategie |
| **DTO ↔ Entity Mapping** | Manuell in Handlern, fehleranfällig |

### Aufgaben

| ID | Aufgabe | Crate | Aufwand |
|----|---------|-------|---------|
| A-01 | OpenAPI Duplikate entfernen, `components` deduplizieren | api | 2h |
| A-02 | Unit Types für alle IDs (`SiteId`, `OrderId`, `WorkerId`, ...) | domain/shared | 4h |
| A-03 | `strum` für Enum-Serialisierung (Display, EnumIter, VariantNames) | domain | 2h |
| A-04 | API Versioning Policy dokumentieren (Header vs Path) | api | 2h |
| A-05 | `utoipa` `#[derive(ToSchema)]` auf alle DTOs prüfen | api | 2h |
| A-06 | Auto-Mapping Entity↔DTO (macros oder `impl From`) | api/domain | 4h |
| A-07 | Request Validation Middleware (zentral, nicht pro Handler) | api | 4h |

---

## 4. Domain-Modelling

### Probleme

| Entity | Felder | Problem |
|--------|--------|---------|
| `Site` | 38 | God Object, mischt Agronomie, Compliance, Geo, Admin |
| `Order` | Recurrence inline | Recurrence = eigenes Aggregate |
| `PlantProtectionRecord` | Dupliziert Flächenlogik | Sollte `CalculationService` nutzen |
| `WorkerLocation` | Einfache Points | Event Sourcing für Tracks besser |

### Ziel: Aggregate Roots definieren

```
Tenant
├── Site (Core) → SiteAgronomy, SiteCompliance, SiteGeo, SiteAdmin
├── Order (Core) → OrderRecurrence, OrderExecution
├── Worker (Core) → WorkerLocation (Event Sourced), WorkLog
├── Animal (Core) → Treatment, Grazing
├── Vineyard / OliveGrove (Sub-Aggregates von Site)
├── WaterSource / WaterQuota / WaterUsage
└── WeatherStation / PhenologyRecord
```

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| D-01 | `Site` aufsplitten: `SiteCore` + `SiteAgronomy` + `SiteCompliance` + `SiteGeo` | 16h |
| D-02 | `OrderRecurrence` als eigenes Entity + Repository | 8h |
| D-03 | `PlantProtectionRecord` → `CalculationService` delegieren | 4h |
| D-04 | `WorkerLocation` → Event Store (NATS JetStream + MongoDB) | 16h |
| D-04 | Domain Events definieren (für Event Sourcing / CQRS) | 8h |
| D-05 | Value Objects extrahieren (`GeoPoint`, `AreaHa`, `DosagePerHa`, `BbchStage`) | 8h |

---

## 5. Sicherheit

### Lücken

| Lücke | Risiko | Fix |
|-------|--------|-----|
| JWT Secret nur Env, keine Rotation | Token-Klau → dauerhaft gültig | Key Rotation + JWKS Endpoint |
| Kein Rate Limiting | Brute Force, DoS | `actix-web-limiter` |
| CORS `permissive()` | CSRF, Data Leak | Origin-Whitelist per Tenant/Env |
| Audit-Log nicht tamper-proof | Manipulation unbemerkt | Hash-Chain / Merkle Tree |
| Argon2 `0.6.0-rc.8` (Pre-Release) | Instabilität | Stable Version (`0.6.0`) prüfen |
| Auth Token in `localStorage` (Frontend) | XSS → Token Diebstahl | HttpOnly Cookie + CSRF Token |
| Keine Security Headers | CSP, HSTS, X-Frame-Options | Middleware |

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| S-01 | JWT Key Rotation + JWKS Endpoint (`/.well-known/jwks.json`) | 8h |
| S-02 | Rate Limiting Middleware (pro IP + pro User) | 4h |
| S-03 | CORS Configuration per Environment (dev/staging/prod) | 2h |
| S-04 | Audit-Log Hash-Chain (jeder Entry hash(prev + current)) | 8h |
| S-05 | Argon2 auf Stable updaten, `argon2` Config härten | 2h |
| S-06 | Frontend: HttpOnly Cookie Auth + CSRF (Double Submit) | 8h |
| S-07 | Security Headers Middleware (CSP, HSTS, Referrer-Policy) | 2h |
| S-08 | Penetration Test / Dependency Audit (`cargo audit`) | 4h |

---

## 6. Observability

### Aktueller Stand
- **Logging**: `tracing` aber ohne strukturierte Fields (`tenant_id`, `site_id`, `user_id`)
- **Metrics**: Nur Prometheus Default (HTTP Requests), keine Business-Metrics
- **Health**: `/health` → nur `{"status":"ok"}` – keine Dependency Checks
- **Distributed Tracing**: Fehlend (kein W3C Trace Context, kein OpenTelemetry)

### Gewünschte Metrics

```prometheus
# Business
agrocore_sites_total{tenant_id="..."} 123
agrocore_orders_created_total{type="plant_protection",tenant_id="..."} 45
agrocore_area_ha_tracked{tenant_id="...", crop_type="grape"} 1200.5
agrocore_pesticide_kg_applied{tenant_id="...", substance="copper"} 12.3

# Technical
agrocore_db_query_duration_seconds{repo="site",op="find"} 0.045
agrocore_nats_publish_duration_seconds 0.012
agrocore_jwt_validation_duration_seconds 0.003
```

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| O-01 | Strukturiertes Logging: `tracing::info!(tenant_id=%tid, site_id=%sid, ...)` | 4h |
| O-02 | OpenTelemetry Setup (OTLP Exporter → Jaeger/Tempo) | 8h |
| O-03 | Business Metrics (Prometheus) – Macro/Helper für Counter/Histogram | 8h |
| O-04 | Health Checks: DB Ping, NATS Connect, Disk Space, Memory | 4h |
| O-05 | Correlation IDs (Trace Context) durch alle Services | 4h |
| O-06 | Grafana Dashboards (JSON) für API, Business, Infra | 8h |
| O-07 | Alerting Rules (PrometheusRule) – Latency, Error Rate, Disk | 4h |

---

## 7. Performance & Skalierung

### Engpässe

| Bereich | Problem | Lösung |
|---------|---------|--------|
| Repository `find_all_visible` | Lädt alle, filtert im Speicher | MongoDB `$match` mit `visibility_filter` |
| `SiteDto::from` | N+1 Queries für Relations | Eager Loading / DataLoader Pattern |
| Pagination | Offset-based → langsam bei großen Daten | Cursor-based (keyset pagination) |
| Connection Pool | 100 max / 10 min fix | Tuning via Env, Monitoring |
| NATS | Kein JetStream | Persistence, Replay, Ack, Consumer Groups |

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| P-01 | Repository Queries: `visibility_filter` in MongoDB Query pushen | 8h |
| P-02 | DataLoader / Eager Loading für DTO Mapping | 8h |
| P-03 | Cursor-based Pagination (after/before + limit) | 8h |
| P-04 | Connection Pool Tuning (Env Config + Metrics) | 4h |
| P-05 | NATS → JetStream Migration (Streams, Consumers, Ack) | 16h |
| P-06 | Caching Layer (Redis) für häufige Reads (Sites, Users, Config) | 12h |
| P-07 | Database Index Audit (unused, missing, compound) | 4h |
| P-08 | Load Testing (k6 / vegeta) – Baseline & Regression | 8h |

---

## 8. Frontend (Admin-UI)

### Probleme

| Issue | Auswirkung |
|-------|------------|
| `Box::leak` für i18n Strings | Memory Leak bei Sprachwechsel |
| Keine E2E Tests | Regressionen unbemerkt |
| Leaflet direkt via `web-sys` | Wartungsschwierig, keine Leptos-Integration |
| Auth Token in `localStorage` | XSS → Token Diebstahl |
| Keine Optimistic Updates | Schlechte UX bei Latenz |
| Keine Storybook / Component Library | Inconsistente UI |

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| F-01 | i18n: `Box::leak` entfernen → `Memo`/`RwSignal` + `use_context` | 4h |
| F-02 | Auth: HttpOnly Cookie + CSRF (Backend S-06 Voraussetzung) | 8h |
| F-03 | Optimistic Updates mit `Action`/`Resource` Pattern | 8h |
| F-04 | Leaflet Wrapper (`leptos-leaflet` oder eigener) | 8h |
| F-05 | E2E Tests: Playwright (Login, Sites CRUD, Map, Wizard) | 16h |
| F-06 | Component Tests (Leptos testing-library) | 8h |
| F-07 | Storybook / UI Kit (Radix Leptos / Tailwind Components) | 16h |
| F-08 | PWA Support (Service Worker, Offline, Install) | 8h |
| F-09 | Bundle Size Analyse (`wasm-opt`, `twiggy`) | 4h |

---

## 9. Infrastructure / DevOps

### Fehlend

| Komponente | Status |
|------------|--------|
| Dockerfile (multi-stage) | ❌ |
| docker-compose.yml (Local Dev) | ❌ |
| GitHub Actions CI/CD | ❌ |
| MongoDB Migration Strategy | ❌ |
| Secrets Management | ❌ |
| Backup/Restore Scripts | ❌ |
| Helm Charts / K8s Manifests | ❌ |

### Aufgaben

| ID | Aufgabe | Aufwand |
|----|---------|---------|
| I-01 | Multi-stage Dockerfiles (API, Weather, Reporting, UI) | 8h |
| I-02 | `docker-compose.yml` (MongoDB, NATS, API, UI, Grafana, Prometheus) | 8h |
| I-03 | GitHub Actions: check → test → build → docker → deploy | 16h |
| I-04 | MongoDB Migration Tool (Versioned, Idempotent) | 12h |
| I-05 | Secrets: Vault / Sealed Secrets / SOPS Integration | 8h |
| I-06 | Backup/Restore Scripts (MongoDB, NATS JetStream) | 8h |
| I-07 | Helm Charts für Kubernetes Deployment | 16h |
| I-08 | Staging Environment (Preview Deployments per PR) | 8h |
| I-09 | Dependency Renovate / Dependabot Config | 2h |

---

## 10. Domain-spezifisch (Landwirtschaft)

### Fachliche Lücken (DE/PT/ES Regulatorik)

| Feature | Beschreibung | Priorität | Aufwand |
|---------|--------------|-----------|---------|
| **Dünge-Bilanz** | N/P/K Bilanzierung pro Schlag/Betrieb (Düngeverordnung DE, GAP) | **Hoch** | 40h |
| **GAP Cross-Compliance Checks** | Greening, GAEC, SMR Prüfungen automatisiert | **Hoch** | 40h |
| **PAC/Antrag Management** | Flächenmeldung, Eco-Schemes, Coupled Support | **Hoch** | 32h |
| **Pflanzenschutz-Nachweis** | Electronic Application Records (DE: PflSchAnwV, ES: CUADERNO) | **Hoch** | 24h |
| **Feldstück-Historie / Fruchtfolge** | Crop Rotation Planning, Vorfrucht-Effekte | Mittel | 24h |
| **ISOBUS / Maschinendaten** | ISOXML Import, Job Data Exchange (JDE) | Mittel | 40h |
| **Wetter-basierte Prognosen** | Krankheitsmodelle (Peronospora, Oidium, Botrytis) | Niedrig | 32h |
| **Montado/Dehesa Management** | Korkernte, Beweidung, EU-Förderung (PT/ES) | Mittel | 24h |
| **Wasserrecht / Konzessionen** | Comunidad de Regantes, Konzessions-Management | Mittel | 16h |
| **Öl-Rückverfolgbarkeit** | Charge-Tracking vom Hain bis Flasche (EU 2023/2419) | Niedrig | 16h |

### Bestehende Basis nutzen
- `PlantProtectionAreaMethod` → Basis für Pflanzenschutz-Nachweis
- `CalculationService` → Dünge-Berechnung erweitern
- `Site` + `SigpacData` → PAC-Flächenmeldung
- `OliveOilRecord` + `OliveGrove` → Rückverfolgbarkeit

---

## 11. Priorisierung & Roadmap

### Phase 1: Fundament (Wochen 1-4) – "Make it reliable"
| Priorität | Tasks | Ziel |
|-----------|-------|------|
| **P0** | T-01, T-02, T-03, T-09 | Tests laufen in CI |
| **P0** | L-01 bis L-05 | Clippy/Fmt/Check grün |
| **P0** | A-01, A-02 | API sauber, IDs typsicher |
| **P0** | S-02, S-03, S-08 | Basis-Sicherheit |
| **P0** | I-01, I-02 | Local Dev mit Docker Compose |

### Phase 2: Observability & Performance (Wochen 5-8) – "Make it observable"
| Priorität | Tasks | Ziel |
|-----------|-------|------|
| **P1** | O-01, O-02, O-03, O-04 | Logging, Tracing, Metrics, Health |
| **P1** | P-01, P-03 | Repository Performance, Cursor Pagination |
| **P1** | F-01, F-02 | Frontend Memory Leak, Auth Security |

### Phase 3: Domain-Modelling & Events (Wochen 9-14) – "Make it scalable"
| Priorität | Tasks | Ziel |
|-----------|-------|------|
| **P1** | D-01, D-02, D-05 | Aggregate Roots, Value Objects |
| **P1** | P-05 | NATS → JetStream |
| **P1** | M-01 (Messaging Tests) | Event-Driven verlässlich |

### Phase 4: Fachliche Features (Wochen 15+) – "Make it useful"
| Feature | Tasks | Schätzung |
|---------|-------|-----------|
| Dünge-Bilanz | D-03, Domain-1, Domain-2 | 40h |
| GAP Compliance | Domain-2, Domain-3 | 40h |
| PAC Management | A-07, Domain-3 | 32h |
| Pflanzenschutz-Nachweis | D-03, Domain-4 | 24h |

### Phase 5: Platform & UX (Laufend)
| Bereich | Tasks |
|---------|-------|
| Frontend | F-03 bis F-09 |
| DevOps | I-03 bis I-09 |
| Domain | Restliche Domain-Features |

---

## Schätzung Gesamt

| Kategorie | Stunden |
|-----------|---------|
| Testing | 55h |
| Code Quality | 7h |
| API/Serialisierung | 20h |
| Domain Modelling | 60h |
| Sicherheit | 38h |
| Observability | 40h |
| Performance | 60h |
| Frontend | 80h |
| Infrastructure | 86h |
| Domain-Features | 288h |
| **Total** | **~734h** (~19 Wochen @ 40h, ~38 Wochen @ 20h) |

---

## Quick Wins (erste 2 Tage)

1. `cargo clippy --workspace -- -D warnings` → Fixes committen
2. `cargo fmt --check --workspace` → Fixes committen
3. `clippy.toml` + `rustfmt.toml` anlegen
4. Docker Compose für MongoDB + NATS + API
5. `tests/` Verzeichnisse anlegen, erste Unit Tests für `calculation.rs`
6. OpenAPI Duplikate in `api/src/lib.rs` entfernen
7. Health Check: DB Ping + NATS Connect

---

## Nächste Schritte

> **Entscheidung nötig**: Welche Phase startet du?  
> Empfehlung: **Phase 1** (Fundament) – dafür brauche ich dein Go für:
> - Testcontainers Setup (MongoDB in CI)
> - Docker Compose Struktur
> - Clippy Config Präferenzen

Soll ich den **Phase-1-Implementierungsplan** als separate `.hermes/plan.md` mit konkreten Tasks & Befehlen erstellen?