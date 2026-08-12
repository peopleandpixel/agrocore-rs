# AgroCore-RS
## Farm Operations Platform — Product Flyer

---

### Version 0.8.3 | Built in Rust | GPL-3.0 License

**AgroCore-RS** is a unified farm operations platform that consolidates field management, task planning, workforce coordination, IoT sensor integration, compliance tracking, harvest logistics, livestock management, and financial reporting into a single, local-first system.

---

## THE PROBLEM

Modern farms use 5–15 different disconnected tools:
- Spreadsheet-based field records
- Separate apps for weather, tasks, equipment, and livestock
- Manual compliance reporting (GAP, Organic, PAC-SIP)
- No real-time visibility into ongoing work
- IoT data trapped in proprietary dashboards
- Data locked in cloud silos with recurring subscription fees

Farmers lose 10–20 hours per week to tool-switching, manual data entry, and fragmented reporting. Compliance audits become a nightmare. Operational decisions are made on stale or incomplete data.

---

## THE SOLUTION: AGROCORE-RS

AgroCore-RS unifies all farm operations into a single system that runs **locally by default** — no mandatory cloud dependency. Deploy on-premise, on a server, or hybrid. Your data stays yours.

---

## KEY FEATURES

### 1. Integrated Farm Management
- **Sites & Parcels** — Polygon boundaries, area calculation (PostGIS), multi-layer properties
- **Orders & Tasks** — Recurring orders, task assignment, execution policies (Manual/AutoPresence/Scheduled), worklogs
- **Workforce** — Worker profiles, location tracking (GPS), work hour logging, task status per worker
- **Equipment** — Fleet management, service intervals, cost tracking, assignment to workers/sites

### 2. Real-Time IoT Integration
- **MQTT + NATS** — Bidirectional communication (NATS JetStream + MQTT 5 bridge)
- **Home Assistant Auto-Discovery** — Zero-config integration with HA (sensors, binary sensors, buttons, number inputs)
- **Device Registry** — Full CRUD with capabilities (Temperature, Humidity, Soil Moisture, GPS, Battery, Signal, Actuators)
- **Telemetry & Commands** — Historical telemetry retrieval, real-time device commands, firmware updates over-the-air

### 3. Multi-Country LPIS Integration
- **8 Country Providers**: Spain (SIGPAC), Portugal (iLPIS), France (RPG), Italy (SIAN), Netherlands (BRP), Germany (LPIS), Poland (LPIS), Austria (INVEKOS)
- **Official Parcel Validation** — Validate imported field boundaries against government LPIS data
- **Bulk Import** — SIGPAC bulk import of ~25M parcels across 15 Spanish regions
- **GeoJSON & Shapefile Import** — With LPIS-backed validation (area, boundary, usage code)
- **Configurable Providers** — Per-country settings (base URL, timeout, caching, rate limits, enable/disable)

### 4. Compliance & Certification
- **Checklist Management** — GAP, Organic, GlobalGAP, HACCP checklist templates
- **Audit Trail** — Complete change history with user attribution
- **Plant Protection Records** — Pesticide application tracking with applicator licenses
- **Fertilizer Records** — Düngemittel application with compliance reporting
- **PAC Applications** — EU agricultural fund application management with status tracking

### 5. Harvest Logistics
- **Harvest Seasons** — Multi-season planning with timelines
- **Lots** — Batch tracking with quality grading
- **Deliveries** — Third-party delivery management with recipient tracking
- **Cold Chain Monitoring** — Temperature logging during transport/storage

### 6. Livestock Management
- **Animal Registry** — Species, breed, birth date, health records, status tracking
- **Treatment Records** — Vaccination, medication, veterinary interventions
- **Grazing Records** — Pasture assignment, duration tracking
- **Veterinary Reports** — Export-ready Excel reports for veterinarians

### 7. Financial Management
- **Cost Centers** — Direct/Indirect/Overhead/Project cost center allocation
- **Financial Records** — Income/Expense/Transfer tracking with cost center linking
- **PAC Reporting** — EU subsidy application with export (PAC-SIP format)

