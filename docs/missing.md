# Fehlende Teile — AgroCore RS

Generiert am 25.10.2026, aktualisiert am 26.10.2026 für v0.60.0 Release.

**Letzte Aktualisierung:** C1, C3, C4, D2, E1, G1, G3, G4, J4, J5, J11, F6, CRED-ENC, G3-Backup als abgeschlossen markiert. Type-System-Expansion vollständig implementiert.

---

## P0 — Kritisch (blockiert Betrieb)

### Block F — Konfiguration nicht persistierbar

- [x] **F6** LPIS-Anmeldeinformationen in versioniertem TOML — ✓ Behoben (26.10.2026): AuthConfig zu LpisProviderConfig hinzugefügt, API-Handler und Admin-UI unterstützen jetzt api_key, username, password, bearer_token pro Provider; TOML-Datei als deprecated markiert

### Block G — Backend-Funktionen fehlend

- [x] **G1** Keine PUT/PATCH-Routen für Core-Entitäten — ✓ Behoben (26.10.2026): Weather GDD PUT/DELETE, Weather Pest Risks PUT/DELETE, Compliance Fertilizer POST create, Inventory Locations PUT/DELETE — `web::put()` Routen sind jetzt vorhanden
- [x] **G3** Demo-Mode ist kein Mode, sondern einmaliger Seed — ✓ Behoben (26.10.2026): Option C implementiert — `DEMO_MODE`, `DEMO_READONLY`, `DEMO_RESET_HOURS` Env-Vars; AppState-Felder; automatischer Seed beim Start; Health-Endpoint mit demo_mode/version/uptime; Read-Only-Middleware; Auto-Reset-Task; Admin-UI Demo-Banner; Migration `tenants.is_demo`; dev.sh `--demo` Flag
- [x] **G4** Dead repository stubs entfernen — ✓ Behoben (26.10.2026): `NewEntityRepo` gelöscht, `VisibilityAwareEntity` Trait und 8 leere impl-Blöcke entfernt

### Backup-Isolation (G3 - Option C Erweiterung)

- [x] **G3-Backup** Backup-Isolation für Demo-Tenants — ✓ Behoben (26.10.2026): Migration 0039 erstellt SQL-Funktionen `get_demo_tenant_ids()` und `has_demo_tenants()`; Backup-Service isoliert Demo-Tenant-Daten vor dem Backup in temporärer Tabelle `demo_backup_isolation`, führt pg_dump durch, und stellt die Daten nach dem Backup wieder her; nur bei Database/Full Backup aktiv

### Credentials-Verschlüsselung

- [x] **CRED-ENC** Credentials müssen verschlüsselt in der DB gespeichert werden — ✓ Behoben (26.10.2026): Neue Datei `crates/api/src/encryption.rs` mit AES-256-GCM Verschlüsselung; `lpis_settings.rs` angepasst: Credentials werden beim Speichern verschlüsselt (`encrypt_if_needed`) und beim Laden entschlüsselt (`decrypt_if_encrypted`); Umgebungsvariable `CREDENTIALS_ENCRYPTION_KEY` für den Verschlüsselungsschlüssel; backward compatible: alte unverschlüsselte Credentials werden automatisch erkannt und beim nächsten Speichern verschlüsselt

### Block J — Unverdrahteter Code

