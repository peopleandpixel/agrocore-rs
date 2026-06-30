# agrocore-rs — Projekt-Roadmap

## Phase 1: Basis-API (MVP) — ABGESCHLOSSEN

### Erledigt
- [x] Workspace-Struktur (4 Crates: shared, domain, infrastructure, api)
- [x] Domain Entities (Tenant, User, Site, Order, TaskData)
- [x] Infrastructure (MongoDB Repositories, Password Hashing, JWT)
- [x] DTOs für API-Responses
- [x] JWT Auth Middleware
- [x] API Routes (Sites, Orders, Users, Tasks, Auth/Login)
- [x] Cargo.toml mit Dependencies
- [x] actix-web 4 als Framework
- [x] CORS aktiviert
- [x] MongoDB Indizes (4 Indizes bei Connect)
- [x] Server main.rs fertig
- [x] 6 Integrationstests in `crates/api/tests/`
- [x] Alle Dependencies auf neuesten Stand
- [x] `cargo check` ohne Fehler/Warnings
- [x] `cargo test` alle Tests grün

---

## Phase 2: Compliance & GAP-Nachweis — ABGESCHLOSSEN

- [x] AuditLog Entity (CreateAuditLogDto, AuditAction)
- [x] ComplianceChecklist Entity (GAP, Organic, GlobalGAP, HACCP)
- [x] ChecklistItem Entity (Evidence-URL, Completion-Tracking)
- [x] FertilizerRecord Entity (NPK-Bilanz, quantity_kg, area_ha)
- [x] PlantProtectionRecord Entity (Active Substance, Pre-Harvest Days, Re-Entry)
- [x] ApplicatorLicense Entity (License Type, Validity, Certificate Number)

---

## Phase 3: Spezialkulturen Wein & Oliven — ABGESCHLOSSEN

- [x] Vineyard Entity (DOC Area, Vintage, Grape Variety)
- [x] KelterDelivery Entity (Gross/Net Weight, Lot Number, Temperature)
- [x] QualityGrade Enum (Reserva, GrandeReserva, Garrafeira, etc.)
- [x] DocArea Enum (Douro, Alentejo, Vinho Verde, etc.)
- [x] OliveGrove Entity (Variety, Tree Count, Organic)
- [x] OliveOilRecord Entity (Acidity, Peroxide, Sensory Score)
- [x] OilGrade Enum (Extra Virgin, Virgin, Lampante, etc.)

---

## Phase 4: Wassermanagement — ABGESCHLOSSEN

- [x] WaterSource Entity (Well, Reservoir, River, Canal, Comunidad)
- [x] WaterUsage Entity (Volume, Irrigation Method, Efficiency)
- [x] WaterQuota Entity (Allocated, Used, Year)
- [x] WaterSourceType Enum
- [x] IrrigationMethod Enum (Drip, Sprinkler, Flood, Pivot, etc.)

---

## Phase 5: Arbeitergesetzgebung — ABGESCHLOSSEN

- [x] Worker Entity (Contract Type, Language, Skills, Certifications)
- [x] Certification Entity (Name, Issued By, Validity)
- [x] WorkLog Entity (Hours, Overtime, Rest Period, Night Shift)
- [x] ContractType Enum (Permanent, Temporary, Seasonal, Freelance)

---

## Phase 6: Finanzen & EU-Beihilfen (PAC) — ABGESCHLOSSEN

- [x] PAC-Antragstellung (einheitliches Antragsformular SIP)
- [x] Kreislaufwirtschaft-Beihilfen (Eco-esquemas in ES)
- [x] Kosten-Stellenrechnung (pro Parzelle, pro Kulture, pro Arbeitsgang)
- [x] SIGPAC-Parzellen-IDs (offizielle EU-Flurstücks-IDs)
- [x] REGEPAC (Registro de Explotaciones Agrarias)

---

## Phase 7: Ernte-Logistik & Qualität — ABGESCHLOSSEN

- [x] Los-Tracking / Chargen-Nummern
- [x] Wiegung (Brutto/Netto-Gewicht pro Ernte-Lieferung)
- [x] Kühlketten-Protokolle
- [x] Ernte-Jahresgang-Verknüpfung

---

## Phase 8: Wetter & Klima — ABGESCHLOSSEN

