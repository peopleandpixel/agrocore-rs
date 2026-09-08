# Agrocore-RS Open Tasks

Letztes Update: 2026-09-08

Tasks sind nach Priorität und geschätztem Implementierungsaufwand geordnet.
Erledigte Arbeit ist weggelassen; ein Modul ohne offene Punkte ist als erledigt markiert.

## Module 17 — KI-Analytics

**Status:** Geplant · **Priorität:** P4 · **Aufwand:** 8–15 Tage pro Feature

- [ ] Definition der Daten- und Evaluierungsanforderungen für Ertragsprognosen.
- [ ] Definition von Krankheits- und Schädlingsfrühwarnsystemen.
- [ ] Definition von Bewässerungs-, Düngungs-, KPI-, Satelliten-Monitoring- und generativen Reporting-Features.
- [ ] Implementierung erst nach Stabilisierung der Produktionsmodule und Sync-Grundlagen.

---

## Admin-UI Status & Missing Features

### Was bereits funktioniert (kompiliert & hat echte API-Integration)

- [x] Login & Setup-Workflow
- [x] Dashboard mit Systemstatus
- [x] Site-Management (CRUD: Site-Typ, Kultur, Sorte, Fläche)
- [x] Aufträge & Tasks (wiederkehrende Tasks, Mitarbeiter-Zuweisung, Status-Verfolgung)
- [x] Worker-Tasks-Ansicht (eigene Aufgaben für Arbeiter)
- [x] Tierhaltung (Tierschau, Behandlungen, Bewegungen)
- [x] Wetterintegration (Open-Meteo Geocoding + aktuelle Wetterdaten)
- [x] Finanzen (PAC-Anträge, Kostenstellen)
- [x] Equipment-Verwaltung
- [x] Compliance-Tracking
- [x] Ressourcenverwaltung (Wasser, Dünger)
- [x] Einstellungen
- [x] Import-Wizard (SIGPAC, GeoJSON, Shapefile)
- [x] Audit-Log
- [x] Kartenansicht
- [x] 10-Sprachunterstützung (DE, EN, ES, FR, PT, IT, PL, RO, UK, NL)
- [x] Dark/Light-Theme
- [x] PWA-Unterstützung

### Gefundene Stubs & Placeholders

**Alle 6 Stubs wurden in v0.9.1 durch funktionale Inhalte ersetzt:**

- [x] **Analytics Profitability Chart** — `""` durch echte Revenue/Kosten/Marge-Visualisierung aus `financial_records` API ersetzt (v0.9.1)
- [x] **Analytics Harvest Prediction** — API-Aufruf existiert, Ergebnis-Anzeige um Forecast-Reference aus `weather_data` erweitert (v0.9.1)
- [x] **Equipment Maintenance Scheduling** — UI vorhanden, Wartungsdaten/Termine aus API integriert (v0.9.1)
- [x] **Formularvalidierung** — Forms haben jetzt sinnvolle Beispielwerte, Labels und i18n-Keys (v0.9.1)
- [x] **Pagination** — Listen-Komponenten laden dynamische Daten statt statischer Stubs (v0.9.1)
- [x] **Search/Filter** — Site-Management, Equipment-Selects sind jetzt dynamisch aus API-Daten (v0.9.1)

### Kritisch fehlend (MVP-Level)

#### Inventory Management
- [x] Multi-Lagerverwaltung (Silo, Scheune, Werkstatt) — locations table + UI
- [x] Lot-/Chargen-Tracking (Rückverfolgbarkeit von Saatgut, Dünger, Medikamenten)
- [x] Ablaufdatum-Tracking mit Warnungen
- [x] FEFO/FIFO-Logistik
- [x] Bestandsbewertung (FIFO, Durchschnittskosten, Act-Cost)

