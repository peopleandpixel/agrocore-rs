# Agrocore-RS Implementation Timeline

**Status:** Project compiles successfully. Domain entities, repositories, and infrastructure layer are well-defined. API handlers exist for most modules.

**Last Updated:** 2026-07-14
**Current State:** Core modules implemented, some modules need PostgreSQL repo completion & API endpoint wiring

---

## 📊 Module Status Matrix

| Module | Landing Page Claim | Domain Entity | Repository Trait | PG Repo | API Handler | Tests | Status |
|--------|-------------------|---------------|------------------|---------|-------------|-------|--------|
| **1. Core: Flächen & Parzellen** | ✅ Multi-Crop, SIGPAC, PostGIS | ✅ site.rs | ✅ SiteRepository | ✅ site.rs | ✅ sites.rs | ✅ | **PRODUCTION** |
| **2. Core: Aufträge & Tasks** | ✅ GPS-Auto, Workflows | ✅ order.rs | ✅ OrderRepository | ✅ order.rs | ✅ orders.rs | ✅ | **PRODUCTION** |
| **3. Core: Mitarbeiter** | ✅ Zeiterfassung, GPS | ✅ workforce.rs | ✅ WorkerRepo, WorkLog, Location | ✅ worker.rs, work_log.rs, worker_location.rs | ✅ workforce.rs | ⚠️ partial | **NEARLY DONE** |
| **4. Core: Geräte** | ✅ Wartungsintervalle | ✅ equipment.rs | ✅ EquipmentRepository | ✅ equipment.rs | ✅ equipment.rs | ✅ | **PRODUCTION** |
| **5. Compliance: Pflanzenschutz** | ✅ Wartezeiten, Flächenberechnung | ✅ plant_protection.rs | ✅ PlantProtectionRecordRepo | ✅ plant_protection_record.rs | ✅ compliance.rs | ⚠️ partial | **NEARLY DONE** |
| **6. Compliance: GAP/Bio** | ✅ AuditLogs, Checklisten | ✅ compliance.rs | ✅ ComplianceChecklistRepo, AuditLogRepo | ✅ compliance.rs, audit_log.rs | ✅ compliance.rs | ⚠️ partial | **NEARLY DONE** |
| **7. Spezialkulturen: Weinbau** | ✅ DOC, Qualitätsgrade | ✅ vineyard.rs | ✅ VineyardRepo, KelterDeliveryRepo | ✅ vineyard.rs, kelter_delivery.rs | ✅ specialized.rs | ⚠️ partial | **NEARLY DONE** |
| **8. Spezialkulturen: Oliven** | ✅ Öl-Grad | ✅ olive.rs | ✅ OliveGroveRepo, OliveOilRecordRepo | ✅ olive_grove.rs, olive_oil_record.rs | ✅ specialized.rs | ⚠️ partial | **NEARLY DONE** |
| **9. Wasser** | ✅ Quellen, Quoten, Bewässerung | ✅ water.rs | ✅ WaterSource/Usage/QuotaRepo | ✅ water_source.rs, water_usage.rs, water_quota.rs | ✅ water.rs | ⚠️ partial | **NEARLY DONE** |
| **10. Ernte-Logistik** | ✅ Chargen, Kühlkette | ✅ harvest.rs | ✅ HarvestSeason/Lot/Delivery/ColdChain | ✅ harvest_season.rs, harvest_lot.rs, harvest_delivery.rs, cold_chain_log.rs | ✅ harvest.rs | ⚠️ partial | **NEARLY DONE** |
| **11. Viehwirtschaft** | ✅ Behandlungen, Weideplanung | ✅ livestock.rs | ✅ AnimalRepository | ✅ animal.rs | ✅ livestock.rs | ⚠️ partial | **NEARLY DONE** |
| **12. Wetter & Phänologie** | ✅ Stationen, BBCH, Frost | ✅ weather.rs | ✅ WeatherStation/Data/PhenologyRepo | ✅ weather_station.rs, weather_data.rs, phenology_record.rs | ✅ weather.rs | ⚠️ partial | **NEARLY DONE** |
| **13. Finanzen: PAC** | ✅ Anträge, Eco-Schemes | ✅ finance.rs | ✅ PACApplicationRepo | ✅ pac_application.rs | ✅ finance.rs | ⚠️ partial | **NEARLY DONE** |
| **14. Finanzen: Kostenstellen** | ✅ Buchhaltung, Kategorien | ✅ finance.rs | ✅ CostCenterRepo, FinancialRecordRepo | ✅ cost_center.rs, financial_record.rs | ✅ finance.rs | ⚠️ partial | **NEARLY DONE** |
| **15. Reporting/Export** | ✅ Excel, GeoJSON, PAC | ✅ (uses entities) | N/A (uses repos) | N/A | ✅ reporting.rs | ⚠️ | **NEARLY DONE** |
| **16. KI-Analytics** | 🔜 v1.1 "Coming Soon" | ❌ | ❌ | ❌ | ❌ | ❌ | **PLANNED** |