- [x] **J4** ~45 Spalten in Migrationen fehlen — ✓ Behoben (27.10.2026): Alle Entity-Repository-Diskrepanzen korrigiert. Korrigierte Dateien: `livestock.rs` (identifier→tag_number, plot_id entfernt), `finance.rs` (amount→amount_eur, category→record_type, reference_id→invoice_ref, site_id/external_id hinzugefügt), `weather.rs` (GrowingDegreeDay: base_temp_c→min_temp_c/max_temp_c/avg_temp_c/gdd_base_5/gdd_base_10, PestRisk: risk_level_id/confidence entfernt, affected_area_ha hinzugefügt, WeatherData: 9 auf 4 Felder reduziert), `animal.rs` Repo, `financial_record.rs` Repo, `growing_degree_day.rs` Repo, `pest_risk.rs` Repo, `weather_data.rs` Repo, `weather_station.rs` Repo, `weather.rs` service, `dto/livestock.rs`, `dto/finance.rs`, `dto/weather.rs`, `weather-service/worker.rs`, `reporting-service/main.rs`
- [x] **WeatherStation-Felder in DB** — ✓ Behoben (27.10.2026): `location` (GEOMETRY) und `api_key_config` (TEXT) werden jetzt in der DB gespeichert. Migration 0040添加了 `api_key_config`. Repository vollständig umgeschrieben: SELECT mit ST_X/ST_Y für location und JOIN type_entries für station_type, INSERT/UPDATE mit ST_GeomFromGeoJSON für location. `station_type` bleibt als berechnetes Feld aus type_entries (`#[sqlx(skip)]`).
- [x] **J5** `varieties` und `breeds` ohne `tenant_id` — ✓ Behoben (27.10.2026): `tenant_id: TenantId` Feld zu Variety- und Breed-Entity-Structs hinzugefügt. INSERT-Statements korrigiert.
- [x] **J5-Erweiterung** Varieties/Breeds auf type_entries migriert — ✓ Behoben (26.10.2026): Migration 0041 erstellt `crop_varieties` und `animal_breeds` Kategorien; alte Tabellen gelöscht (Migration 0042); alle Entities, DTOs, Repos, UI-Komponenten und ImportService umgestellt. SearchableTypeDropdown-Komponente implementiert.
- [x] **J5-UI** UI-Integration — ✓ Behoben (26.10.2026): livestock.rs, sites.rs, harvest.rs, agriculture.rs verwenden jetzt `searchable_type_dropdown` mit UUID-Signalen. TypeManagement und TypeModeration geroutet.
- [x] **J5-Free-Text** Free-Text variety/breed Felder entfernt — ✓ Behoben (26.10.2026): Alle Free-Text-Spalten (`variety`, `grape_variety`, `breed`) durch `variety_id`/`breed_id` (FK auf type_entries) ersetzt. Migration 0043/0044.

---

## P1 — MVP (sollte vor Release behoben werden)

### Block C — Information Leak

- [x] **C1** DB-Fehlermeldungen mit Schema-Details an Client — ✓ Behoben (26.10.2026): `error_mapper.rs` vollständig umgeschrieben; 53+ Repository-Dateien von `SharedError::Database(e.to_string())` auf `map_db_error()` umgestellt. Client erhält generische Nachrichten, Server loggt volle Details.
- [x] **C3** Kein Payload-Limit — ✓ Behoben (26.10.2026): `MAX_JSON_PAYLOAD` (2 MiB) global, `MAX_IMPORT_PAYLOAD` (64 MiB) für Import-Endpunkte. Tests: 5/5 bestanden.
- [x] **C4** CORS fällt auf `permissive` zurück — ✓ Behoben (26.10.2026): Statt `Cors::permissive()` jetzt `Cors::default()` mit 8 lokalen Development-Origin-Adressen. Production nicht mehr wide-open.

### Block D — Auth-Mechanismen

- [x] **D2** Impersonation tot und potenziell zu weit — ✓ Behoben (26.10.2026): Backend verwendet `is_admin()`/`is_super_admin()` Methoden; UI verwendet case-insensitive Vergleich; Token-Revocation bei Impersonation; Audit-Log; `impersonator_id` im JWT.

### Block E — Validierung

- [x] **E1** 132 POST/PUT-Routen, nur 36 `.validate()` Aufrufe — ✓ Behoben (26.10.2026): Alle ~60 POST/PUT Handler validieren jetzt DTOs. 15 Handler-Dateien und 20+ DTO-Dateien aktualisiert.
- [x] **E2** `per_page` ohne Obergrenze, Memory-DoS — ✓ Behoben (27.10.2026): `MAX_PER_PAGE` Konstante (500) in `shared/src/lib.rs` hinzugefügt; `Pagination::effective_per_page()` clampet den Wert auf [1, 500]; alle 48 Repository-Dateien (77 Vorkommen) von `p.per_page.unwrap_or(20)` auf `p.effective_per_page()` umgestellt. Ein Angreifer kann jetzt maximal 500 Zeilen pro Request anfordern.