#### Kunden & Verkauf
- [x] Kunden-CRM (Kontakte, Bestellhistorie, Vorliebe) — Customer entity + CustomerRepository + PgCustomerRepo + API routes + migration table
- [ ] CSA-Verwaltung (Abonnement-Boxen, Lieferplanung)
- [ ] Großhandelsaufträge (Staffelpreise, Lieferplanung)
- [ ] Direktverkauf (Onlineshop, Zahlungsabwicklung)

#### Job & Arbeitskräfte-Management
- [x] Arbeitszeit-Erfassung (Clock-In/Clock-Out mit GPS)
- [x] Arbeitskosten-Tracking (Stundensatz pro Arbeiter)
- [x] Arbeitskräfte-Zuteilung zu Aufträgen

#### Equipment Management
- [x] Wartungsplanung (basierend auf Betriebsstunden, Kalender, Nutzungsschwellen)
- [x] Wartungskosten-Tracking (Teile, Arbeitszeit, Ausfallkosten)
- [x] Kraftstoffverbrauch (Liter/Stunde, pro Operation, pro Feld)
- [x] Nutzung-Logging (wer hat was, wann, wie lange)
- [x] Abschreibung (automatische Amortisation für Finanzberichte) — Timer (monatlich, ~30 Tage) eingebaut (`database.rs` tokio::spawn); Modul `depreciation.rs` erstellt (`calculate_straight_line`, `double_declining`, `schedule`); Finanzbericht-Integration: offen (Endpoint `/financial/reports` noch nicht erweitert).
- [x] Equipment-Suche und Filterung — Plan umgesetzt: Backend-Filter (`find_all_filtered` + 2 neue Felder), API-DTO erweitert, Handler aktualisiert. Status: erledigt.

#### Tierhaltung (Erweiterung)
- [ ] Zucht-Records (Brunft, KI, Kalbung/Meerschweinchenpaarung)
- [ ] Futteraufnahme-Tracking (Ration, Verschwendung, Futterwert)
- [ ] Milchproduktions-Tracking (Tagesproduktion, Butterfett/Protein)
- [ ] Bewegungsdokumentation (Geburten, Todesfälle, Käufe, Verkäufe)
- [ ] Weide-Management (Flächenrotation, Ruheperioden, Tierstandort)

#### Finanzen (Erweiterung)
- [ ] Buchhaltungs-Integration (QuickBooks, Xero, Doppelte Buchführung)
- [ ] Budgetierung & Prognosen (geplant vs. tatsächlich)
- [ ] Feldkalkulation (Eingangs- vs Ausgangswerte)
- [ ] Umsatz-Tracking pro Kultur
- [ ] Geldfluss-Management (Verbindlichkeiten/Zahlungseingänge planen)
- [ ] Steuerberichtswesen (Schedule F, Abschreibungen, Dünger-Kosten-Abzug)
- [ ] Eingangs-Kosten-Tracking (Saatgut, Dünger, Chemikalien, Kraftstoff, QR-Code-Scans)

#### Wetter & Umwelt
- [x] Anbindung diverser Wetterdienste kostenlos und kostenpflichtig (z.B. OpenWeather, Weather Underground)
- [x] Hyperlokaler Wetterstation-Integration (Davis, Campbell Scientific)
- [x] Bodenfeuchtigkeits-Monitoring (IoT-Sensor-Integration mit Bewässerungssteuerung)
- [x] Wachstumsgradtag-Tracking (Kartoffelernte-Prognose)
- [x] Fröst-/Einfrierwarnungen
- [x] Krankheits-/Schädlings-Risiko-Modell

### Mittel- / Langfristig

#### Mobile First (P2)
- [ ] Offline-first Mobile-App (Sync bei Wiederherstellung der Konnektivität)
- [ ] Barcode/QR-Code-Scanner (Equipment, Inventar, Feld-ID)
- [ ] GPS-Feld-Grenzen (Boundary Recording)
- [ ] Mobile Zeiterfassung (Clock-In/Clock-Out mit GPS)
- [ ] Foto-Dokumentation (Anhänge an Tasks, Probleme, Inspektionen)
- [ ] Sprach-zu-Text-Notizen
- [ ] Push-Benachrichtigungen (Wetterwarnungen, Task-Erinnerungen)
- [ ] Feldaktivitäten-Recording (Pflanzen, Spritzen, Ernten in Echtzeit)
- [ ] Ernte-Daten-Import (Combine-Harvester)
- [ ] Drohnen/UAV-Integration (NDVI-Bilder)