### 8. Weather & Phenology
- **Weather Stations** — Local station management with data ingestion
- **Weather Data** — Temperature, precipitation, humidity, wind measurements
- **Phenology Tracking** — BBCH growth stage observation and prediction
- **Harvest Prediction** — Algorithmic harvest date estimation based on BBCH development and temperature accumulation

### 9. Reporting & Export
- **Excel Export** — Orders, sites, harvest data, PAC-SIP, veterinary reports
- **GeoJSON Export** — Field boundaries and spatial data
- **Asynchronous Processing** — NATS-based worker architecture for report generation

### 10. Specialized Crop Support
- **Vineyard Management** — Vineyard parcels, kelter delivery tracking, quality grades
- **Olive Grove Management** — Olive grove parcels, olive oil records, oil quality grading
- **Crop Calculations** — Material amount calculation (tank volumes, sprayer calibration), water rate calculation (L/ha)

### 11. Offline-First Architecture
- **SQLite Sync Engine** — Full offline capability for field devices
- **Conflict Resolution** — Automatic merge with server-side conflict detection
- **Data Resilience** — No data loss during connectivity outages

### 12. Security & Privacy
- **JWT Authentication** — 30-minute access tokens, 7-day refresh tokens
- **Argon2 Password Hashing** — Industry-standard password security
- **Multi-Tenant Isolation** — PostgreSQL Row-Level Security (RLS) for complete data separation
- **Role-Based Access Control** — 4 roles (admin, manager, worker, viewer) with granular permissions
- **Rate Limiting** — 120 requests/minute per IP with burst handling
- **Security Headers** — HSTS, CSP, X-Frame-Options, X-Content-Type-Options

---

## COMPETITIVE Advantages

### 🏆 LOCAL-FIRST BY DEFAULT
Unlike FarmLogs, Climate FieldView, John Deere Operations Center, or Granular — AgroCore-RS runs on-premise by default. No mandatory cloud subscription. Your data never leaves your network unless you choose to sync.

### 🚀 PERFORMANCE & RELIABILITY
- **Rust 2024** — Memory-safe, zero-GC, high-performance backend
- **PostgreSQL + PostGIS** — Battle-tested spatial database
- **NATS JetStream** — High-throughput, persistent event streaming
- Sub-50ms API response times under load

### 🌍 TRUE multi-country LPIS
No competitor offers integrated validation against 8 national parcel systems (SIGPAC, iLPIS, RPG, SIAN, BRP, German LPIS, Polish LPIS, INVEKOS). FarmLogs and Climate FieldView only cover the US market.

### 🔌 HOME ASSISTANT INTEGRATION
Built-in MQTT + Home Assistant auto-discovery — the only farm platform that integrates natively with smart farm IoT ecosystems. Sensors appear automatically in HA dashboards. No other agricultural software offers this.

### 🔧 FULL OFFLINE CAPABILITY
While competitors require internet connectivity, AgroCore-RS provides full offline mode via SQLite sync engine. Field workers can operate completely disconnected from the server.

### 💰 NO PER-USER MONTHLY FEES
Open-source GPL license — deploy for a single flat cost. No per-user subscriptions, no data extraction fees, no per-acre charges.

### 🛠️ EXTENSIBLE ARCHITECTURE
- Plugin system for new LPIS providers
- Webhook system for external integrations
- Event-driven architecture (NATS)
- Full OpenAPI 3.0 spec at `/api-docs/openapi.json`

### 📱 PRIVAT-LICENSED ADMIN UI
Progressive Web App (PWA) admin interface — works on mobile, tablet, and desktop. No app store required.

---

## TECHNICAL SPECIFICATIONS

### Backend
- **Language:** Rust 2024 Edition
- **Web Framework:** Actix Web 4
- **Database:** PostgreSQL 16 + PostGIS 3.4
- **Messaging:** NATS 2.10 (JetStream) + MQTT 5 (rumqttc 0.25)
- **Authentication:** JWT (jsonwebtoken 10) + Argon2
- **Rate Limiting:** actix-governor (120 req/min)
- **Caching:** moka (Memory) + Redis
- **Containerization:** Docker + Docker Compose
- **Orchestration:** Kubernetes-ready (Helm charts)
- **Monitoring:** Prometheus + Grafana + Loki/Promtail
- **Tracing:** OpenTelemetry via tracing-actix-web

