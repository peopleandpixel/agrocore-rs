# agrocore-rs — Projekt-Roadmap (Checkliste)

## ✅ ABGESCHLOSSEN (Phase 1-16)

### Phase 1: Basis-API (MVP)
- [x] Workspace-Struktur (shared, domain, infrastructure, api)
- [x] Domain Entities (Tenant, User, Site, Order, TaskData)
- [x] Infrastructure (MongoDB, Password Hashing, JWT)
- [x] DTOs für API-Responses
- [x] JWT Auth Middleware
- [x] API Routes (Sites, Orders, Users, Tasks, Auth)
- [x] Governor Rate-Limiting (120 req/min, per-IP)
- [x] Tests in separaten Verzeichnissen (11 Handler)

### Phase 2-16: Alle Features implementiert
- [x] Compliance & GAP-Nachweis (AuditLog, Checklisten)
- [x] Spezialkulturen & Dauerkulturen (Wein, Oliven, DOC Areas)
- [x] Wassermanagement (Sources, Usage, Quotas)
- [x] Arbeitergesetzgebung (Worker, Certifications, WorkLog)
- [x] Finanzen & EU-Beihilfen (PAC, Cost Centers, SIGPAC)
- [x] Ernte-Logistik & Qualität (Chargen, Wiegung, Kühlkette)
- [x] Wetter & Klima (BBCH-Phänologie, Frost-Warnungen)
- [x] Reporting & Export (Excel, GeoJSON, Swagger/OpenAPI)
- [x] Microservices (NATS, Messaging, Placeholder-Workers)
- [x] DevOps & Observability (Prometheus, Grafana, Loki, Health-Checks)
- [x] IAM (RBAC, Visibility-Aware Entities, Token-Session)
- [x] Livestock (Behandlungen, Bewegungsplanung, Futterbedarf)
- [x] Analytics (Erntevorhersage, KPI-Berechnung)

## 🔴 OFFEN (Infrastructure - Optionen)

### Deployment [ERLEDIGT]
- [x] **Option A:** Docker Compose (lokal) - 🐳 `docker-compose up -d`
- [ ] **Option B:** Kubernetes Manifests (Produktiv) - später wenn Cluster verfügbar

### Integration Tests [ERLEDIGT - Mock-Repos]
- [x] Compliance-Tests - GAP-Checkliste Mock (site_test.rs, compliance_test.rs)
- [x] Olive-Tests - OliveGrove CRUD Mock (olive_test.rs)
- [x] Site-Tests - Polygon-Geo Mock (site_test.rs)

### Data Consistency Review [ERLEDIGT]
- [x] DTO vs Repository Validation - Alle Entities haben Create/Update DTOs
- [x] Geo-Queries Indizes - SpatialObjectRepository mit `find_containing_point` existiert

### Worker Task Automation [ERLEDIGT]
- [x] Standard-Dauer-Feld im Order DTO (TaskExecutionPolicy.default_duration_minutes)
- [x] Auto-Completion Logic: Time-basiert (TaskAutomationAction::Completed)
- [x] Location-Trigger: GPS-Polygon-Betreten/Verlassen (process_presence_event)
- [x] Worker Web UI: Aufgabenliste mit Start/Pause/Stop-Buttons (/worker/tasks)
- [x] Task-Status Workflow: open → started → completed/automatisch
- [x] Worker-Task-Status Repository für Multi-Worker-Aufgaben (WorkerTaskStatus)
- [x] Aggregierter Status: Started wenn mind. ein Worker aktiv ist

## 🧹 TECHNICAL DEBT GELÖST

- [x] Soft-Delete Filtering - `is_active != false` in Repositories
- [x] Reporting Pagination Fix - `page: Some(0)` für erste Seite
- [x] Fallback-Hosts - Umgebung auf `localhost`
- [x] JWT-Secret Caching - Einmaliger Load statt pro Request
- [x] Pagination Indizes - `tenant_id + is_active + updated_at`
- [x] Governor Integration - actix-governor 0.10
- [x] GitHub Actions Security Audit (weekly)
- [x] GitHub Actions Release Workflow (semver tags)
- [x] Worker-Task-Status Entity & Repository (Multi-Worker-Aufgaben)
- [x] Worker Web UI Task Page mit Steuerungs-Buttons