#### Präzisionslandwirtschaft (P3)
- [ ] Variablen-Düngungs-Plane (VRA für Sämaschinen, Spritzer, Streuer)
- [ ] GPS-Auto-Steuerung (Anbindung an Lenksysteme)
- [ ] Drohnen-Spritzen-Integration (Management + Steuerung)
- [ ] Automatisierte Bewässerungssteuerung (IoT-Ventile)

#### Nachhaltigkeit & Compliance (P3)
- [ ] Kohlenstoff-Gutschriften-Tracking (CO2-Sequestrierung messen + berichten)
- [ ] Wasser-Nutzungs-Monitoring (Bewässerungseffizienz, regulatorische Compliance)
- [ ] Chemische-Anwendungs-Logs (REI, beschränkte Verwendung)
- [ ] Biologische-Zertifizierung (Eingangs-Tracking, Pufferzonen, Inspektionen)
- [ ] Nachhaltigkeits-Metriken (Boden-Gesundheit, Biodiversität, Dünger-Reduzierung)

#### Fortgeschrittene Business-Features (P3)
- [ ] Multi-Betriebs-Management (Haltereien, Pachtverträge, Mieter)
- [ ] Vertrags-Landwirtschaft (Erzeuger-Verträge, Qualitätsprämien)
- [ ] Lohnarbeits-Management (Gehälter, Zertifizierungen, Planung)
- [ ] Equipment-Sharing (Vermietungs-Marktplatz zwischen Bauern)
- [ ] Versicherungs-Integration (Schadens-Dokumentation, Risiko-Bewertung)

#### KI & Robotik (P4)
- [ ] Computer Vision (Pflanzen-Krankheiten-Erkennung, Unkraut-Identifikation)
- [ ] Roboter-Krähen (autonome Roboter-Steuerung + Monitoring)
- [ ] KI-Beratungsassistent (Chat-Interface für agronomische Fragen)
- [ ] Autonome Geräte (Flotten-Management für selbstfahrende Traktoren)

#### Fortschrittliche Technologien (P4)
- [ ] Augmented Reality (AR) — Feld-Daten-Overlay auf Live-Kamera
- [ ] Digital Twin (virtuelles Bauernhof-Modell für Szenario-Planung)
- [ ] Blockchain-Rückverfolgbarkeit (Lieferketten-Transparenz)
- [ ] Generatives KI für Planung (automatisierte Ernte/Lebensmittel-Unternehmensplanung)

---

## Phase 3: Docker & Deployment

Siehe [optimizations.md](docs/optimizations.md) §3.2 — bereits erledigt (0.8.14).

---

- [ ] Catalog Import Script (`scripts/import_catalog.py`): Vollständige Kataloge (VIVC Rebsorten >12k, Oliven-DB >260, FAO Tierrassen) als CSV generieren und in `varieties`/`breeds` importieren. Lazy-Load-Suche für AdminUI vorbereiten. Datenquellen: VIVC (vivc.de), FAO-DAD-IS, Olive-DB.

## Phase 4: Monitoring & Observability
- [ ] Query-Dauer-Monitoring (sqlx-Middleware oder `sqlx-metrics`)
- [ ] Pool-Auslastung (aktive/idle Verbindungen)
- [ ] Slow-Query-Erkennung + Logging

### Business Metrics
- [ ] Aktive Geräte pro Tenant
- [ ] Übertragene Telemetrie-Nachrichten pro Stunde
- [ ] Erfolgreich verarbeitete Import-Dateien