---

## 🎯 Legend
- ✅ = Implementiert & kompiliert
- ⚠️ = Implementiert aber braucht: Integrationstests, API-Endpoints vervollständigen, oder Edge-Cases
- 🔜 = Geplant (nicht auf Landing Page als "fertig" beworben)

---

## 📅 IMPLEMENTATION TIMELINE

### PHASE 1: STABILISIERUNG (Woche 1-2) — *Sofort*
**Ziel:** Alle "Nearly Done" Module zu 100% production-ready machen

| Task | Modul | Aufwand | Beschreibung |
|------|-------|---------|--------------|
| 1.1 | **Mitarbeiter (Workforce)** | 2 Tage | `worker_task_status.rs` API endpoints in `workforce.rs` vervollständigen, `find_latest_by_worker` & `get_latest_locations` in Handler einbauen |
| 1.2 | **Pflanzenschutz** | 1 Tag | `compliance.rs` Handler: `update_applicator_license` endpoint fehlt noch | ✅ DONE |
| 1.3 | **Compliance (GAP/Bio)** | 2 Tage | `audit_log.rs` Handler fehlen komplett; `compliance.rs` braucht `find_by_site` & `find_by_type` Endpoints |
| 1.4 | **Weinbau** | 1 Tag | `specialized.rs` Handler: `Vineyard` CRUD + `KelterDelivery` Endpoints prüfen/vervollständigen |
| 1.5 | **Olivenbau** | 1 Tag | `specialized.rs` Handler: `OliveGrove` + `OliveOilRecord` Endpoints prüfen |
| 1.6 | **Wasser** | 1 Tag | `water.rs` Handler: `WaterSource/Usage/Quota` Endpoints prüfen |
| 1.7 | **Ernte** | 1 Tag | `harvest.rs` Handler: alle 4 Repos (Season, Lot, Delivery, ColdChain) Endpoints prüfen |
| 1.8 | **Viehwirtschaft** | 1 Tag | `livestock.rs` Handler: `add_treatment`, `add_grazing_record` Endpoints fehlen |
| 1.9 | **Wetter** | 1 Tag | `weather.rs` Handler: `PhenologyRecord` Endpoints prüfen |
| 1.10 | **PAC/Finanzen** | 1 Tag | `finance.rs` Handler: `CostCenter`, `FinancialRecord` Endpoints komplettieren |
| 1.11 | **Reporting** | 2 Tage | `reporting.rs`: Excel Export (rust_xlsxwriter), GeoJSON Export, PAC SIP Export testen & dokumentieren |

**Deliverable Phase 1:** Alle 15 Core-Module haben vollständige CRUD-API, OpenAPI-Docs generieren ohne Lücken, `cargo test --workspace` läuft grün.

---

### PHASE 2: QUALITÄT & HÄRTUNG (Woche 3-4)

