# Agrocore-RS Open Tasks

Last updated: 2026-08-09

Tasks are ordered by priority, then by estimated implementation time. Completed work is omitted; a module with no remaining work is marked complete for reference.

## Module 16 — IoT & Messaging

**Status:** Production · **Priority:** Complete

PostgreSQL persistence, tenant-scoped CRUD, route registration, OpenAPI exposure, and IoT route integration coverage are complete.

## Module 2 — Core: Aufträge & Tasks

**Status:** Production · **Priority:** Complete

TaskData update/delete persistence, tenant scoping, migration coverage, and task-route integration coverage are complete.

## Module 7 — Spezialkulturen: Weinbau

**Status:** Production · **Priority:** Complete

Vineyard site filtering, CRUD routes, migration compatibility, and route integration coverage are complete.

## Module 9 — Wasser

**Status:** Production · **Priority:** P1 · **Estimated effort:** 1 day

Water-source and water-quota repository filtering, CRUD operations, tenant scoping, pagination, and quota balance initialization are complete.

## Module 12 — Wetter & Phänologie

**Status:** Production · **Priority:** P1 · **Estimated effort:** 1 day

Weather-data and phenology-record tenant-scoped update/delete persistence is complete, with existing CRUD and visibility coverage passing.

## Module 14 — Finanzen: Kostenstellen

**Status:** Production · **Priority:** P1 · **Estimated effort:** 1 day

Cost-center and financial-record update/delete persistence, cost-center filtering, pagination, and tenant scoping are complete; the workspace integration suite passes.

## Module 3 — Core: Mitarbeiter

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 1 day

- Add full integration coverage for workforce CRUD, worker-task status, location history, authorization, pagination, and tenant visibility.

## Module 5 — Compliance: Pflanzenschutz

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 0.5 day

- Add integration coverage for plant-protection CRUD, applicator-license updates, authorization, and tenant visibility.

## Module 6 — Compliance: GAP/Bio

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 0.5 day

- Add integration coverage for compliance checklists and audit logs, including site/type filters and tenant visibility.

## Module 8 — Spezialkulturen: Oliven

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 0.5 day

- Add integration coverage for olive groves and olive-oil records, including authorization and tenant visibility.

## Module 10 — Ernte-Logistik

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 1 day

- Add integration coverage for harvest seasons, lots, deliveries, and cold-chain logs.
- Include CRUD, authorization, pagination, and tenant-visibility cases.

## Module 11 — Viehwirtschaft

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 0.5 day

- Add integration coverage for animals, treatments, grazing records, authorization, and tenant visibility.

## Module 13 — Finanzen: PAC

**Status:** Nearly done · **Priority:** P2 · **Estimated effort:** 0.5 day

- Add integration coverage for PAC applications, including CRUD, authorization, pagination, and tenant visibility.

## Module 15 — Reporting/Export

**Status:** Production · **Priority:** P2 · **Estimated effort:** 0.5 day

- Verify Excel, GeoJSON, and PAC/SIP exports in an end-to-end test environment.
- Document supported export formats and failure behavior.

## Module 1 — Core: Flächen & Parzellen

**Status:** Production · **Priority:** P3 · **Estimated effort:** 0.5 day

- Add or maintain end-to-end coverage for site CRUD, SIGPAC import, PostGIS geometry, authorization, and tenant visibility.

## Module 4 — Core: Geräte

**Status:** Production · **Priority:** P3 · **Estimated effort:** 0.5 day

- Add or maintain end-to-end coverage for equipment CRUD, maintenance data, authorization, and tenant visibility.

## Module 17 — KI-Analytics

**Status:** Planned · **Priority:** P4 · **Estimated effort:** 8–15 days per feature

- Define the data and evaluation requirements for yield prediction.
- Define disease and pest early-warning models.
- Define irrigation, fertilization, KPI, satellite-monitoring, and generative-reporting features.
- Implement features only after the production modules and sync foundation are stable.