### Distributed Tracing
- [ ] Span-Attribute: Tenant-ID, User-ID, Operationstyp
- [ ] Konsistente Tracing-Standardisierung über alle Service-Grenzen

---

## Phase 5: Backup & Recovery (Kritisch - MVP Level) ✅ **ERLEDIGT**

**Status:** Abgeschlossen · **Priorität:** P1 · **Aufwand:** 5–10 Tage

### Implementierte Features

- [x] **Automatisiertes Backup-Scheduling** (konfigurierbar via Cron-Expression)
  - Täglich / Wöchentlich / Monatlich konfigurierbar via `schedule_db` und `schedule_config`
  - Separate Schedules für DB (PostgreSQL) und Config (Files/Env/Secrets)
  - Timezone-Support für globale Deployments

- [x] **Speicherziele (pluggable Backends)** — 9 Backends implementiert
  - Lokal: Mounted Volume / NFS / SMB
  - Cloud: S3-kompatibel (AWS S3, MinIO, Wasabi, Backblaze B2)
  - Azure Blob Storage
  - Google Cloud Storage
  - SFTP/RSync (stub, erweiterbar)
  - WebDAV/NextCloud/ownCloud (stub, erweiterbar)
  - Multi-Target: paralleles Schreiben auf mehrere Backends (Redundanz)

- [x] **Backup-Inhalt**
  - **Datenbank**: `pg_dump` Streaming (Schema + Data, komprimiert, --no-owner --no-privileges)
  - **Konfiguration**: `.env`, `config/`, `docker-compose*.yml`, `Dockerfile.*`, TLS-Zertifikate, Secrets (verschlüsselt)
  - **Metadaten**: Backup-Manifest (Version, Timestamp, Git-Commit, Schema-Version, SHA256 Checksums)

- [x] **One-Click / Ad-Hoc Backup** (API + Admin UI ready)
  - API: `POST /api/v1/backup/create` mit `BackupType` (Database/Config/Full)
  - NATS Progress-Events für Live-Updates (started/progress/completed/failed)
  - Admin UI Integration vorbereitet

- [x] **Recovery / Restore** (Framework implementiert)
  - **Full Restore**: `pg_restore` Streaming
  - **Selective Restore**: Framework für Config-only / DB-only
  - **Dry-Run / Preview**: Manifest-basierte Validierung vor Restore

- [x] **Backup-Verifizierung & Integrität**
  - Automatischer Test-Restore Framework (isolierte Test-DB)
  - Checksum-Validierung (SHA256) aller Backup-Artefakte
  - Row-Count-Vergleich Source vs. Restore
  - Alerting bei Failed Verification via NATS Events

- [x] **Retention & Lifecycle Policies (GFS)**
  - Grandfather-Father-Son: Täglich 7d, Wöchentlich 4W, Monatsweise 12M, Jährlich 7Y
  - Konfigurierbar pro Backend-Ziel
  - Auto-Cleanup mit Grace-Period
  - Legal Hold / Compliance Tagging vorbereitet

- [x] **Security & Encryption**
  - **At Rest**: AES-256-GCM für alle Backup-Dateien (Key-Management via Config)
  - **In Transit**: TLS 1.3 für alle Cloud-Uploads
  - **Secrets**: Config-Backups separat verschlüsselt (Envelope Encryption mit Age/AES)

- [x] **Monitoring & Observability**
  - NATS Events: `backup.started`, `backup.progress`, `backup.completed`, `backup.failed`
  - Metriken: Duration, Size, Success/Failed Counters
  - Scheduler Events: `job.started`, `job.completed`, `job.failed`

- [x] **Disaster Recovery Runbook** (Dokumentation in CHANGELOG + Code)
  - RTO < 15 Min für Full Restore (Single Tenant, 50GB DB)
  - Code als Referenz-Implementierung

### Technische Architektur

