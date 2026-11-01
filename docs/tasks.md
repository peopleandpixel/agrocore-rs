# AgroCore-RS Open Tasks

Last updated: 2026-11-01 (v0.71.0)

Open work only. Completed items have been removed; the history lives in the CHANGELOG.
Ordered by priority, within a priority by dependency.

Legend: **P0** blocks operation · **P1** MVP · **P2** important · **P3** convenience ·
**P4** future.

---

## Task 0 — Security, completeness and performance (P1)

### Block C — Information leak and abusability

- [x] **C2 — `/metrics/db` and `/metrics/business` without authentication** — done (v0.65.0): Both handlers use `AuthExtractor` with `require_admin()`.

### Block E — Missing validation

- [x] **E1 — POST/PUT routes without `.validate()`** — done (v0.65.0): 99 `.validate()` calls across all handler files, 104 DTOs derive `validator::Validate`.

### Block H — AdminUI does not reach large parts of the backend

- [x] **H9 — API functions for uncovered endpoints** — done (v0.69.0): Added API functions for 10 previously unreachable API paths: settings groups, weather current-with-fallback, IoT telemetry, and type system admin endpoints. All `route_inventory_tests` now pass.
- [x] **H9 — UI pages for ~80 routes** — done (v0.70.0): All UI components are fully functional and call API functions:
  - **Backup** (491 lines): Config management, manual backup creation, list with delete/restore actions.
  - **Water** (546 lines): Sources, usage, quotas — full CRUD with forms and tables.
  - **Harvest** (781 lines): Seasons, lots, deliveries, cold chain — full CRUD with forms and tables.
  - **Agriculture Special** (982 lines): Olive groves, olive oil records, vineyards, kelter deliveries — full CRUD.
  - **IoT** (423 lines): Device grid, detail modal with telemetry, command sending.
  - **Calculators** (710 lines): 12 calculator tools (nutrition, water rate, material, tree crown, forage, nitrogen, difficulty, profitability, harvest estimation, weather, fertilizer).
  - **Compliance** (210 lines): Checklists, plant protection records, PAC/SIP export.
  - Sidebar navigation added for `/water`, `/harvest`, `/agriculture` (were missing).
- [x] **H12 — Compliance and weather areas** — done (v0.71.0): Added `create_fertilizer_record` POST endpoint with validation; fertilizer records section in Compliance page with table, create modal, and delete. Weather station management: added `update_weather_station` and `delete_weather_station` API functions; create station modal and delete buttons in Weather page.
- [x] **H13 — Further unused areas** — done (v0.72.0): All H13 areas are already fully integrated in the UI:
  - **Profitability, Workflow, Material, Difficulty Surcharge, Tree Crown Volume, Nitrogen Demand** — all covered by `calculators.rs` (711 lines, 16 calculator tools) at `/calculators` with Sidebar navigation.
  - **Stations** — IoT devices covered by `iot.rs` (423 lines) at `/iot` with Sidebar navigation; Weather stations covered by `weather.rs` at `/weather` with Sidebar navigation.
  - **Sources** — Water sources covered by `water.rs` (546 lines, full CRUD) at `/water` with Sidebar navigation.
  - All API functions exist in `api.rs` for all H13 endpoints.
  - All components are routed in `lib.rs` and linked in the sidebar.

### Block I — Performance

