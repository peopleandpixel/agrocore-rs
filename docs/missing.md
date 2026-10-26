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
- [ ] **E2** `per_page` ohne Obergrenze, Memory-DoS

### Block H — Admin UI erreicht große Teile des Backends nicht

- [ ] **H2** Keine Update-Funktionen in der UI — nur `user` und `worker` haben vollständigen CRUD; alle anderen Entitäten nur create und delete
- [ ] **H5** Neun API-Wrapper sind tot — `stock_in`, `stock_out`, `transfer_inventory`, `adjust_inventory`, `update_inventory_item`, `update_worker`, `refresh_token`, `logout`, `fetch_health` sind definiert, aber nirgends aufgerufen
- [ ] **H6** Logout-Button sendet keine Anfrage — `api::logout()` wird in `lib.rs:810-814` nicht aufgerufen, nur `clear_auth_token()` + `clear_user_role()` + `reload()`, JWT bleibt 30 Min gültig
- [ ] **H7** `livestock_new::configure` doppelt registriert, `livestock::configure` gar nicht — teilweise behoben (J1)
- [ ] **H8** Settings-Seite simuliert Funktionalität — LPIS-Sektion mit `<input>` ohne `prop:value`, ohne `on:input`, ohne Signal; Button ohne `on:click`; Timezone `<select>` ohne `on:change`; Version hardcoded als `"v0.4.2-stable"` (Projekt ist bei 0.59.0)
- [ ] **H10** Workforce Task-Progress unsichtbar — 5 Routen unter `/workforce/tasks/{id}/status*` nicht aufgerufen
- [ ] **H11** Lagerbuchung nicht möglich — `/inventory/stock-in`, `stock-out`, `transfer`, `adjust` nicht aufgerufen
- [x] **H12** Compliance und Weather-Bereiche ungenutzt — ✓ Teilweise behoben (26.10.2026): Fertilizer, Applicator-Lizenzen, Plant-Protection, Frost-Warnings, GDD, Pest-Risks haben jetzt UI-Seiten. Station-Management noch offen.
- [x] **H13** Weitere ungenutzte Bereiche — ✓ Teilweise behoben (26.10.2026): seasons, quotas, cold-chain, deliveries, olive-groves, vineyards, pest-risks, phenology, harvest, building, lots, agriculture haben jetzt UI-Seiten. profitability, workflow, material, stations, sources, data, summary, usage, difficulty-surcharge, tree-crown-volume, nitrogen-demand noch offen.

### Block I — Performance

- [ ] **I6** Audit-Trigger auf High-Frequency-Tabellen — `worker_locations`, `clock_entries`, `worker_task_statuses`, `task_data`, `weather_data`, `soil_moisture_readings`
- [ ] **I7** N+1-Patterns
- [ ] **I8** Fehlende Indexes
- [ ] **I9** `SELECT *` auf breiten Tabellen
- [ ] **I10** Queries ohne LIMIT
- [ ] **I11** Keine Kompression
- [ ] **I12** Kein `spawn_blocking`

### Block J — Unverdrahteter Code (weiter)

- [ ] **J8** `get_device_telemetry` immer leer, `send_command` lügt — Telemetry holt nichts, Command-Status wird ohne Publish gesetzt
- [x] **J11** Zwei Domain-Traits ohne Implementierung — ✓ Behoben (26.10.2026): `SoilMoistureReadingRepo` und `SoilMoistureAlertRepo` Postgres-Implementierungen erstellt, `SoilMoistureAlert` Entity an DB-Schema angepasst
- [ ] **J12** `depreciation` Feature kann nicht aktiviert werden — `mocks` jetzt behoben (M1), `depreciation` immer noch aus
- [ ] **J13** Dead configuration code — `mqtt_broker`, `rust_log` (`VisibilityAwareEntity` ✓ entfernt)
- [ ] **J14** `scheduler::stop()` und weitere Stubs — graceful shutdown nicht implementiert; dazu gehört auch `try_deliver()` Stub in `messaging/src/lib.rs:484` (gibt immer `Ok(())` zurück)
- [ ] **J15** Compliance-Filter laden die ganze Tabelle und filtern in Rust
- [ ] **J16** Düngerkosten fester Prozentsatz — `cost_eur: total_amount * 0.5` (Placeholder)

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