| Task | Bereich | Aufwand | Beschreibung |
|------|---------|---------|--------------|
| 2.1 | **Integrationstests** | 5 Tage | Für jeden Handler: `crates/api/tests/` erweitern. Mindestens: CRUD + Auth + Pagination + Visibility-Filter Tests. Ziel: 80% Coverage bei Handlers. |
| 2.2 | **PostgreSQL Migration** | 3 Tage | `POSTGRES_MIGRATION_CHECKLIST.md` abarbeiten: fehlende Repos implementieren (`task_data.rs` existiert, `kelter_delivery.rs` existiert - prüfen), `database.rs` alle Repo-Methoden registrieren |
| 2.3 | **Migration Scripts** | 2 Tage | SQL-Migrationen für alle Tabellen finalisieren (`migrations/001_initial_schema.sql` erweitern), Seed-Daten für Demo/Dev |
| 2.4 | **API Docs & OpenAPI** | 1 Tag | Swagger UI prüfen, alle Schemas in `ApiDoc` registriert, Beispiel-Requests/Responses ergänzen |
| 2.5 | **Rate Limiting & Security** | 1 Tag | Governor Config prüfen (120 req/min), Security Headers, CORS, JWT-Refresh-Token Flow testen |
| 2.6 | **Observability** | 2 Tage | Prometheus Metrics (alle Handler instrumentiert), Grafana Dashboards anlegen, Loki Log-Struktur prüfen, Health Checks (`/health`, `/ready`) |
| 2.7 | **Docker & Deployment** | 2 Tage | `docker-compose.yml` für Dev + Prod, Multi-stage Dockerfile optimieren, K8s Manifests (Helm Chart) erstellen |

**Deliverable Phase 2:** Production-ready Release Candidate. `docker compose up -d` startet alles. Swagger UI vollständig. Monitoring funktionsfähig.

---

### PHASE 3: MULTI-TENANT & ENTERPRISE FEATURES (Woche 5-8)

| Task | Feature | Aufwand | Beschreibung |
|------|---------|---------|--------------|
| 3.1 | **Multi-Tenant Isolation Hardening** | 3 Tage | Row-Level Security (RLS) Policies in Postgres für alle Tabellen, Visibility-Filter in Repos gegen DB-Constraints prüfen |
| 3.2 | **Offline-First Sync Engine** | 10 Tage | Client-seitig: SQLite + Sync-Queue (CRDT oder Last-Write-Wins), Server: Sync-Endpoint `/api/sync` mit Conflict-Resolution, Background-Sync-Worker |
| 3.3 | **Advanced RBAC** | 5 Tage | Custom Roles (über `UserRole::Custom(Uuid)`), Permission-Scoping pro Resource, Delegation, API-Key Management |
| 3.4 | **Audit Trail Vollständig** | 3 Tage | `AuditLogRepo` für ALLE Entities automatisch (Trigger oder Middleware), Immutable Logs, Export für Auditoren |
| 3.5 | **Webhooks & Event-System** | 5 Tage | NATS-basiert: `OrderCreated`, `TaskCompleted`, `ComplianceDue`, `WeatherAlert` Events. Webhook-Delivery mit Retry/Dead-Letter |
| 3.6 | **Mobile/PWA Admin UI** | 10 Tage | Leptos Admin-UI: Offline-Cache (Service Worker), Touch-optimiert, Worker-Task-Interface (Start/Pause/Stop Buttons), GPS-Tracking |
| 3.7 | **SIGPAC & Kataster-Import** | 5 Tage | Batch-Import GeoJSON/Shapefile, automatische Flächenberechnung, Duplikat-Erkennung, Validierung gegen LPIS |

---

### PHASE 4: KI & ADVANCED ANALYTICS (Woche 9-14) — *Landing Page: "v1.1"*