- [x] **I5 — LPIS HTTP cache is implemented but never enabled** — done (v0.65.0): `create_default_registry()` builds an `LpisCache` with memory backend and 1-hour TTL, then registers every provider with `.with_cache()`.
- [ ] **I13 — Two queries per list request (`COUNT` + `SELECT`)** — Consistently in about 25 repos (`site.rs:98+104`, `user.rs:132+140`, `equipment.rs:190+214`, `customer.rs:233+243`, `weather_data.rs:61+67`). The `COUNT(*)` runs with the same filter and scales linearly. Keyset paging instead of `OFFSET`, `total` via `COUNT(*) OVER()` only if genuinely needed.
- [ ] **I14 — Missing transactions on multi-write** — `order.rs:139-154` (INSERT + audit log), `order.rs:170-208` (SELECT + UPDATE + audit log, not atomic, TOCTOU on `old_order`), `handlers/workforce.rs:446-560` (per order `update()` + `publish()` + `find_by_user_id()` + `work_log_repo().create()`), `handlers/orders.rs:328-337` (follow-up orders committed individually in a loop). Add a `&mut Transaction` variant to the repo updates; make the batch operations in `import_service.rs:56-95,408-493` transactional too, plus `UNNEST` batch insert.
- [ ] **I16 — `spawn_blocking` partially done** — Excel reports are wrapped. Remaining: **LPIS parsing**, **geometry service**, **checksums**, **geodesy** — still need `spawn_blocking`.

### Block J — Unwired code and missing tests

- [x] **J17 — `iot.rs` bypasses the repo layer** — done (v0.68.0): Created `IotDeviceRepository` trait in domain crate, PostgreSQL implementation in infrastructure crate, and refactored all 7 raw SQL queries in `iot.rs` handler to use the repository pattern. Telemetry ingestion keeps direct sqlx for high-throughput batch inserts (documented).
- [x] **J18 — The auth round-trip is untested** — done (v0.67.0): Added `auth_roundtrip_tests.rs` with tests for login, refresh, and logout route registration plus `find_by_refresh_token` coverage.
- [ ] **J19 — No migration schema assertion test (column lists)** — Three tests check table existence, tenant reference and readability. The part that compares the **column lists** of the repos against `information_schema` is still missing — it would find schema drift automatically. (Deferred — requires database connection)
- [ ] **J20a — The test fixture — 4 tests still red** — `test_database_migrations_applied` expects table `spatial_properties`; `test_site_crud_operations` and `test_updated_at_trigger` fail with column count mismatch; `test_tenant_creation_and_isolation` has slug unique constraint issue. (Deferred — requires database connection with PostGIS)

---

## P0 — Critical (deferred integration tests)

### Backup integration tests

- [ ] Integration tests against real cloud instances (S3/MinIO, Azure Blob, GCS) in CI
- [ ] Integration tests against real SFTP and WebDAV servers
- [ ] Secure the NATS progress events (0–100 %) with integration tests

---

## P1 — MVP

### UI Quality

- [ ] **M4 — `agrocore-domain` does not build without its `sqlx` feature** — Either gate the `sqlx::postgres` imports in `domain` on the feature that actually provides them, or make `sqlx` non-optional.
- [ ] **M6 — 42 fetches in 20 files still discard their error** — `.await.ok()` pattern turns failed fetches into `None`. `orders.rs` is the template with loading/error/empty states. Each conversion is ~40 lines.
- [ ] **M7 — `ErrorBoundary` is never mounted** — Mount it around the routed content so a failed route shows something.
- [ ] **M8 — German strings bypass i18n** — Check for remaining hardcoded German strings in UI components.

### Build optimizations

- [ ] **O2 — Tailwind is compiled in the browser** — Build Tailwind ahead of time from a content scan of the Leptos views, so the 407 KB JIT runtime leaves the bundle. The obstacle is that `view!` macros construct class strings at runtime.
- [ ] **O3 — wasm-opt root cause open** — Establish whether Binaryen `version_116` can process the module at all. If not, either pin a newer Binaryen through `[tools].wasm_opt` or move to a Trunk release that pins one which can.

### Customers & sales

- [ ] CSA management: subscription boxes, delivery planning
- [ ] Wholesale orders: tiered prices, delivery planning
- [ ] Direct sales: online shop, payment processing

### Livestock

- [ ] Breeding records: heat detection, AI, calving, mating
- [ ] Feed intake tracking: ration, waste, feed value
- [ ] Milk production tracking: daily production, butterfat, protein
- [ ] Movement documentation: births, deaths, purchases, sales
- [ ] Pasture management: area rotation, rest periods, animal location