- [x] Wetter-Stationen-Integration (Temperatur, Niederschlag, Luftfeuchte)
- [x] Frost-Warnungen
- [x] BBCH-Phänologie-Modell mit Vorhersage

---

## Phase 9: Reporting & Export — ABGESCHLOSSEN

- [x] PAC-Antragstellung (einheitliches Antragsformular SIP)
- [x] Excel-Export (rust_xlsxwriter)
- [x] GeoJSON-Export für Karten
- [x] OpenAPI/Swagger-Dokumentation (utoipa)

---

## Phase 10: Microservices Transition

### Erledigt
- [x] **Messaging-Bus Evaluierung & Setup:** Integration von NATS in die Workspace-Struktur.
- [x] **Shared Messaging Crate:** Erstellung einer `crates/messaging` Crate für einheitliche Message-Typen.
- [x] **Docker Compose Setup:** Erstellung einer lokalen Entwicklungsumgebung inkl. MongoDB, NATS und Observability-Stack (Loki, Prometheus, Grafana).

### Service-Extraktion
- [x] **Reporting-Service:** Extraktion der Excel- und GeoJSON-Generierung in einen eigenständigen Service (Placeholder-Worker implementiert).
- [x] **Weather-IoT-Service:** Auslagerung der Wetterstationen-Integration und BBCH-Modellierung (Placeholder-Worker implementiert).
- [x] **Async-Audit-Log:** Umstellung des Audit-Loggings auf ein asynchrones Messaging-Modell (Worker vorbereitet).

### Deployment & Monitoring
- [x] **Service-Discovery:** Vereinheitlichung der Konfiguration über Umgebungsvariablen.
- [x] **Distributed Tracing:** Zentrales Tracing-Setup in `shared` und Trace-ID Unterstützung in Messaging-Events.
- [x] **Shared Crate Refactoring:** Einführung von `telemetry` Modul und Vereinheitlichung der Abhängigkeiten.

---

## Phase 11: Infrastruktur, DevOps & Observability

### Erledigt
- [x] **OpenAPI / Swagger:** Integration von `utoipa` und `utoipa-swagger-ui`.
- [x] **Structured Logging:** Zentralisierte `tracing` Konfiguration für alle Microservices.
- [x] **Metrics:** Prometheus-Endpunkt für Performance-Monitoring in der API integriert.
- [x] **CI/CD:** GitHub Actions für automatische Tests und Linting konfiguriert.
- [x] **Logging-Aggregation:** Setup von Loki für zentrales Log-Management (via Docker Compose).
- [x] **Grafana Dashboards:** Vorbereitung der Infrastruktur für Grafana (via Docker Compose).

### Offen
- [x] **Grafana Dashboards:** Visualisierung der Prometheus-Metriken (Business-KPIs vs. System-Health). (Infrastruktur bereit)
- [ ] **Kubernetes Manifests:** Vorbereitung für den Cluster-Betrieb (Deployment, Services, Ingress).
- [x] **Health-Checks:** Standardisierte `/health` Endpunkte für alle Microservices zur Liveness/Readiness-Prüfung.

---

## Phase 12: Microservice-Reife & Messaging-Muster

### Erledigt
- [x] **NATS Request-Response Pattern:** Typsichere Implementierung für den Reporting-Service und andere Inter-Service Kommunikation.
- [x] **Event-Driven Updates:** Sites/Orders emittieren Events bei Änderungen (`GlobalEvent`), die von anderen Services konsumiert werden können.
- [x] **Shared Schema Registry:** Zentrale Definition der Message-DTOs und Events in `agrocore-messaging`.
- [x] **Error Handling & Retries:** Automatische Retry-Logik mit Exponential Backoff im `MessagingClient`.
- [x] **Circuit Breaker:** Schutz der API vor überlasteten oder nicht erreichbaren Microservices mittels `failsafe` im `MessagingClient`.
- [x] **Workspace Stability:** Alle Crates im Workspace (`api`, `weather-service`, `reporting-service`) kompilieren fehlerfrei.
- [x] **Geometry-Service:** Spezialisierter GIS-Service für Geometrie-Mapping (Ackerschläge, Weiden, Waldparzellen) und Topografie-Analysen.
- [x] **Asset-Registry:** Zentraler Service zur Verwaltung von Inventar (Maschinen, Equipment, biologische Assets).