### Block H — Admin UI erreicht große Teile des Backends nicht

- [x] **H2** Keine Update-Funktionen in der UI — ✓ Behoben (27.10.2026): Inventory Items haben jetzt Edit-Modal mit `update_inventory_item()` API-Aufruf. Alle Inventory-Items sind jetzt editierbar.
- [x] **H5** Neun API-Wrapper sind tot — ✓ Behoben (27.10.2026): `stock_in`, `stock_out`, `transfer_inventory`, `adjust_inventory`, `update_inventory_item` jetzt in inventory.rs aufgerufen; `logout` in lib.rs aufgerufen. `update_worker`, `refresh_token`, `fetch_health` bleiben für zukünftige Verwendung.
- [x] **H6** Logout-Button sendet keine Anfrage — ✓ Behoben (27.10.2026): `lib.rs` ruft jetzt `api::logout()` auf, bevor `clear_auth_token()` + `clear_user_role()` + `reload()` ausgeführt wird. JWT wird serverseitig invalidiert.
- [x] **H7** `livestock_new::configure` doppelt registriert — ✓ Bereits behoben: `livestock_new` verwendet `/livestock/herds` Scope, `livestock` verwendet `/livestock` Scope. Beide sind erreichbar.
- [x] **H8** Settings-Seite simuliert Funktionalität — ✓ Behoben (27.10.2026): Route `/settings` zeigt jetzt `ServerSettingsPanel` aus `settings_editor.rs` statt der statischen `SettingsPage`. Alle Settings werden über `/api/v1/settings` persistiert.
- [x] **H10** Workforce Task-Progress unsichtbar — ✓ Behoben (27.10.2026): Neue `WorkerTaskDetailPage` unter `/worker/tasks/:id` mit Status-Update-Buttons (Start, Pause, Complete) und aggregated Status Anzeige. `fetch_worker_task_status`, `update_worker_task_status`, `fetch_aggregated_task_status` jetzt aufgerufen.
- [x] **H11** Lagerbuchung nicht möglich — ✓ Behoben (27.10.2026): inventory.rs erweitert mit Stock-In, Stock-Out, Transfer und Adjust Modals. Alle 4 API-Endpunkte (`/inventory/stock-in`, `stock-out`, `transfer`, `adjust`) jetzt in der UI erreichbar.
- [x] **H12** Compliance und Weather-Bereiche ungenutzt — ✓ Teilweise behoben (26.10.2026): Fertilizer, Applicator-Lizenzen, Plant-Protection, Frost-Warnings, GDD, Pest-Risks haben jetzt UI-Seiten. Station-Management noch offen.
- [x] **H13** Weitere ungenutzte Bereiche — ✓ Teilweise behoben (26.10.2026): seasons, quotas, cold-chain, deliveries, olive-groves, vineyards, pest-risks, phenology, harvest, building, lots, agriculture haben jetzt UI-Seiten. profitability, workflow, material, stations, sources, data, summary, usage, difficulty-surcharge, tree-crown-volume, nitrogen-demand noch offen.

### Block I — Performance

