# Changelog

Alle Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

## [0.5.1] - 2026-07-25

### Added
- **RLS Policies:** Row-Level Security policies for all 34+ tenant-scoped tables (2 migration files)
- **Offline-First Sync Engine:** Core domain entities for offline-first synchronization:
  - Vector clocks for causal ordering
  - Sync mutations (create/update/delete) with conflict detection
  - Conflict resolution strategies (LastWriterWins, ServerWins, ClientWins, Merge, Manual)
  - Sync batches, pull/push requests, full sync operations
  - Client sync state tracking
- **Sync API Endpoints:** POST /api/v1/sync/push, /api/v1/sync/pull, /api/v1/sync (full sync)
- **Sync Configuration:** Configurable batch size, conflict resolution, CRDT support, sync intervals

### Fixed
- **Clippy Clean:** Fixed collapsible_if warnings in site.rs boundary encoding

### Changed
- **Version bump:** 0.5.0 → 0.5.1
- **All Quality Gates:** `cargo check`, `cargo test` (103+ tests), `cargo clippy` all pass clean

### Technical
- Domain entities all compile with `ToSchema` derives
- TenantId wrapper properly implemented across all services
- API handlers use correct TenantId wrapping
- OpenAPI/Swagger docs generate without errors
- Integration test infrastructure with testcontainers

## [0.5.0] - 2026-07-25

### Added
- **Complete PostgreSQL Schema:** All 34 tables with 15 migration files covering core entities (sites, orders, equipment, users, tenants), compliance (plant protection, checklists, audit logs, applicator licenses, fertilizer), specialized cultures (vineyard, olive), harvest logistics, livestock, water management, weather/phenology, finance (PAC, cost centers, financial records)
- **Full Repository Layer:** 35+ PostgreSQL repository implementations with all CRUD operations, pagination, and visibility filtering
- **Docker Stack:** Multi-stage Dockerfiles for API, services, and admin-ui; docker-compose.yml with health checks for postgres, nats, api, reporting, weather, admin-ui
- **Observability:** Health check endpoint (`/health`), Prometheus metrics, structured logging with tracing
- **Migration Files:** 8 new migrations (olive, vineyard, water, fertilizer/compliance, livestock, task_data, core additional tables, plant protection/worker status, tasks)
- **Admin UI:** Leptos 0.8 + Nginx static serving with SPA routing

### Fixed
- **Task Data Schema:** Fixed task_data table to match domain entity (added order_id, site_id, description, paused_at, resume_at, duration_minutes, machine_id, machine_hours, cost_center_id, area_covered, materials_used, observations, gps_track, photo_urls)
- **All Missing Tables:** Created migrations for audit_logs, cost_centers, financial_records, pac_applications, plant_protection_records, worker_task_statuses, tasks
- **PostgreSQL Repositories:** All 35+ repositories now compile and registered in database.rs

### Changed
- **Version bump:** 0.4.0 → 0.5.0
- **All Quality Gates:** `cargo check`, `cargo test` (103+ tests), `cargo clippy` all pass clean
- **Docker Compose:** Updated to v3.8 with proper health checks, dependency ordering, non-root users

### Technical
- Domain entities all compile with `ToSchema` derives
- TenantId wrapper properly implemented across all services
- API handlers use correct TenantId wrapping
- OpenAPI/Swagger docs generate without errors
- Integration test infrastructure with testcontainers

## [0.4.0] - 2026-07-24

### Fixed
- **Clippy Clean:** All clippy warnings resolved across workspace (useless_conversion, unnecessary_cast, map_flatten)
- **Domain Tests:** Fixed TenantId wrapper usage in all domain tests (spatial, workflow, user, site, order, spatial)
- **API DTOs:** Removed unnecessary `.into()` calls on Uuid/Option fields
- **Weather DTO:** Replaced `map().flatten()` with `and_then()` for Option serialization
- **Plant Protection DTO:** Removed unnecessary `as u32` casts

### Changed
- **Version bump:** 0.3.3 → 0.4.0
- **Integration Tests:** All 21 tests pass (10 unit + 3 api + 3 auth + 1 health + 2 rate_limit + 2 reporting)
- **Clean Build:** `cargo check`, `cargo test`, `cargo clippy` all pass without warnings (except proc-macro-error2 future incompat from Leptos 0.8)

### Technical
- Core domain entities all compile with `ToSchema` derives
- TenantId wrapper properly implemented in reporting/weather services
- API handlers use correct TenantId wrapping
- OpenAPI/Swagger docs generate without errors

## [Unreleased]

## [0.3.3] - 2026-07-18

### Added
- **Applicator License API:** Complete REST API for Pflanzenschutz compliance (applicator licenses) in the compliance module.
  - `GET /api/v1/compliance/applicator-licenses` - List all applicator licenses
  - `GET /api/v1/compliance/applicator-licenses/{id}` - Get specific applicator license
  - `POST /api/v1/compliance/applicator-licenses` - Create applicator license
  - `PUT /api/v1/compliance/applicator-licenses/{id}` - Update applicator license
  - `DELETE /api/v1/compliance/applicator-licenses/{id}` - Delete applicator license
- **Admin-UI:** Complete ApplicatorLicense types and API client in `api.rs` (ApplicatorLicenseDto, LicenseTypeDto, Create/Update DTOs)
- **DTOs:** Complete bidirectional conversion between domain and API types for ApplicatorLicense, LicenseType
- **Compliance Checklist & Fertilizer Record DTOs:** Added Create/Update DTOs for compliance checklists and fertilizer records with proper From conversions