### Offen

---

## Phase 13: Erweitertes Auftragsmanagement & Ressourcen

### Erledigt
- [x] **Generisches Aufgaben-Management:** Vollständige Implementierung der Auftragserstellung & Modifikation für alle Agrar-Sparten. (Basis-Logik in API vorhanden)
- [x] **Workflow Engine:** State-Machine basierte Statusübergänge für komplexe Arbeitsabläufe.
- [x] **Folgeaufträge:** Logik für automatisierte Folgeaufgaben (z.B. Ernte nach Pflanzenschutz-Wartezeit).
- [x] **Equipment- & Maschinenverwaltung:** Verwaltung von Ausrüstung, Wartungsintervallen und Verfügbarkeit.
- [x] **Kostenstellen-Zuordnung:** Detaillierte Zuweisung von Maschinen- und Materialkosten zu Aufträgen und Flächen.

### Offen

---

## Phase 14: Identity & Access Management (IAM)

### Erledigt
- [x] **Mandanten-Konfiguration:** Erweiterte Einstellungen pro Betrieb (Tenant-spezifische Regeln und Validierungen).
- [x] **Sichtbarkeitsregeln:** Implementierung von feingranularen Filtern auf Repository-Ebene (VisibilityAwareEntity).
- [x] **RBAC / Token-Rollen:** Rollenbasierte Zugriffskontrolle für verschiedene Benutzertypen (Arbeiter, Agronom, Admin).
- [x] **Benutzer-Events:** Systemweite Benachrichtigungen bei Benutzeraktionen oder Stammdatenänderungen.

### Offen

---

## Phase 15: Tierhaltung (Livestock Management) & Spezialisierung

### Erledigt
- [x] **Belegungsplanung:** Grafische/Logische Verwaltung der Weiderotation (welche Tiergruppe auf welcher Fläche).
- [x] **Futterbedarfsberechnung:** Automatisierte Kalkulation basierend auf Tierzahl, Alter und Weidequalität.
- [x] **Behandlungsregister:** Dokumentation von Tiergesundheit (Impfungen, Medikamente) inkl. Wartezeiten.
- [x] **Erschwernis-Zulagen:** Generisches System für Arbeitserschwernisse (z.B. Steillagen, schwer zugängliche Flächen).

---

## Phase 16: Advanced Analytics & Prediction Models

### Erledigt
- [x] **Erntevorhersage:** Algorithmen zur Schätzung des Erntezeitpunkts und der Menge basierend auf BBCH und Wetterdaten.
- [x] **Veterinär-Reporting:** Automatisierte Berichte für Behörden (Bestandsveränderungen, Medikamenteneinsatz).
- [x] **Kosten-Nutzen-Analyse:** Detaillierte Auswertung der Wirtschaftlichkeit pro Schlag/Weide.
- [x] **IoT-Sensor-Integration:** Einbindung von Echtzeit-Daten (Bodenfeuchte, Tieraktivität) in die Entscheidungsfindung.

---

## Technische Debt & Optimierungen

### Testing & Performance
- [x] **Unit Tests:** Validierungslogik in `domain` isoliert testen.
- [x] **Mocking:** Repositories in API-Tests mocken für DB-Unabhängigkeit.
- [ ] **Integration-Tests:** Ausweitung der API-Tests auf Fachmodule (Compliance, Olive, Vineyard).
- [ ] **Data Consistency:** Cross-Check der Domain-Validierung zwischen API-DTOs und Repositories.
- [x] **Index-Audit:** Überprüfung aller Repositories auf fehlende Indizes (besonders für Geo-Queries in Sites).
- [x] **Connection Pooling:** MongoDB-Client-Konfiguration für hohen Durchsatz optimieren.

### Spezifische Repositories & Routen
- [x] MongoDB Repositories für neue Entities (Vineyard, Olive, Water, Compliance, Workforce) implementiert.
- [x] API Routes für neue Module integriert.
- [x] **Data Validation:** Umstellung auf striktere `validator` Regeln in den API-DTOs.
- [x] **Frontend-Integration:** Neue Crate `crates/admin-ui` mit Leptos, TailwindCSS und DaisyUI erstellt. Rollenbasiertes Dashboard-Layout implementiert.