| Task | Feature | Aufwand | Beschreibung |
|------|---------|---------|--------------|
| 4.1 | **Erntevorhersage (Yield Prediction)** | 15 Tage | ML-Pipeline: Historische Ernte-Daten + Wetter + Boden + Satellit (Sentinel-2 NDVI) → XGBoost/LightGBM Model → API `/api/v1/predict/yield` |
| 4.2 | **Krankheits-/Schädlings-Frühwarnung** | 10 Tage | Wetter-basierte Modelle (z.B. Rebenperonospora, Olivenfliege), Integration Plant-Protection-Records → Push-Benachrichtigung |
| 4.3 | **Optimale Bewässerungs-Empfehlung** | 10 Tage | ET0-Berechnung (FAO-56) + Bodenfeuchte + Wetter-Prognose → Tägliche Empfehlung `/api/v1/recommend/irrigation` |
| 4.4 | **Düngungs-Optimierung (N-Bilanz)** | 8 Tage | Düngeverordnung (DüV) Konformität, Nmin-Proben + Ernteentzug → bedarfsgerechte Empfehlung |
| 4.5 | **Wirtschaftlichkeits-Analyse (KPI Dashboard)** | 8 Tage | Deckungsbeitrag pro Kultur/Fläche, Maschinenkosten, Arbeitszeit, Benchmarking vs. Branchendurchschnitt |
| 4.6 | **Satelliten-Monitoring (Sentinel-2)** | 10 Tage | NDVI/EVI Time-Series pro Parzelle, Anomalie-Erkennung, Vegetations-Status in Admin-UI visualisieren |
| 4.7 | **Generative Reports (LLM)** | 5 Tage | Natürliche Sprache für PAC-Berichte, GAP-Nachweise, Jahresabschluss → Template + LLM (lokales Ollama oder API) |

---

### PHASE 5: ECOSYSTEM & MARKETPLACE (Monat 4-6)

| Task | Feature | Aufwand | Beschreibung |
|------|---------|---------|--------------|
| 5.1 | **Plugin/Extension System** | 10 Tage | WASM-basierte Plugins für: Zertifizierungs-spezifische Checks, Regionale Gesetze (DE/PT/ES/FR), Maschinen-Hersteller-Integrationen |
| 5.2 | **Marketplace für Module** | 15 Tage | Community kann Module veröffentlichen, Rating, Review, Installation via Admin-UI |
| 5.3 | **IoT Device Registry** | 8 Tage | Wetterstationen, Bodenfeuchtesensoren, Traktor-Telematik (ISOBUS), LoRaWAN Gateway Integration |
| 5.4 | **Multi-Farm / Kooperativen Support** | 8 Tage | Verbund-Management: Gemeinsame Maschinen, Gemeinschaftliche Bewässerung, Kollektive Vermarktung |
| 5.5 | **Carbon Farming / Zertifikate** | 10 Tage | Humus-Aufbau Messung, CO2-Zertifikate Generierung, Verifizierung (Verra/Gold Standard kompatibel) |

---

## 🚀 QUICK WINS (Diese Woche noch machbar)

| # | Task | Datei | Aufwand |
|---|------|-------|---------|
| 1 | `worker_task_status` API Endpoints in `workforce.rs` | `crates/api/src/handlers/workforce.rs` | 2h |
| 2 | `add_treatment` / `add_grazing_record` Endpoints | `crates/api/src/handlers/livestock.rs` | 2h |
| 3 | `ComplianceChecklist` `find_by_site` / `find_by_type` | `crates/api/src/handlers/compliance.rs` | 2h |
| 4 | `AuditLog` Handler (CRUD) | `crates/api/src/handlers/compliance.rs` | 3h |
| 4 | `Vineyard` + `KelterDelivery` Endpoints prüfen | `crates/api/src/handlers/specialized.rs` | 2h |
| 5 | `OliveGrove` + `OliveOilRecord` Endpoints prüfen | `crates/api/src/handlers/specialized.rs` | 2h |
| 6 | Swagger UI: fehlende Schemas in `ApiDoc` registrieren | `crates/api/src/lib.rs` | 1h |

---