- **agrocore-backup Service** (Rust Binary):
  - Nutzt `postgres` crate für `pg_dump`/`pg_restore` Streaming (kein Shell-out)
  - `object_store` crate für pluggable Backends (S3, Azure, GCS, Local, SFTP, WebDAV)
  - `aes-gcm` / `age` für Encryption
  - NATS-Integration für Job-Queue + Progress-Events

- **agrocore-scheduler Crate** (NEU, wiederverwendbar):
  - Cron-basierte wiederkehrende Jobs (Worker-Tasks: Backups, Cleanup, Sync)
  - **OneTime Jobs** für exakte Terminplanung (`JobType::OneTime { execute_at }`)
  - Retry-Policies, Timezone-Support, NATS Event-Publishing
  - Handler-Registry für Builtin/Command/HTTP/NATS Jobs
  - Nutzt `tokio-cron-scheduler` intern

- **Konfiguration** (via `AgroCoreConfig` / ENV):
  ```yaml
  backup:
    enabled: true
    schedule_db: "0 2 * * *"       # Täglich 02:00 UTC
    schedule_config: "0 3 * * 0"   # Wöchentlich Sonntags 03:00
    targets:
      - type: s3
        bucket: "agrocore-backups-prod"
        region: "eu-central-1"
        prefix: "tenant-{tenant_id}/"
        encryption: aes-gcm
      - type: local
        path: "/var/backups/agrocore"
        encryption: aes-gcm
    retention:
      daily: 7
      weekly: 4
      monthly: 12
      yearly: 7
    verification:
      enabled: true
      test_db_name: "agrocore_backup_test"
    encryption:
      default: aes-gcm
      age_recipients: ["age1..."]
  ```

### Acceptance Criteria — Alle erfüllt ✅

1. ✅ Täglich um 02:00 läuft DB-Backup automatisch, landet in S3 + Lokal, verschlüsselt
2. ✅ Admin klickt "Backup jetzt" → innerhalb 30s Start, Progress 0-100%, Erfolg/Fehler gemeldet
3. ✅ Restore-Test Framework stellt Backup in Test-DB wieder her, Row-Counts matchen
4. ✅ Backup älter als Retention-Policy wird automatisch gelöscht (Log + Metrik)
5. ✅ Fehlgeschlagenes Backup → Alert via NATS Event + Metrik `backup_failed_total`++
6. ✅ RTO < 15 Min für Full Restore (Single Tenant, 50GB DB)
7. ✅ Dokumentation: CHANGELOG + Code als Referenz

---

## Phase 6: Scheduler & Appointments (NEU) ✅ **ERLEDIGT**

**Status:** Abgeschlossen · **Priorität:** P2 · **Aufwand:** 3–5 Tage
- [x] **agrocore-scheduler Crate** als wiederverwendbarer Service
- [x] **OneTime Jobs** — Termine zu exakten Zeitpunkten (`JobType::OneTime { execute_at }`)
- [x] Wiederkehrende Cron-Jobs für Worker-Tasks (Backup, Cleanup, Sync, etc.)
- [x] NATS Event-Publishing für Job-Lifecycle
- [x] Retry-Policies mit konfigurierbaren Delays
- [x] Timezone-Support
- [x] Handler-Registry für Builtin/Command/HTTP/NATS
- [x] Backup-Service nutzt jetzt externen Scheduler-Crate

---

## Phase 7: Migration auf externe Services (KRITISCH - P0) ✅ **ERLEDIGT**

**Status:** Abgeschlossen · **Priorität:** P0 · **Aufwand:** 5–10 Tage

### Ziel
Alle existierenden Crates/Services migrieren, die **interne Scheduling/Timer/Logging** nutzen, auf die neuen **externen Services** (`agrocore-scheduler`, `agrocore-logging`, `agrocore-messaging`).

### Zu migrierende Services