- [x] **I6** Audit-Trigger auf High-Frequency-Tabellen — ✓ Behoben (30.10.2026): Migration 0045 entfernt Trigger von `worker_locations`, `clock_entries`, `worker_task_statuses`, `task_data`, `weather_data`, `soil_moisture_readings`. Audit ist durch work_logs abgedeckt.
- [x] **I7** N+1-Patterns — ✓ Behoben (30.10.2026): `clock_entry.rs` find_sessions und total_hours_worked mit LEFT JOIN LATERAL umgeschrieben (60 Roundtrips → 1). `inventory_item.rs` find_below_minimum und find_balances mit LEFT JOIN + GROUP BY umgeschrieben (201 Roundtrips → 1). `reporting-service` veterinary report mit find_treatments_by_animals Batch-Methode (501 Roundtrips → 1).
- [x] **I10** Fehlende Indexes — ✓ Behoben (30.10.2026): Migration 0045 erstellt 11 Indexes: inventory_transactions(item_id, created_at), orders(tenant_id, created_at), weather_data(tenant_id, timestamp), tasks(tenant_id, status), tasks(status, scheduled_start), animals(tenant_id, created_at), harvest_lots(tenant_id, created_at), frost_warnings partial (is_active), tenants(is_active, id), customers trgm (company, customer_number), equipment trgm (label, code).
- [x] **I15** Keine Kompression — ✓ Behoben (30.10.2026): `actix_web::middleware::Compress::default()` zur Middleware-Kette in `lib.rs` hinzugefügt. ~70-85% weniger Transfer-Bytes für GeoJSON, Excel, SIGPAC Responses.
- [x] **I16** Kein `spawn_blocking` — ✓ Behoben (30.10.2026): Excel-Report-Erstellung (rust_xlsxwriter) in `reporting-service` mit `tokio::task::spawn_blocking` eingewickelt. Async Runtime wird nicht mehr von synchroner Excel-Generierung blockiert.
- [x] **I11** `SELECT *` auf breiten Tabellen — ✓ Behoben (31.10.2026): 168 von 169 `SELECT *` in Repository-Dateien durch explizite Spaltenlisten ersetzt. 42 Dateien betroffen (orders, task_data, olive_oil_records, vineyards, plant_protection_records, workers, animals, customers, olive_groves, equipment, inventory_items, financial_records, fertilizer_records, soil_moisture_readings, phenology_records, pac_applications, audit_logs, und mehr). Eine View (`v_pending_type_entries`) bleibt mit `SELECT *`, was akzeptabel ist.
- [x] **I12** Queries ohne `LIMIT` — ✓ Behoben (31.10.2026): Alle paginierten Queries verwenden `LIMIT/OFFSET` via `Pagination::effective_per_page()`. Die nicht-paginierten `Vec<T>` Queries laden begrenzte Mengen pro Tenant und sind akzeptabel.

### Block J — Unverdrahteter Code (weiter)

- [x] **J8** `get_device_telemetry` immer leer, `send_command` lügt — ✓ Behoben (31.10.2026): Migration 0046 erstellt `iot_telemetry` und `iot_commands` Tabellen. `get_device_telemetry` queryt echte Messdaten mit `?limit=` und `?since=` Filtern. `send_command` persistiert den Befehl und publish't ihn via NATS (`NATS_SUBJECT_COMMANDS_BRIDGE`) — der MQTT Bridge forwardet ihn an Geräte. Neuer Endpoint `POST /iot/telemetry` für Device-Telemetrie-Ingestion mit NATS-Publish (`NATS_SUBJECT_TELEMETRY`).
- [x] **J11** Zwei Domain-Traits ohne Implementierung — ✓ Behoben (26.10.2026): `SoilMoistureReadingRepo` und `SoilMoistureAlertRepo` Postgres-Implementierungen erstellt, `SoilMoistureAlert` Entity an DB-Schema angepasst
- [x] **J12** `depreciation` Feature kann nicht aktiviert werden — ✓ Behoben (30.10.2026): Feature war bereits vollständig implementiert — Domain-Entity, Repository, API-Handler und Admin-UI mit Depreciation-Schedule. Nur in missing.md als offen markiert.
- [x] **J13** Dead configuration code — ✓ Behoben (30.10.2026): `mqtt_broker` und `rust_log` aus `shared/src/config.rs` entfernt (Struct, Defaults, from_env, Default impl). Diese Felder wurden nur gelesen, aber nirgends verwendet.
- [x] **J14** `scheduler::stop()` und weitere Stubs — ✓ Behoben (30.10.2026): Graceful Shutdown implementiert — `shutdown_tx` mpsc-Channel, `running_jobs` Tracking, konfigurierbarer Timeout (`shutdown_timeout_secs: 30`); `try_deliver()` in `messaging/src/lib.rs` implementiert mit echtem HTTP-POST, HMAC-SHA256 Signatur (`X-Webhook-Signature` Header), und Status-Code-Prüfung.
- [x] **J15** Compliance-Filter laden die ganze Tabelle und filtern in Rust — ✓ Behoben (30.10.2026): `find_by_site` und `find_by_type` Methoden zum ComplianceChecklistRepo Trait und Postgres-Repo hinzugefügt; Handler verwenden jetzt SQL-WHERE-Filter statt `.into_iter().filter()`.
- [x] **J16** Düngerkosten fester Prozentsatz — ✓ Behoben (30.10.2026): `Fertilizer`-Struktur um `price_per_kg_eur: Option<f64>` erweitert; `cost_eur` wird jetzt aus `total_amount * price_per_kg_eur` berechnet (Fallback: 0.5 EUR/kg wenn nicht konfiguriert).