### Frontend
- **Framework:** Leptos 0.8 (WASM-compiled SPA)
- **Build Tool:** Trunk
- **PWA Support:** Full offline caching via Service Workers
- **Mobile:** Responsive design, installable as PWA on iOS/Android

### DevOps
- **CI/CD:** GitHub Actions + GitLab CI
- **Quality Gates:** cargo fmt, check, test, clippy (-D warnings)
- **Migrations:** 18 SQL migration files (automated on startup)
- **Testing:** 100+ integration tests with mock infrastructure

---

## DEPLOYMENT OPTIONS

### Option 1: Docker Compose (Recommended)
```bash
docker compose up -d
# Services: postgres, nats, mosquitto, api, reporting-service, weather-service, admin-ui
```

### Option 2: Native Installation
```bash
cargo run -p agrocore-api
cd crates/admin-ui && trunk serve
```

### Option 3: Kubernetes
```bash
# Helm chart available in infra/k8s/
helm install agrocore ./infra/helm/agrocore
```

### Option 4: Systemd (Linux Server)
Systemd unit files included in `systemd_units/` for production service management.

---

## CURRENT VERSION: 0.8.3

### Recent Highlights
- **Full IoT Integration** — MQTT Bridge, Home Assistant Auto-Discovery, Device Registry
- **Multi-Country LPIS** — 8 national parcel systems with configurable providers
- **Offline-First Sync Engine** — Complete offline capability with conflict resolution
- **Advanced RBAC** — 4-role system with granular per-resource permissions
- **Webhook System** — Event-driven external integrations via NATS
- **Performance Optimizations** — DecodingKey caching, PgPool config, repo macro, topic precomputation, role mapping optimization
- **Full Integration Test Suite** — 100+ tests covering all 17 modules
- **SIGPAC API Integration** — 3 endpoints for Spanish parcel queries

---

## COMPARISON WITH COMPETITORS

| Feature                          | AgroCore-RS | FarmLogs | Climate FieldView | John Deere OC | Granular |
|----------------------------------|-------------|----------|-------------------|---------------|----------|
| Local-first (on-premise)         | YES         | NO       | NO                | NO            | NO       |
| Multi-country LPIS (8 countries) | YES         | NO       | NO                | NO            | NO       |
| IoT / MQTT Integration           | YES         | NO       | Limited           | Limited       | NO       |
| Home Assistant Auto-Discovery    | YES         | NO       | NO                | NO            | NO       |
| Offline-First                    | YES         | NO       | NO                | Limited       | NO       |
| Open Source (GPL)                | YES         | NO       | NO                | NO            | NO       |
| No per-user fees                 | YES         | NO       | NO                | NO            | NO       |
| Rust performance                 | YES         | NO       | NO                | NO            | NO       |
| EU PAC/SIP Reporting             | YES         | NO       | NO                | NO            | NO       |
| Harvest Prediction (BBCH)        | YES         | YES      | YES               | YES           | YES      |
| Vineyard/Olive Specializations   | YES         | NO       | NO                | NO            | NO       |
| Full Audit Trail                 | YES         | NO       | NO                | NO            | NO       |

---

## LICENSE

**GNU General Public License v3.0 or later**

This means:
- ✅ Free to use, modify, and distribute
- ✅ Full source code access
- ✅ No vendor lock-in
- ✅ Commercial use permitted
- ❌ Must distribute modifications under same license (GPL)

---

## GET STARTED

1. Clone: `git clone https://github.com/peopleandpixel/agrocore-rs`
2. Configure: Copy `.env.example` to `.env` and set your passwords
3. Run: `docker compose up -d` or `./scripts/dev.sh`
4. Access: API at `http://localhost:8080`, Admin UI at `http://localhost:3000`
5. Docs: API at `http://localhost:8080/swagger-ui/`, OpenAPI at `http://localhost:8080/api-docs/openapi.json`

---

**© 2024-2026 AgroCore-RS Development Team**
**License: GNU GPL v3.0+**
**Built in Portugal 🇵🇹 — Designed for European Agriculture**