## 📋 DEFINITION OF DONE (für jedes Modul)

Ein Modul gilt als **PRODUCTION**, wenn:

- [ ] Domain Entity mit Create/Update DTOs ✅
- [ ] Repository Trait in `domain/src/repositories.rs` ✅
- [ ] PostgreSQL Implementation in `infrastructure/src/postgres/*.rs` ✅
- [ ] Repo in `database.rs` registriert & in `postgres.rs` exportiert ✅
- [ ] API Handler in `api/src/handlers/*.rs` mit allen CRUD + Custom Endpoints ✅
- [ ] Handler in `handlers/mod.rs` registriert & Routes in `configure()` ✅
- [ ] OpenAPI Schemas in `ApiDoc` (lib.rs) registriert ✅
- [ ] Integration Tests in `api/tests/` (CRUD + Auth + Visibility) ✅
- [ ] `cargo test --workspace` läuft grün ✅
- [ ] Swagger UI zeigt alle Endpoints korrekt an ✅
- [ ] Docker Compose startet Service ohne Fehler ✅

---

## 🔗 ABHÄNGIGKEITEN & RISIKEN

| Risiko | Impact | Mitigation |
|--------|--------|------------|
| PostgreSQL Repo-Vervollständigung zieht sich | HIGH | Parallel arbeiten: 2 Devs an verschiedenen Repos, täglich `cargo check` |
| Offline-Sync Komplexität | HIGH | Phase 3 erst starten wenn Phase 1-2 stabil; Spike-Woche für Architecture Decision |
| NATS Event-System Design | MEDIUM | Erst einfache Events, später JetStream Persistence; Schema Registry (JSON Schema) von Anfang an |
| KI-Modelle brauchen Trainingsdaten | HIGH | Synthetische Daten + Open Data (Sentinel, DWD, LPIS) nutzen; Transfer Learning |
| Multi-Tenant RLS in Postgres | MEDIUM | Frühzeitig Spike; Alternative: Application-Level Filtering (bereits implementiert) |

---

## 📈 MEILENSTEINE

| Datum | Meilenstein | Kriterium |
|-------|-------------|-----------|
| **2026-07-21** | **M1: Core Complete** | Alle 15 Module Production-Ready (Phase 1 Done) |
| **2026-08-04** | **M2: Release Candidate** | Tests, Docker, Monitoring, Docs (Phase 2 Done) |
| **2026-08-18** | **M3: Enterprise Ready** | Multi-Tenant, Offline, Webhooks, PWA (Phase 3 Done) |
| **2026-09-15** | **M4: v1.1 KI-Release** | Yield Prediction, Disease Alert, Irrigation AI (Phase 4 Done) |
| **2026-11-01** | **M5: Ecosystem Launch** | Plugin System, Marketplace, IoT, Carbon (Phase 5 Done) |

---

## 👥 TEAM-EMPFEHLUNG

| Rolle | Phase 1-2 | Phase 3-4 | Phase 5 |
|-------|-----------|-----------|---------|
| **Backend (Rust/Postgres)** | 2 | 2 | 1 |
| **Frontend (Leptos/TS)** | 1 | 2 | 2 |
| **DevOps/Infra** | 0.5 | 1 | 1 |
| **ML/Data Engineering** | 0 | 1 | 2 |
| **QA/Testing** | 0.5 | 1 | 1 |

---

## 📝 NÄCHSTE SCHRITTE (HEUTE)

1. `cd /home/jens/RustroverProjects/agrocore-rs`
2. Quick Wins abarbeiten (siehe Tabelle oben) - ca. 1 Tag
3. Phase 1 Tasks als GitHub Issues anlegen mit Labels `phase-1`, `backend`
4. `POSTGRES_MIGRATION_CHECKLIST.md` final abarbeiten
5. `cargo test --workspace` als CI-Gate etablieren

---

*Dokument erstellt auf Basis des Code-Stands vom 2026-07-14. Regelmäßig aktualisieren bei Fortschritt.*