### Changed
- **Version bump:** 0.3.2 → 0.3.3

### Technical
- Full OpenAPI/Swagger documentation for all new endpoints via utoipa
- Consistent error handling with `ApiError` types
- Proper tenant isolation on all new endpoints
- Migration file for applicator_licenses table: `2026071801_applicator_licenses.sql`
- ChecklistItem now implements utoipa::ToSchema for OpenAPI generation

## [0.3.2] - 2026-07-18

### Added
- **Worker Task Status API:** Complete REST API for multi-worker task status management in the workforce module.
  - `GET /api/v1/workforce/tasks/{id}/status` - List all worker statuses for a task
  - `GET /api/v1/workforce/tasks/{id}/status/{worker_id}` - Get specific worker's status for a task
  - `POST /api/v1/workforce/tasks/{id}/status` - Create worker task status
  - `PUT /api/v1/workforce/tasks/{id}/status/{worker_id}` - Update worker task status
  - `GET /api/v1/workforce/tasks/{id}/status/aggregate` - Get aggregated status using domain logic
- **Admin-UI:** Complete WorkerTaskStatus types and API client in `api.rs` (WorkerTaskStatusDto, WorkerTaskStatusTypeDto, Create/Update DTOs, Aggregate DTO, pagination)
- **DTOs:** Complete bidirectional conversion between domain and API types for WorkerTaskStatus

### Changed
- **Version bump:** 0.3.1 → 0.3.2

### Technical
- Full OpenAPI/Swagger documentation for all new endpoints via utoipa
- Consistent error handling with `ApiError` types
- Proper tenant isolation on all new endpoints

## [0.3.1] - 2026-07-17

### Fixed
- **Admin-UI:** Komplette Überarbeitung und Modernisierung der `sites.rs` Komponente (Leptos 0.7/0.8 API), Behebung von Kompilierfehlern und Typ-Inferenz-Problemen.
- **Entwicklungs-Stack:** `dev.sh` wartet nun explizit auf die Einsatzbereitschaft von PostgreSQL (`pg_isready`), um Startfehler zu vermeiden.
- **Infrastruktur:** Implementierung von Retry-Mechanismen für Datenbank- (PostgreSQL) und Messaging-Verbindungen (NATS), um die Robustheit beim Systemstart zu erhöhen.

## [0.3.0] - 2026-07-15

### Added
- **Benutzerverwaltung:** Vollständige CRUD-Funktionalität für Benutzer inklusive Rollenübersicht in der Admin-UI.
- **Audit-Log UI:** Neuer Bereich in der Admin-UI zur Anzeige der System-Audit-Logs (nur für Administratoren).
- **Navigation:** Konsistente Sidebar-Navigation mit Zugriff auf alle Systembereiche (Flächen, Equipment, Aufträge, Livestock, Finanzen, Compliance).

### Changed
- **Modus-Umschalter:** Der Wechsel zwischen "Einfachem" und "Normalem" Modus wurde als zentraler Switch in die Sidebar verschoben.
- **Flächen-Editor:** Verbesserte Kartenansicht beim Anlegen von Flächen mit größerem Dialogfenster (90% Viewport-Höhe) und wiederhergestellten Zeichenwerkzeugen.
- **RBAC:** Menüpunkte und Aktionen in der Admin-UI werden nun basierend auf den Benutzerrollen (Admin, Manager, Worker) gefiltert.

### Fixed
- **Infrastruktur:** Fehlende Imports in der PostgreSQL-Implementierung (`UpdateWorkLogDto`) behoben.
- **Admin-UI:** Diverse JavaScript-Fixes in der Leaflet-Integration zur Vermeidung von Initialisierungsfehlern.

## [0.2.0] - 2026-07-15

### Added
- Umfassendes Auditing-System: Jede Erstellung, Änderung und Löschung von Kern-Entitäten (Orders, PAC-Applications, Worker) wird nun automatisch protokolliert.
- Historien-Funktion mit Diff: Audit-Logs speichern den alten und neuen Zustand der Daten als JSON, was einen detaillierten Vergleich ermöglicht.
- `PgAuditLogRepo`: Vollständige Implementierung für persistente Speicherung und Abfrage von Audit-Einträgen.

### Changed
- Strikte Tenant-Isolation: Alle Datenbankabfragen validieren nun konsequent die `tenant_id`.
- RBAC-Erweiterung: Sichtbarkeitsprüfungen (`find_by_id_visible`) in Repositories berücksichtigen nun Benutzerrollen (Admin, Manager, Worker).
- Repository-Updates: `PgOrderRepo`, `PgPACApplicationRepo` und `PgWorkerRepo` nutzen nun das neue Auditing-System.
- **Migration von MongoDB zu PostgreSQL abgeschlossen.**
- Sämtliche Services (`api`, `reporting`, `weather`, `geometry`, `asset-registry`) nutzen nun PostgreSQL als primäre Datenbank.
- Domänen-Modelle von MongoDB-spezifischen Attributen und Logiken bereinigt.
- `VisibilityAwareEntity` Trait vereinfacht (keine MongoDB-Filter mehr).
- Docker-Compose und Kubernetes Manifeste auf PostgreSQL/PostGIS umgestellt.
- Entwicklungs-Skripte (`dev.sh`, `dev-start.sh`) für PostgreSQL angepasst.

### Removed
- MongoDB-Container und zugehörige Abhängigkeiten aus dem gesamten Projekt entfernt.
- `mongodb` und `bson` Crates aus den Abhängigkeiten entfernt.
- Veraltete WKT-Konvertierungslogik in den Repositories.