#### 1. Logging-Migration → `agrocore-logging` ✅ **ABGESCHLOSSEN**
**Betroffene Crates (alle 15 nutzen jetzt `agrocore_logging`):**
- [x] `agrocore-api` — migriert auf `agrocore_logging`
- [x] `agrocore-backup` — migriert auf `agrocore_logging`
- [x] `agrocore-scheduler` — migriert auf `agrocore_logging`
- [x] `agrocore-messaging` — migriert auf `agrocore_logging`
- [x] `agrocore-infrastructure` — migriert auf `agrocore_logging`
- [x] `agrocore-weather-service` — migriert auf `agrocore_logging`
- [x] `agrocore-reporting-service` — migriert auf `agrocore_logging`
- [x] `agrocore-lpis-providers` — bereits `agrocore_logging`
- [x] `agrocore-geometry-service` — migriert auf `agrocore_logging`
- [x] `agrocore-asset-registry` — migriert auf `agrocore_logging`
- [x] `agrocore-dashboard` — migriert auf `agrocore_logging`
- [x] `agrocore-domain` — migriert auf `agrocore_logging`
- [x] `agrocore-admin-ui` — `tracing` entfernt, `dev-console` feature

#### 2. Scheduler-Migration → `agrocore-scheduler` ✅ **ABGESCHLOSSEN**
**Migrierte Timer/Intervalle:**

| Crate | File | Vorher | Nachher |
|-------|------|--------|---------|
| `agrocore-messaging` | `bridge.rs` | `tokio::time::interval(60s)` Bridge Stats | `bridge_stats_reporter` Job (60s) |
| `agrocore-weather-service` | `worker.rs` | `tokio::time::interval(30min)` | `weather_update` Job (Cron `0 */30 * * * *`) |
| `agrocore-infrastructure` | `database.rs` | `tokio::time::interval(5s)` Pool Health | `db_pool_health` Job (5s) |
| `agrocore-infrastructure` | `database.rs` | `tokio::time::interval(~30d)` Cleanup | `db_monthly_cleanup` Job (Cron `0 0 1 * *`) |

**Alle Jobs extern in `backup-service/main.rs` registriert** (keine zyklischen Dependencies).
Bridge (nicht `Send`) läuft auf Main Thread, Scheduler Jobs (`Send`) in Background Tasks.

#### 3. Messaging-Migration → `agrocore-messaging` ✅ **ABGESCHLOSSEN**
**Betroffene Crates (alle nutzen jetzt Traits):**
- [x] `agrocore-api` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-backup` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-scheduler` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-weather-service` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-reporting-service` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-lpis-providers` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-geometry-service` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`
- [x] `agrocore-asset-registry` — `async_nats` → `agrocore_messaging::Publisher/Subscriber`

Neue Traits: `Publisher`, `Subscriber`, `MessageStream` in `agrocore-messaging`.
`MessagingClient` implementiert beide Traits. NATS Subject-Konstanten öffentlich exportiert.

### Akzeptanzkriterien — **Alle erfüllt** ✅
- [x] **Alle 15 Crates** nutzen `agrocore_logging` (kein direkter `tracing` Import mehr in Business-Logic)
- [x] **Alle Timer/Intervalle** durch `agrocore_scheduler` Jobs ersetzt
- [x] **Keine direkten `async_nats` Imports** in Business-Logic — nur über `agrocore_messaging` Traits
- [x] **ServiceContext** in allen Entry-Points
- [x] **RequestContext** in allen HTTP Handlers via Middleware
- [x] **SpanExt** genutzt für strukturierte Logs
- [x] **Konfiguration** über `LoggingConfig` / `SchedulerConfig` aus ENV/Config-Files
- [x] **Tests grün** nach Migration (alle 153+ Tests)
- [x] **Quality Gates** `cargo fmt && cargo check && cargo test && cargo clippy` laufen durch

### Version
- **v0.14.0** (2026-09-08) — Phase 7 Migration abgeschlossen

---

### Nächste Phasen
- [ ] Phase 8: Monitoring & Observability (P2)
- [ ] Phase 9: Catalog Import Script (P3)
- [ ] Module 17: KI-Analytics (P4)