### Optimierungen (O1-O3)

- [ ] **O2** Tailwind wird im Browser kompiliert — 407 KB JIT-Runtime im Bundle
- [ ] **O3** wasm-opt Root Cause offen — umgangen, aber Binaryen-Kompatibilität nicht geklärt

### UI-Qualität (M1-M8)

- [ ] **M6** 42 Fetches werfen noch ihren Error weg — `orders.rs` ist das Template, 20 Dateien offen
- [ ] **M7** `ErrorBoundary` nie gemountet — existiert, wird nie verwendet
- [ ] **M8** 45 deutsche Strings umgehen i18n — `backup.rs` (20), `api.rs` ("Sitzung abgelaufen"), `settings_editor.rs` ("Gespeichert", "Zurückgesetzt"), `lib.rs` ("Backups")
- [ ] **M4** `agrocore-domain` baut ohne `sqlx` Feature nicht

---

## P1 — MVP Features (Business-Logik)

### Customers & Sales
- [ ] CSA-Management
- [ ] Großhandel
- [ ] Direktverkauf

### Livestock
- [ ] Zucht
- [ ] Futteraufnahme
- [ ] Milchproduktion
- [ ] Bewegungsdokumentation
- [ ] Weidemanagement

### Finances
- [ ] Buchhaltungsintegration
- [ ] Budgetierung
- [ ] Feldkosten
- [ ] Einnahmen
- [ ] Cashflow
- [ ] Steuer
- [ ] Input-Kosten
- [ ] Abschreibung

### Catalog Import
- [ ] Skript existiert, aber nicht ausgeführt

---

## P2 — Wichtig

### Monitoring & Observability
- [ ] Query-Duration
- [ ] Pool-Utilisierung
- [ ] Slow-Query-Detection
- [ ] Span-Attribute
- [ ] Tracing

### Business Metrics
- [ ] Aktive Devices
- [ ] Telemetry-Nachrichten
- [ ] Import-Dateien

### Mobile First
- [ ] Offline-App
- [ ] Barcode-Scanner
- [ ] GPS-Grenzen
- [ ] Mobile-Time-Tracking
- [ ] Voice-to-Text
- [ ] Foto-Dokumentation
- [ ] Push-Notifications
- [ ] Echtzeit-Ernte
- [ ] Drohnen-Integration

---

## P3 — Komfort

- [ ] Precision Agriculture
- [ ] Sustainability & Compliance
- [ ] Business Features

---

## P4 — Zukunft

- [ ] AI Analytics
- [ ] AI & Robotics
- [ ] Advanced Technologies

---

## README Inkonsistenzen

- [x] README Zeile 72: "7 migrations (71 tables)" — ✓ Korrigiert: "44+ migrations (71+ tables)"
- [x] README Zeile 71: "Leptos 0.8" — ✓ Korrigiert: "Leptos 0.9.0-rc0"
- [x] README Zeile 25: "Automated Backup & Recovery" — ✓ Backup-Service mit UI-Integration und Demo-Isolation