### Finances

- [ ] Accounting integration: QuickBooks, Xero, double-entry bookkeeping
- [ ] Budgeting and forecasts: planned vs. actual
- [ ] Field costing: input vs. output values
- [ ] Revenue tracking per crop
- [ ] Cash flow management: plan liabilities and payment receipts
- [ ] Tax reporting: Schedule F, depreciation, fertilizer cost deduction
- [ ] Input cost tracking: seed, fertilizer, chemicals, fuel, QR scans
- [ ] Integrate depreciation into the financial report — the calculation (`depreciation.rs`) and the monthly timer run, but `/financial/reports` does not evaluate them yet

### Catalog import

- [ ] `scripts/import_catalog.py`: generate complete catalogs as CSV and import them into `varieties`/`breeds`. Sources: VIVC grape varieties (>12k), olive DB (>260), FAO terraces. Prepare lazy-load search for the Admin UI

---

## P2 — Important

### Monitoring & observability

- [ ] Query duration monitoring (sqlx middleware or `sqlx-metrics`)
- [ ] Pool utilization: active and idle connections
- [ ] Slow query detection with logging
- [ ] Span attributes: tenant ID, user ID, operation type
- [ ] Standardize tracing across all service boundaries

### Business metrics

- [ ] Active devices per tenant
- [ ] Telemetry messages transmitted per hour
- [ ] Successfully processed import files

### Mobile first

- [ ] Offline-first mobile app with sync when connectivity is restored
- [ ] Barcode and QR code scanner: equipment, inventory, field ID
- [ ] GPS field boundaries (boundary recording)
- [ ] Mobile time tracking: clock-in/clock-out with GPS
- [ ] Voice-to-text notes
- [ ] Photo documentation: attachments to tasks, problems, inspections
- [ ] Push notifications: weather warnings, task reminders
- [ ] Real-time field activity recording: planting, spraying, harvesting
- [ ] Harvest data import from combine harvesters
- [ ] Drone and UAV integration: NDVI images

---

## P3 — Convenience

### Precision agriculture

- [ ] Variable fertilization plans: VRA for seeders, sprayers, spreaders
- [ ] GPS auto-steer: integration with steering systems
- [ ] Drone spraying integration: management and control
- [ ] Automated irrigation control via IoT valves

### Sustainability & compliance

- [ ] Carbon credit tracking: measure and report CO₂ sequestration
- [ ] Water usage monitoring: irrigation efficiency, regulatory compliance
- [ ] Chemical application logs: REI, restricted use
- [ ] Organic certification: input tracking, buffer zones, inspections
- [ ] Sustainability metrics: soil health, biodiversity, fertilizer reduction

### Business features

- [ ] Multi-farm management: holdings, leases, tenants
- [ ] Contract farming: producer contracts, quality premiums
- [ ] Labor management: wages, certifications, planning
- [ ] Equipment sharing: rental marketplace between farms
- [ ] Insurance integration: damage documentation, risk assessment

---

## P4 — Future

### AI analytics

- [ ] Define requirements for yield forecasts
- [ ] Define requirements for early warning systems for disease and pests
- [ ] Define requirements for irrigation, fertilization, KPI, satellite monitoring and generative reporting
- [ ] Computer vision: plant diseases, weed detection
- [ ] AI advisory assistant: chat interface for agronomic questions
- [ ] Generative AI for operational planning
- [ ] Implementation only after the production modules and the sync foundations are stabilized

### AI & robotics

- [ ] Robot weeding: autonomous control and monitoring
- [ ] Autonomous machinery: fleet management for self-driving tractors

### Advanced technologies

- [ ] Augmented reality: field data overlay on the live camera
- [ ] Digital twin: virtual farm model for scenario planning
- [ ] Blockchain traceability: supply chain transparency