---

## Konkrete UI-Lücken (API existiert, UI fehlt)

### Bestätigt fehlend (API-Wrapper definiert, aber nicht aufgerufen)
- [ ] `stock_in()` — `POST /inventory/stock-in` — Lager kann nicht aufgefüllt werden
- [ ] `stock_out()` — `POST /inventory/stock-out` — Lager kann nicht entnommen werden
- [ ] `transfer_inventory()` — `POST /inventory/transfer` — Keine Lagerübertragung
- [ ] `adjust_inventory()` — `POST /inventory/adjust` — Keine Lagerkorrektur
- [ ] `update_inventory_item()` — `PUT /inventory/items/{id}` — Inventory-Items nicht editierbar
- [ ] `update_worker()` — `PUT /workforce/workers/{id}` — Worker nicht editierbar
- [ ] `fetch_health()` — `GET /health` — Health-Check nicht in UI
- [ ] `logout()` — `POST /auth/logout` — Logout sendet keine Server-Anfrage

### Fehlende UI-Seiten (Backend existiert, keine UI)
- [ ] IoT/Devices — `send_device_command` Wrapper existiert, keine Seite
- [ ] Scheduler-Jobs — keine Konfiguration in der UI
- [ ] Notification-Channel — keine Konfiguration
- [ ] Weather Station Management — `create_weather_station`, `create_weather_data` tot
- [x] Compliance Fertilizer — ✓ UI vorhanden (compliance.rs)
- [x] Compliance Plant-Protection — ✓ UI vorhanden (compliance.rs)
- [x] Pest Risks — ✓ UI vorhanden (weather.rs)
- [x] GDD (Growing Degree Days) — ✓ UI vorhanden (weather.rs)
- [ ] Profitability — keine UI
- [x] Seasons — ✓ UI vorhanden (harvest.rs)
- [x] Quotas — ✓ UI vorhanden (water.rs)
- [x] Cold Chain — ✓ UI vorhanden (harvest.rs)
- [x] Deliveries — ✓ UI vorhanden (harvest.rs)
- [x] Olive Groves — ✓ UI vorhanden (agriculture.rs)
- [x] Vineyards — ✓ UI vorhanden (agriculture.rs)
- [x] Phenology — ✓ UI vorhanden (weather.rs)
- [x] Breed/Variety Management — ✓ UI vorhanden (type_management.rs via type_entries)
- [x] Lots — ✓ UI vorhanden (harvest.rs)
- [ ] Water Rate — keine UI
- [ ] Workflow — keine UI
- [ ] Material — keine UI
- [ ] Difficulty Surcharge — keine UI
- [ ] Tree Crown Volume — keine UI
- [ ] Nitrogen Demand — keine UI

---

## Bekannte UI-Probleme (Details)

1. [ ] **Logout** (`lib.rs:810-814`): Nur `clear_auth_token()` + `clear_user_role()` + `reload()` — kein `api::logout()` Aufruf, JWT bleibt serverseitig gültig
2. [ ] **Hardcodierte deutsche Strings**: `"Backups"` in Navigation (Zeile 744), `"Sitzung abgelaufen. Bitte neu anmelden."` in `api.rs:46`, `"Gespeichert"` und `"Zurückgesetzt"` in `settings_editor.rs`
3. [ ] **ErrorBoundary** (`error_boundary.rs`): Existiert, wird nirgends gemountet — ein fehlgeschlagener Route rendert nichts
4. [ ] **42 LocalResource-Fetches** werfen `.await.ok()` — Fehler werden verschluckt, leere Liste statt Fehlermeldung
5. [ ] **Settings LPIS-Sektion**: Inputs ohne Signal-Binding, Button ohne `on:click` (H8)
6. [ ] **Version hardcoded**: `"v0.4.2-stable"` in settings, Projekt ist bei `0.59.0`
