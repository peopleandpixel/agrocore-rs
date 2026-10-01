# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.25.0] - 2026-10-01

Behebt die Privilege Escalation aus dem Audit vom 2026-10-01 (`tasks.md` A1, D3) —
die am leichtesten ausnutzbare Schwachstelle des Projekts.

### Fixed
- **Privilege Escalation: jeder User konnte sich zum Admin machen** — `handlers/users.rs:153-157`. Die Autorisierung las:

  ```rust
  if let Err(e) = auth.require_admin()
      && auth.0.user_id != user_id
  { return Err(e.into()); }
  ```

  Die Admin-Prüfung entfiel, sobald der Ziel-Account die eigene ID war. Da `UpdateUserDto` ein `roles`-Feld mitbringt und `PgUserRepo::update` es ungeprüft bindet (`postgres/user.rs:318`), genügte `PUT /api/v1/users/{eigene_id}` mit `{"roles":["Admin"]}` — voller Admin-Zugriff beim nächsten Login. Nicht sichtbar, weil die Bedingung in der negativen Testform eine Begründung für sich zu haben scheint.

  Jetzt gilt: Rollenwechsel und Änderung von `is_active` erfordern Admin, unabhängig vom Ziel. Ein Self-Service-Pfad mit eng definierter Feldliste ersetzt die bisherige Möglichkeit, den eigenen Account zu bearbeiten.

- **Passwort-Update ohne Mindestlänge** — `dto/user.rs`. `UpdateUserDto.password` hatte keine `validate`-Angabe, während `CreateUserDto` `min = 8` setzte. Über die Eskalation konnte ein bestehendes Passwort damit auf einen leeren String gesetzt werden. Beide Update-Pfade verlangen jetzt 12 bis 128 Zeichen; `CreateUserDto` bleibt bei 8, damit bestehende Konten gültig bleiben.

### Added
- **`PUT /api/v1/users/me`** — `handlers/users.rs:update_own_profile` mit `UpdateOwnProfileDto`. Nimmt ausschließlich `firstname`, `lastname`, `password`, `language` und `color` entgegen. `roles`, `is_active`, `internal_cost_per_hour` und `external_cost_per_hour` existieren im DTO nicht; die Zuordnung auf den Domain-DTO setzt alle übrigen Felder explizit auf `None`, damit `PgUserRepo::update` die Spalten unangetastet lässt statt sie zu überschreiben. Die Route ist bewusst **vor** `/users/{id}` registriert — sonst hätte das `{id}`-Muster den Pfad „me" als UUID zu parsen versucht.
- **Neun Regressionstests** (`crates/api/tests/privilege_escalation_tests.rs`) — DTO-Ebene: die Abwesenheit privilegierter Felder im Self-Service-DTO, Passwort-Policy auf beiden Update-Pfaden (leer, ein Zeichen, 22 Zeichen, nicht gesetzt), und dass die bestehende E-Mail-Validierung durch die Änderung nicht abgeschwächt wurde.

### Tests
- 258 Tests im Workspace, 0 Fehler (249 + 9 neue).

## [0.24.0] - 2026-10-01

Erster Teil des Code-Audits vom 2026-10-01 (Block I und J). Behebt den schwerwiegendsten
Befund: acht Tabellen wurden von fertig implementierten Repositories abgefragt, existierten
aber in keiner Migration. Zusätzlich wurde ein Sicherheitsaudit dokumentiert
(`docs/tasks.md`, Blöcke A–J).

### Fixed
- **Acht Tabellen fehlten im Schema, sieben Repos waren zur Laufzeit tot** — `spatial_objects`, `groups`, `trees`, `buildings`, `livestock`, `water_usages`, `animal_treatments`, `animal_grazing_records`. Die Repos waren vollständig implementiert, keine Migration legte die Tabellen an, jede Query scheiterte mit `relation "..." does not exist`. Migration `0000000003_missing_domain_tables.sql` angelegt: sechs Tabellen mit Indizes, GIST-Geometrieindizes, RLS-Policies nach bestehendem Muster und `ALTER TABLE water_usage RENAME TO water_usages`.
- **`spatial_objects` wurde von jedem GPS-Ping abgefragt, der Fehler war unsichtbar** — `handlers/workforce.rs:353,364` rief `spatial_object_repo().find_containing_point(...)` mit `.unwrap_or_default()`. Jeder Standort-Ping lief zweimal in eine nicht existierende Tabelle, das Ergebnis war immer leer: die Standort-zu-Feld-Zuordnung funktionierte nie und fiel nicht auf. Jetzt wird der Fehler mit `warn!` protokolliert (Tenant, Koordinaten, Fehlertext); der Ping läuft weiter, aber der Fehler ist sichtbar.
- **INSERT-Statements haben `tenant_id` nicht gebunden** — `tree.rs`, `group.rs`, `building.rs`, `livestock.rs`. Alle SELECTs filtern mit `WHERE tenant_id = $1`, der INSERT ließ die Spalte weg — ein neu angelegter Datensatz wäre nicht mehr auffindbar gewesen. Nicht sichtbar, weil die Methodensignatur `tid: TenantId` korrekt aussah. Vier Queries und Bind-Reihenfolgen korrigiert.
- **Falsche Tabellennamen in `animal.rs`** — abgefragt wurden `animal_treatments` und `animal_grazing_records`, vorhanden sind `treatment_records` (Migration `:676`) und `grazing_records` (`:665`); zusätzlich schrieb das Repo `treatment_date` statt `date`. Auf die vorhandenen Tabellen umgestellt, kein neues Schema nötig.
- **`animals` fehlten drei Spalten, die das Repository liest** — `identifier`, `livestock_type` und `status`. `identifier` wird aus `tag_number` gebackfillt. Für `livestock_type` genügt kein Default, weil vorhandene Inserts (inklusive Demo-Seed) nur `species` setzen: ein BEFORE-Trigger leitet `livestock_type` bei jedem INSERT und UPDATE aus `species` ab. Ein erster Versuch mit `SET NOT NULL` brach den Demo-Seed (`null value in column "livestock_type"`).

### Fixed
- **Der Test-Fixture startete ein Image ohne PostGIS** — `crates/infrastructure/tests/common/mod.rs` verwendete `testcontainers_modules::postgres`, das fest auf `postgres:11-alpine` verdrahtet ist und kein PostGIS enthält. Migration `0000000000` erstellt aber die `postgis`-Extension, deshalb scheiterten **alle neun** Integrationstests an `extension "postgis" is not available` — sie konnten nie gelaufen sein. Auf `GenericImage::new("postgis/postgis", "16-3.4")` umgestellt (das Modul bietet kein `with_tag()`), und den Connect mit Backoff plus Retries versehen, weil Postgres während der Init-Phase einmal neu startet und laufende Verbindungen zurücksetzt.

### Added
- **Drei Regressionstests** (`crates/infrastructure/tests/database_setup_tests.rs`) — `test_repository_tables_exist` prüft jede von einem Repository abgefragte Tabelle, `test_tenant_scoped_tables_have_tenant_id` findet Tabellen ohne Mandantenbezug, `test_new_domain_rows_are_tenant_scoped` legt Zeilen in allen vier neuen Tabellen an und liest sie über den Tenant-Filter zurück. Alle drei schlagen bei Rückkehr des ursprünglichen Zustands fehl.
- **Tabellenliste im Migrations-Test erweitert** — der bestehende `test_database_migrations_applied` prüfte `spatial_objects` bereits und wäre durch die Migration jetzt grün; `groups`, `trees`, `buildings`, `livestock` und `water_usages` ergänzt.

### Changed
- **RLS-Policies auf den neuen Tabellen** nach dem bestehenden Muster über `get_current_tenant_id()` ergänzt. Sie greifen aus demselben Grund nicht wie die übrigen 190: `app.current_tenant_id` wird nirgends gesetzt und `FORCE ROW LEVEL SECURITY` fehlt (siehe `tasks.md` A3). Die Policies sind aus Konsistenzgründen da, nicht als Garantie.

### Verified
- Alle vier Migrationen laufen in einer frischen PostgreSQL-Instanz in Reihenfolge durch.
- Migration `0000000003` ist dreimal hintereinander auf derselben Datenbank gelaufen, ohne Duplikate.
- Der Demo-Seed läuft nach der Migration durch und legt 3 Tiere, 3 Sites und 6 Grazing-Records an.
- Alle Repository-Queries gegen das neue Schema ausgeführt: `spatial_objects` (find_by_id, count, GPS-Ping-Abfrage), `trees`, `groups`, `buildings`, `livestock` (je count und INSERT), `water_usages`, `animals`, `treatment_records`, `grazing_records`.
- Der GPS-Ping liefert erstmals Daten; `livestock_type` wird korrekt aus `species` abgeleitet (`Cattle -> Cattle`).

### Known Limitations
- Vier vorbestehende Integrationstests in `database_setup_tests.rs` scheitern weiterhin, unabhängig von dieser Änderung: `test_database_migrations_applied` erwartet eine Tabelle `spatial_properties`, die keine Migration anlegt; `test_site_crud_operations` bricht mit `INSERT has more target columns than expressions`; `test_tenant_creation_and_isolation` erzeugt pro Test einen Tenant mit festem Slug `test-tenant` und scheitert am Unique-Constraint, sobald mehr als ein Test denselben Slug nutzt; `test_tenant_scoped_tables_have_tenant_id` findet Tabellen ohne `tenant_id`. `test_updated_at_trigger` ist nach dem Fixture-Fix ebenfalls rot. Siehe `tasks.md` J19 und J4.
- Der Fehler in `workforce.rs` wird geloggt, nicht behoben. Eine fehlgeschlagene Geometrie-Abfrage führt weiterhin zu keiner Standort-Zuordnung — nur ist es jetzt sichtbar.
- `work_logs` und rund 40 weitere Spalten fehlen weiterhin (`tasks.md` J4).
- `varieties` und `breeds` haben weiterhin keine `tenant_id`-Spalte, ihre Repos filtern aber danach — Varianten und Rassen sind global statt mandantenisoliert (`tasks.md` J5).

## [0.23.0] - 2026-10-01

Backup-Service Phase 8 Abschnitt 4: Streaming-Pipeline, funktionale Retention, echtes Restore, Monitoring-Metriken sowie die bislang nur konfigurierten Backends SFTP und WebDAV.

### Added
- **Streaming-Backup-Pipeline** — `pg_dump` liest stdout über einen 1-MiB-Puffer statt `cmd.output()`. Bei Dumps im zweistelligen GB-Bereich war das ein OOM-Risiko, weil der komplette Dump gleichzeitig im Speicher lag.
- **Streaming-Restore** — `pg_restore` bekommt die Daten direkt über stdin. `restore_from_storage` lädt den Dump nicht mehr komplett via `download_bytes()`.
- **`StorageBackendTrait` erweitert** — neue Methoden `upload_stream`, `download_stream`, `list_objects`, `delete_object`, `load_manifest` und `save_manifest`. Cloud-Backends nutzen `object_store::put_multipart` für chunkweises Hochladen.
- **Prometheus-Metriken** (`crates/backup-service/src/metrics.rs`) — `backup_duration_seconds`, `backup_size_bytes`, `backup_success_total`, `backup_failed_total` und `backup_restore_total`. Verknüpft mit Erfolg, Fehlschlag, Dauer, Größe und Restore-Ergebnis. 4 Unit-Tests.
- **SFTP-Backend** (`crates/backup-service/src/sftp_backend.rs`) — Passwort- und Private-Key-Authentifizierung über `russh`/`russh-sftp`, Streaming-Upload mit 1-MiB-Chunks, Download, rekursives Listing, Löschen und Manifest-Persistenz.
- **WebDAV-Backend** (`crates/backup-service/src/webdav_backend.rs`) — Basic Auth, `MKCOL` für Collections, Streaming-Uput über `PUT` mit Chunked Transfer Encoding, `PROPFIND` für rekursives Listing inklusive XML-Parsing, `DELETE`, Manifest-Persistenz. 11 Unit-Tests für URL-Konstruktion, Pfad-Encoding und Response-Parsing.
- **Verschlüsselung für beide neuen Backends** — `encrypt_payload`/`decrypt_payload` in `encryption.rs` wenden die konfigurierte Target-Verschlüsselung an. Der Streaming-Upload verschlüsselt chunkweise, damit der Speicherbedarf begrenzt bleibt.
- **`dry_run` für Restore** — API-Request (`RestoreRequest.dry_run`) und CLI (`agrocore-backup restore <ID> --dry-run`) prüfen, ob ein Backup wiederherstellbar ist, ohne in die Datenbank zu schreiben.
- **14 neue Tests** — 8 Storage-Streaming, 7 Retention, 4 Manifest, 3 echte PostgreSQL-Integrationstests.

### Fixed
- **Retention konnte nichts löschen** — `parse_dump_timestamp` nahm mit `rsplit_once('_')` das letzte `_` und isolierte damit `HHMMSS`; der anschließende Split konnte nie gelingen, die Funktion lieferte für `dump_YYYYMMDD_HHMMSS.dump` immer `None`. Der Parser wertet jetzt die letzten beiden Segmente aus.
- **`create_manifest` bekam eine leere Objektliste** — `targets: vec![]` wurde unverändert durchgereicht. Restore fand dadurch kein Dump-Objekt. Manifests werden jetzt mit den tatsächlich in Storage vorhandenen Objekten und Größen befüllt.
- **Restore identifizierte Backups per Dateinamen-Heuristik** — jetzt wird zuerst das persistierte Manifest gelesen, mit Listing-Fallback für Altbestände.
- **Restore schlug bei gemischten Client-/Server-Versionen fehl** — pg_dump 18.6 gegen PostgreSQL 16.4 erzeugt `SET transaction_timeout = 0`, das der Server nicht kennt. Nur dieser Fall wird als Warnung behandelt; echte `pg_restore: error:`-Zeilen lassen den Restore weiterhin fehlschlagen.
- **`list_objects` gab bei unbekannten Targets stillschweigend eine leere Liste zurück** (`_ => Ok(Vec::new())`). Retention hätte so Backup-Ausfälle als "nichts zu löschen" interpretiert. Der Fallback meldet jetzt einen Fehler.
- **`load_manifest` behandelte ein fehlendes Remote-Manifest als Fehler** — nur lokale Storage lieferte `NotFound`. Der Pfad prüft jetzt auch den HTTP-Status der Remote-Backends.
- **`_shared`-Hilfsfunktion ohne Aufrufer** in `webdav_backend.rs` entfernt.

### Changed
- **`list_objects`, `download_stream`, `upload_stream` und `delete_object` dispatchen jetzt explizit** an `BackupTarget::Sftp` und `BackupTarget::WebDAV`. Die vorherigen `warn!("... not yet implemented")`-Zweige sind entfernt.
- **Dump-Objektnamen enthalten die Job-ID** (`<uuid>_dump_<timestamp>.dump`). Retention parst weiterhin id-haltige Namen.
- **`russh` auf 0.49 gepinnt** — 0.54 zieht eine `base64ct`-Version, die mit `argon2`'s Anforderung kollidiert. Konfliktfreie Versionen haben Vorrang, wie im Workspace üblich.
- **`reqwest` um das `stream`-Feature erweitert** — `Body::wrap_stream` ist ohne dieses Feature nicht verfügbar.

### Tests
- 249 Tests im Workspace, 0 Fehler.
- 3 echte `pg_dump`/`pg_restore`-Tests gegen eine laufende PostgreSQL-Instanz, darunter ein Roundtrip mit Row-Count-Vergleich:
  ```bash
  DATABASE_URL=postgresql://agrocore:agrocore@localhost:5432/agrocore \
    cargo test -p agrocore-backup --test pg_dump_e2e_tests -- --ignored --test-threads=1
  ```

### Known Limitations
- Der Download-Pfad der `object_store`-Backends nutzt `GetResult::bytes()`, weil object_store 0.11 keinen asynchronen Byte-Stream für Downloads liefert. Cloud-Downloads sind deshalb weiterhin nicht speicherschonend; Local, SFTP und WebDAV streamen.
- Host-Key-Pinning für SFTP ist nicht implementiert; `check_server_key` akzeptiert jeden Schlüssel. Für den Produktivbetrieb sollte gegen eine `known_hosts`-Datei verifiziert werden.
- Age- und KMS-Verschlüsselung sind weiterhin nicht implementiert; die Verschlüsselung läuft über AES-256-GCM.
- Integrationstests gegen echte SFTP- und WebDAV-Server fehlen; getestet wurden Pfad-, Auth- und Response-Handling.
- NATS-Progress-Events (0–100 %) sind implementiert, aber nicht durch Integrationstests abgesichert.
- Das Disaster-Recovery-Runbook für 50 GB bei RTO < 15 Minuten ist noch offen.

## [0.22.0] - 2026-09-30

### Added
- **Notification Dispatcher** (`agrocore-messaging::notification`) — Der bereits vorhandene, aber nie eingebundene Dispatcher ist jetzt vollständig verdrahtet: `NotificationChannel`-Trait mit acht Kanälen (SMTP, SendGrid, Mailgun, Telegram, ntfy, Webhook, Twilio SMS, `wacli` WhatsApp), Template-Engine, exponentielles Backoff und Dead-Letter-Queue auf `notifications.failed`.
- **Fehlende Kerntypen** (`notification/types.rs`) — `ChannelConfig`, `ChannelMessage`, `NotificationChannel`, `ChannelError`, `DeliveryReport` sowie die Konfigurationsstructs pro Kanal. Das Modul referenzierte 12 Typen, die nie existiert haben.
- **`agrocore-notification-service`** — Service mit NATS-Consumer für `notifications.send`, Konfiguration aus YAML/JSON oder `NOTIFY_<CHANNEL>_<SETTING>`, `/health`-Endpoint und Graceful Shutdown. Läuft als Docker-Container.
- **Port-Konflikt-Erkennung** (`scripts/dev.sh`) — Reserviert Ports gegen Doppelvergabe innerhalb eines Laufs und schreibt die tatsächlichen Ports nach `.env.dev`.
- **Docker-Build-Caching** — BuildKit-Cache-Mounts, gezielte COPY-Schritte, `.dockerignore` und `cargo build --bin`. Rebuild von ~7 min auf ~2,5 s.
- **10 Notification-Tests** — Channel-Konstruktion, YAML-Roundtrip, Konfigurationsvalidierung, Readiness-Default und Dead-Letter-Konfiguration.

### Fixed
- **`Dockerfile.service` war funktional kaputt** — `CMD ["/app/${SERVICE_NAME}"]` expandiert Build-Argumente in Exec-Form nicht, der Container startete mit `exec: "/app/${SERVICE_NAME}": no such file or directory`. Neues Entrypoint-Skript löst den Service zur Laufzeit auf und erhält per `exec` PID 1, damit SIGTERM für Graceful Shutdown ankommt.
- **Runtime-Mismatch im Notification-Service** — `#[tokio::main]` startet Tokio, aber `actix_web::rt::spawn` benötigt ein `LocalSet`; der Dispatcher panickte mit `spawn_local called from outside of a task::LocalSet`. Auf `#[actix_web::main]` umgestellt.
- **Dev-Umgebung startete nie mit belegten Ports** — `docker-compose.dev.yml` nutzte durchgängig `network_mode: host`, es gab keine Port-Mappings, die berechneten Alternativports wurden nie angewendet. Auf Bridge-Netz mit `${VAR:-default}` umgestellt.
- **Port-Kollision im Fallback** — API, Admin UI und Notification landeten alle auf 8083, weil jede Prüfung denselben noch nicht gebundenen Port sah. Reservierungsliste eingeführt; die Zuweisung lief zudem in einer Command-Subshell, wodurch die Reservierung wirkungslos war.
- **`DATABASE_URL` enthielt den Literal-Platzhalter `***`** statt eines Passworts. Mit `network_mode: host` fiel das nicht auf, weil der Container die URL nie selbst auflöste. Jetzt `${POSTGRES_PASSWORD:-agrocore}`.
- **`agrocore-logging` Build Failure** — `lib.rs` re-exportierte `ServiceContextLayer` und `SpanExt` unbedingt, obwohl beide die optionale `tracing`-Abhängigkeit brauchen; Crates mit `default-features = false` (`admin-ui`, `dashboard`) schlugen fehl.
- **Migration schlug fehl** — `0000000000_consolidated_init.sql` rief `trigger_updated_at()` auf, das nie definiert war (korrekt: `set_updated_at()`). Durch `SKIP_MIGRATIONS=1` jahrelang verdeckt.
- **Demo-Seed entsprach nicht dem Schema** — `roles` war `text[]` statt JSONB, `site_type`/`crop_type`/`equipment_type` in veralteten Formaten; die Tabellen `orders`, `equipment`, `inventory_*`, `animals`, `grazing_records`, `customers`, `financial_records` hatten abweichende Spalten; `livestock` existiert nicht. Zwei UUIDs enthielten Nicht-Hex-Zeichen.
- **Demo-Passwörter waren ungültig** — der Seed speicherte bcrypt-Hashes, die Anwendung verifiziert mit Argon2id (`crates/infrastructure/src/postgres/user.rs`), Ergebnis `Invalid password hash: salt too short`. Hashes mit der `argon2 0.6`/`password-hash 0.6`-Version aus `Cargo.lock` neu erzeugt. `demo123` verletzte außerdem `min=8`, jetzt `demo1234`.
- **`sqlx::migrate!` lehnte den Seed ab** — psql-Syntax (`\set`, `:'var'`) wird von SQLx nicht ausgeführt; alle 79 Variablen durch echte UUID-Literale ersetzt.
- **Healthcheck-Logik invertiert** — `grep -q null` lieferte bei vorhandenen Healthchecks fälschlich „no healthcheck defined"; zusätzlich gab `wait_for_health` bei Timeout fälschlich Erfolg zurück.
- **nginx lauschte auf 8081** statt auf den gemappten Port 80, und der API-Proxy zeigte auf `localhost:8080` statt auf den Service-Namen `api` im Bridge-Netz.
- **TUI-Dashboard brach ohne TTY ab** — endete mit `interactive SLT runtime unavailable` und Exit 1; wird jetzt übersprungen, das Script bleibt aktiv.

### Changed
- **Demo-Seed nach `scripts/demo_seed.sql`** — `sqlx::migrate!` akzeptiert im Verzeichnis `migrations/` ausschließlich nummerierte Migrationen. Dadurch liefen die Demo-Daten bei jedem API-Start mit; der Seed ist jetzt opt-in über `--demo` bzw. `DEMO_MODE=true`.
- **Enum-Varianten** — `BackupTarget::GCS` → `Gcs` und `BackupTarget::SFTP` → `Sftp` (`clippy::upper_case_acronyms`). Das Wire-Format bleibt unverändert (`rename_all = "lowercase"`).
- **Demo-Modus** — unterstützt jetzt sowohl `DEMO_MODE=true ./scripts/dev.sh` als auch `./scripts/dev.sh --demo`; der Seed wartet auf `public.tenants` und nutzt `reset: true`.
- **Messaging-Dispatcher teilt Kanäle per `Arc<dyn NotificationChannel>`** statt `Clone` als Trait-Supertrait, das die Dyn-Kompatibilität verhindert hätte.

### Removed
- **Nicht existierende `livestock`-Tabelle** aus dem Demo-Seed entfernt.
- **Redundanter `DispatcherRef`-Wrapper** im Dispatcher, der nur einen zweiten `NotificationDispatcher` zum Delegieren konstruierte.

### Quality Gates
- `cargo fmt --all -- --check` ✅
- `cargo check --workspace --all-targets` ✅
- `cargo test --workspace` ✅ (211 passed, 0 failed)
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ (zero warnings)


## [0.21.1] - 2026-09-30

### Fixed
- **`agrocore-logging` Build Failure with `default-features = false`** — `lib.rs` re-exported `ServiceContextLayer` and `SpanExt` unconditionally, but both require the optional `tracing` dependency. Crates that disable default features (`admin-ui`, `dashboard`) failed to compile because the re-exported items did not exist. The gate was hidden by `cargo check -p agrocore-logging` passing standalone — it only surfaced through a dependent crate. Re-exports are now gated on the same features as their definitions.
- **Admin UI Mangled Absolute API URLs** — `api_url()` unconditionally joined the base with the path, rewriting external endpoints such as `https://api.open-meteo.com/v1/forecast` onto the local API base. Absolute URLs now pass through untouched.
- **Backup Service Duplicated the Entire Library** — `main.rs` re-declared all 10 modules (`mod config; mod encryption; …`) alongside a complete library target, producing two divergent copies of the code and a separate dead-code analysis that yielded 5 phantom errors. The binary now imports from the `agrocore_backup` library crate.
- **Local Backup Target Bypassed Registration** — `upload_bytes` / `download_bytes` read `target.path` directly instead of the registered `local_paths` table, leaving `find_local` dead. Both now resolve through the registration table, which removes the dead code and adds a validation check.
- **Duplicate Test Helpers in Domain Crate** — `point()` / `square()` were defined both inside and after `mod tests` in `domain/src/entities/spatial/mod.rs`; the trailing copies were removed.

### Removed
- **Placeholder Tests** — Removed three tests that could never fail: three `assert!(true, "…")` in `admin-ui/src/tests/api_error_handling.rs` and `assert!(x.is_ok() || x.is_err())` in the backup-service integration tests. Replaced with assertions on real behavior.
- **Unused Imports and Dead Arms** — Removed unused imports across `api`, `backup-service`, and `infrastructure`; removed an unreachable wildcard match arm in `backup-service/src/storage.rs`.

### Changed
- **Enum Variant Naming** — `BackupTarget::GCS` → `Gcs` and `BackupTarget::SFTP` → `Sftp` for Rust naming conventions (`clippy::upper_case_acronyms`). Wire format is unaffected: the enum uses `rename_all = "lowercase"`.
- **`sort_by` → `sort_by_key`** — Descending sorts in `backup-service/src/retention.rs` and `verification.rs` now use `std::cmp::Reverse` keys.

### Added
- **AES-256-GCM Encryption Test Coverage** — New `backup-service/tests/encryption_tests.rs` covering the previously untested encryption path: encrypt/decrypt round-trip fidelity, nonce uniqueness across identical inputs, GCM authentication rejection of tampered ciphertext, truncated-input rejection, plaintext passthrough mode, and error messages for the four unimplemented backends (Age, AWS KMS, Azure Key Vault, GCP KMS).
- **Admin UI API URL Tests** — Real assertions on `api_url` invariants: no doubled or dropped path separators, base-URL prefixing, absolute-URL preservation, and query-string retention. Verified against both configured-base and empty-base (nginx proxy) modes.
- **Strengthened Backup Config Tests** — `test_backup_config_validation_valid` now asserts both the failing case (default `Age` encryption with no recipients) and the passing case; `test_load_config_from_env` asserts a concrete outcome instead of `is_ok() || is_err()`.

### Quality Gates
- `cargo fmt --all -- --check` ✅
- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (201 passed, 0 failed, 60 suites)
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ (zero warnings)

## [0.21.0] - 2026-09-24

### Added
- **Spatial Type System Refactoring** — Complete extraction of spatial types (`GeoPoint`, `Boundary`, `Plot`, `RowConfig`, `SigpacData`, `LpisCountry`) to dedicated `spatial/types.rs` module to resolve circular dependencies
- **SQLx Postgres Support for All Spatial Types** — Full `Type`, `Encode`, `Decode` implementations for `GeoPoint`, `Boundary`, `RowConfig`, `SigpacData`, `LpisCountry`, `SiteType`, `CropType`, `LpisParcel` enabling direct database storage as JSONB/TEXT
- **FromStr Implementations for Enums** — Added `FromStr` for `SiteType` and `CropType` with snake_case parsing for flat-string serialization compatibility
- **LPIS Data Public Re-export** — `LpisParcel` now publicly re-exported as `LpisData` from domain crate

### Changed
- **API DTOs Use Domain Types Directly** — `CreateSiteDto`, `UpdateSiteDto`, `SiteDto` now use `SiteType`, `CropType`, `RowConfig`, `SigpacData`, `Boundary`, `GeoPoint` directly from domain (eliminates conversion layer)
- **Flat String Enum Serialization** — All enums serialize as flat strings (`"field"`) instead of tagged format (`{"Field":{}}`) via `#[serde(rename_all = "snake_case")]`
- **Complex JSONB Fields Use `serde_json::Value`** — `SigpacData`, `RowConfig`, `LpisData`, `plots`, `properties` use flexible JSON Value instead of strict structs
- **Import Service Rewrite** — Complete rewrite of GeoJSON/Shapefile import with correct geozero 0.15.1 API:
  - Shapefile: `read_records()` for DBF properties + `iter_geometries()` with `GeoWriter` for geometries
  - GeoJSON: Proper `Option<Vec>` handling, `GeoJsonGeometry` struct usage
- **JWT Generation Fixed** — `generate_jwt` now accepts `(user_id, tenant_id, roles)` 3-argument signature
- **Dependency Updates** — Pinned `geo-types = 0.7.11` (workspace) to match geozero 0.15.1 re-export; added `tokio` as optional feature to domain crate

### Fixed
- **Circular Dependency Resolution** — Broke `site.rs` ↔ `spatial.rs` cycle via `spatial/types.rs` extraction
- **SQLX_OFFLINE Query Metadata** — All 3 import service queries now cached via `cargo sqlx prepare` with live PostGIS database
- **Site Repository Visibility** — `PgSiteRepo` now public (macro generates public struct)
- **UserRole Import Paths** — Fixed all `UserRole` imports to correct `agrocore_domain::entities::user::UserRole` path
- **Boundary Deserialization** — Fixed deserialization from JSONB in site repository
- **Shapefile Import** — Fixed geozero 0.15.1 API usage (`iter_features()` returns `ProcessorSink` iterator; correct approach uses `read_records()` + `iter_geometries()`)
- **GeoJSON Import** — Fixed `Option<Vec>` unwrapping, `GeoJsonGeometry` struct handling, error type consistency (`SharedError`)
- **JWT Argument Count** — Fixed `generate_jwt` calls in auth handlers to pass 3 arguments
- **Test Updates** — Updated validation tests (`validation_tests.rs`, `dto_validation_tests.rs`, `spatial_tests.rs`, `site_tests.rs`) to match new DTO structure

### Quality Gates
- `cargo fmt --check` ✅
- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (187+ tests passing)
- `cargo clippy --workspace` ✅ (warnings only, no errors)

### Dependencies
- `geo-types = 0.7.11` (workspace, matches geozero 0.15.1)
- `tokio` optional feature in domain crate for async traits

## [0.20.0] - 2026-09-12

### Fixed
- **Admin UI Docker Build**: Fixed WASM build in Dockerfile — use `cargo build --target wasm32-unknown-unknown --lib` instead of `wasm-pack` (target `no-bundler` not available in wasm-pack 0.13.1)
- **CI/CD Pipeline**: Updated `build-wasm` job to use `cargo build` directly
- **Workspace Target Dir**: Copy WASM from `/app/target/` (workspace-level) to crate-level target dir
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.19.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 3)**: Updated 30+ dependencies to latest patch/minor versions
  - actix-web 4.15.0, actix-cors 0.7.2, reqwest 0.13.5
  - serde 1.0.229, serde_json 1.0.151
  - validator 0.21.0, thiserror 2.0.20, anyhow 1.0.104
  - tracing 0.1.44, tracing-subscriber 0.3.23, tracing-actix-web 0.7.22
  - utoipa 5.5.0, utoipa-swagger-ui 9.0.2
  - actix-web-prometheus 0.1.2, prometheus 0.14.0
  - chrono 0.4.45, uuid 1.26.1, config 0.15.25, dotenvy 0.15.7, log 0.4.34
  - geo 0.33.1, geojson 1.0.0, rust_xlsxwriter 0.99.0
  - actix-files 0.7.0, actix-governor 0.10.0, futures 0.3.34
  - async-nats 0.50.0, mockall 0.15.0
  - sqlx 0.8.6 (kept for geozero compatibility), geozero 0.15.1
  - strum 0.28.0, quick-xml 0.42.0, urlencoding 2.1.3
  - moka 0.12.16, bytes 1.12.1, rumqttc 0.25.1
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.18.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 2 continued)**: Updated `redis` to 1.7.0 — complete async API rewrite
- **API Migration**: Updated `TokenRevocationList` in `api/middleware.rs` to use `get_multiplexed_async_connection()` and `AsyncCommands` trait
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.17.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 2)**: Updated `jsonwebtoken` to 11.0.0 with `rust_crypto` feature — crypto backend trait API migration
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.16.0] - 2026-09-12

### Fixed
- **Dependency Updates (Phase 1)**: Updated `argon2` to 0.6.0, `password-hash` to 0.6, `rand` to 0.10.2 — resolves CI build failure with `SaltString`/`thread_rng` imports
- **API Migration**: Updated password hashing in `PgUserRepo::create()`, `system.rs::initial_setup()`, `demo.rs::seed_demo()` to use `password-hash` 0.6 API (`phc::SaltString::generate()`, `hash_password()` without explicit salt)
- **Quality Gates**: All 15 crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

## [0.15.0] - 2026-09-10

### Added
- **agrocore-backup CLI** — Full command-line interface for backup operations:
  - `agrocore-backup run` — Daemon mode (scheduler + MQTT bridge)  
  - `agrocore-backup backup <database|config|full>` — Manual backup execution
  - `agrocore-backup restore <id> [--target-db]` — Restore from backup
  - `agrocore-backup list [--type] [--limit]` — List backups with filtering
  - `agrocore-backup verify <id>` — Verify backup integrity
  - `agrocore-backup status <job-id>` — Show backup job status

- **Complete Backup Verification Pipeline** (`VerificationManager`):
  - Test database creation/dropping via PostgreSQL
  - Automated restore to test database using `pg_restore`
  - SHA256 checksum verification for all manifest objects (target objects + file checksums)
  - Row count verification across all tables
  - Schema comparison (columns, types) between source and restored database
  - Full integration with `StorageBackendTrait::download_bytes()`

- **Restore Implementation** in `PgDump`:
  - `restore_from_storage()` — Downloads dump and pipes to `pg_restore --clean --if-exists`
  - Supports target database parameter

- **StorageBackendTrait::download_bytes()** implemented for all backends:
  - S3/MinIO/B2/Wasabi, Azure, GCS, Local filesystem

- **Quality Gates**: All 15 Crates pass `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`

### Changed
- **Version bump**: 0.14.0 → 0.15.0 (Minor bump for backup CLI + verification features)
- **agrocore-backup**: Main function refactored to use `BackupService::start_scheduler()` instead of inline scheduler
- **BackupService::verify_backup()** now receives `storage` and `targets` for full verification

### Fixed
- **sha2** dependency added to backup-service for checksum verification
- **Unused imports** removed across all crates (logging, messaging, lpis-providers, scheduler, api, domain)
- **Dead code warnings** resolved by implementing verification/restore/CLI features

## [0.14.0] - 2026-09-08

### Added
- **Phase 7: Migration auf externe Services (P0 - KRITISCH)** — Vollständige Migration aller 15 Crates auf zentrale Services
  - **Scheduler-Migration**: 4 Timer von `tokio::spawn` / `tokio-cron-scheduler` → `agrocore-scheduler` Crate
    - `bridge_stats_reporter` (60s) — MQTT Bridge Statistiken
    - `weather_update` (Cron `0 */30 * * * *`) — Wetterdaten-Updates
    - `db_pool_health` (5s) — Datenbank-Pool-Health-Check
    - `db_monthly_cleanup` (Cron `0 0 1 * *`) — Monatliche Bereinigung + Abschreibung
  - Alle Jobs extern in `backup-service/main.rs` registriert (keine zyklischen Dependencies)
  - Bridge (nicht `Send`) läuft auf Main Thread, Scheduler Jobs (`Send`) in Background Tasks

### Changed
- **Messaging-Migration**: Alle Crates von direkter `async_nats` Nutzung → `agrocore_messaging::Publisher/Subscriber` Traits
  - Neue Traits: `Publisher`, `Subscriber`, `MessageStream` in `agrocore-messaging`
  - `MessagingClient` implementiert beide Traits
  - NATS Subject-Konstanten öffentlich exportiert für konsistente Nutzung

- **Logging-Migration (Rest)**: 5 Crates auf `agrocore_logging` migriert
  - `agrocore-domain` (`depreciation.rs`)
  - `agrocore-geometry-service` (`main.rs`)
  - `agrocore-asset-registry` (`main.rs`)
  - `agrocore-reporting-service` (`main.rs`)
  - `agrocore-lpis-providers` (nutzte bereits `agrocore_logging`)

- **WASM Migration**: `agrocore-admin-ui` 
  - `tracing` Dependency entfernt
  - `agrocore-logging` mit `dev-console` Feature hinzugefügt (Browser Console Logging)

- **Version bump**: 0.13.0 → 0.14.0 (Minor bump für Phase 7 Migration)

- **Quality Gates**: Alle 15 Crates kompilieren (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`)
  - 153+ Tests grün
  - Clippy sauber (nur unused-import warnings)
  - WASM Target `wasm32-unknown-unknown` kompiliert fehlerfrei

## [0.13.0] - 2026-09-03

### Added
- **agrocore-messaging crate**: Added request-reply pattern support
  - `MessagingClient::request()` - native async-nats 0.50 request-reply pattern
  - `MessagingClient::request_with_headers()` - request-reply with custom headers
  - `MessagingClient::new_mock()` behind "mocks" feature for testing
- **agrocore-backup crate**: Fixed name conflict with `error` module
  - Renamed imported `error` from agrocore_logging to avoid conflict with local `mod error;`
  - Use `tracing::error` macro instead
  - Added `tracing` and `tracing-subscriber` dependencies
- **IoT Device Registry**: Fixed Home Assistant discovery config generation
  - Fixed type mismatch in `generate_ha_discovery_configs()` call
  - Added proper measurements generation from device capabilities
- **IoTCapabilityType enum**: Added missing variants (GPS, Power, Energy, Pressure, Voltage, Current)

### Changed
- **Version bump**: 0.12.0 → 0.13.0 (Minor bump für neue Messaging-Features und IoT-Fixes)
- **Quality Gates**: Alle 15 Crates kompilieren (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`)

## [0.12.0] - 2026-08-31

### Added
- **agrocore-logging crate (NEW)**: Einheitlicher strukturierter Logging-Service für alle 15 Crates
  - `ServiceContext` / `RequestContext` für automatische Span-Anreicherung (service_name, environment, version, instance_id, request_id, tenant_id)
  - `SpanExt` Trait für strukturierte Felder: `record_error()`, `record_latency()`, `record_db_query()`, `record_http_status()`, `record_tenant()`, `record_user()`
  - Console Layer (pretty output mit Thread-IDs/Names) + OTLP Layer (OpenTelemetry distributed tracing)
  - Macros: `agrocore_span!`, `agrocore_info!`, `agrocore_error!`, `agrocore_warn!`, `agrocore_debug!`
  - Konfiguration via `LoggingConfig` (Env-File + Env-Vars `AGROCORE_LOG__*`)
  - Feature-gated: `dev-console` (default, pretty console), `otlp` (OpenTelemetry)

### Changed
- **Version bump**: 0.11.0 → 0.12.0 (Minor bump für neues Logging-Crate)
- **Quality Gates**: Alle 15 Crates kompilieren, Tests grün (153+), Clippy sauber (nur unused-import warnings)

## [0.11.0] - 2026-08-31

### Added
- **agrocore-scheduler crate (NEW)**: Wiederverwendbarer Scheduler-Service für wiederkehrende Worker-Aufgaben UND einmalige Termine
  - `JobType::OneTime { execute_at: DateTime<Utc> }` für präzise Terminplanung zu exakten Zeitpunkten
  - Cron-basierte wiederkehrende Jobs (wie zuvor) für Worker-Tasks (Backups, Cleanup, Sync, etc.)
  - `SchedulerService` mit NATS Event-Publishing (job.started, job.completed, job.failed)
  - Retry-Policies mit konfigurierbaren Delays und max_retries
  - Timezone-Support für cron-Ausdrücke
  - Handler-Registry für Builtin/Command/HTTP/NATS Jobs
- **Backup-Service**: Refactored auf externen `agrocore-scheduler` Crate
  - Entfernt direkte `tokio-cron-scheduler` Abhängigkeit
  - Nutzt jetzt `SchedulerService` mit `JobDefinition`, `JobType::Builtin`
  - Registrierter Handler "backup_database" für DB- und Config-Backups
  - NATS-Events für Backup-Start/Progress/Completed/Failed

### Changed
- **Version bump**: 0.10.0 → 0.11.0 (Minor bump für neue Scheduler-Features)
- **agrocore-scheduler**: Re-exports für `JobDefinition`, `JobType`, `SchedulerConfig`, `SchedulerService`, `SchedulerError`
- **Quality Gates**: Alle 14 Crates kompilieren, Tests grün (153+), Clippy sauber (nur unused-import warnings)

### Fixed
- **Scheduler**: OneTime Jobs nutzen `tokio::time::sleep` für exakte Ausführungszeit
- **Scheduler**: `add_job()` validiert OneTime Jobs ohne Cron-Parsing
- **Backup-Service**: Clone-Impl für `BackupService` includes `scheduler` field

## [0.10.0] - 2026-08-28

### Added
- **6 neue Domain-Entitäten** (vollständig implementiert, keine Stubs):
  - `Building` mit `BuildingType` Enum, CRUD DTOs, PostgreSQL Repo + API Handler
  - `Group` mit `GroupType` Enum, hierarchische Struktur (`parent_group_id`), CRUD + Children + by_plot
  - `Tree` mit `TreeType` Enum, `group_id` Referenz, CRUD + by_plot + by_group
  - `Livestock` (herden-basiert) mit `LivestockType` Enum, `herd_id`, `count`, CRUD + by_plot + by_herd
  - `Variety` mit `VarietyCategory` Enum, CRUD + by_category
  - `Breed` mit `Species` Enum, CRUD + by_species
- **Repository Traits** in `domain/src/repositories.rs`: 6 neue Traits mit vollständigen CRUD + spezialisierten Find-Methoden
- **PostgreSQL Implementierungen** (6 neue Dateien in `infrastructure/src/postgres/`): alle nutzen `pg_repo!` Macro mit `PaginatedResponse`
- **Infrastructure Wiring** (`database.rs`): alle 6 Repos in `PostgresDb` struct, `connect()`/`from_pool()`, Accessor-Methoden, `Database` Enum Delegation, `MockDatabase` Felder
- **API Layer**: DTOs + Handler für alle 6 Entitäten (`building.rs`, `group.rs`, `tree.rs`, `livestock_new.rs`, `variety.rs`, `breed.rs`), registriert in `handlers/mod.rs`
- **Admin UI i18n**: Alle neuen Navigation-Schlüssel (`nav_groups`, `nav_trees`, `nav_buildings`, `nav_plot_entities`, `nav_livestock`) und Entity-Schlüssel (`livestock_goat`, `livestock_chicken`, `livestock_sheep`, `livestock_cattle`, `tree_cork_oak`, `group_building`, `group_coop`) vollständig für alle 10 Sprachen (de, en, es, fr, pt, it, pl, ro, uk, nl)
- **Pre-existing Fixes**: `TreatmentRecord` mit `sqlx::FromRow`, `find_treatments_by_animal` Methode, `reporting-service` Fetch-Trennung, `livestock.rs` DTO Type-Mismatches behoben
- **Version bump**: 0.9.26 → 0.10.0 (Major bump für 6 neue Domain-Entitäten)

### Changed
- Alle Quality Gates (`cargo fmt`, `cargo check`, `cargo test`, `cargo clippy`) laufen fehlerfrei durch (nur unused-import warnings)

## [0.9.26] - 2026-08-28

### Added
- Neue Domain-Entitäten: Group, Livestock, Tree, Building, Variety, Breed
- Migrationen 001-008, CSV-Kataloge, Import-Script, AdminUI-Module, Navigation, i18n, API-Endpunkte, DB-Repos (vollständig, keine Stubs)

## [0.9.25] - 2026-08-27

### Added
- Migration: `trigger_updated_at()` Funktion (für alle `updated_at`-Trigger)

### Changed
- Version bump: 0.9.24 → 0.9.25

## [0.9.24] - 2026-08-27

### Fixed
- `domain/src/repositories.rs`: `find_all_filtered` + `record_fuel_consumption` + `record_usage` Lifetime (`'b`) behoben; `#[allow(clippy::too_many_arguments)]` korrekt gesetzt; `Cargo.toml` `depreciation` Feature hinzugefügt
- Alle `cargo c` Fehler behoben (`geometry` Timeout eingebaut; `weather` Timeout + tracing import; `reporting` Paginierung 500 + Timeout 30s; `lpis-providers` `Box::pin` fix; `api/middleware` doppelte Imports entfernt)

### Added
- Abschreibung: Timer (`database.rs`), Modul `depreciation.rs`, Domain-Feature `depreciation`
- Equipment-Suche: Filter (`find_all_filtered` + 2 Felder), API-DTO + Handler

### Changed
- docs/tasks.md: Abschreibung + Equipment-Suche als `[x]`
- docs/optimizations.md: Status aktualisiert
- Version bump: 0.9.23 → 0.9.24

## [0.9.23] - 2026-08-27

### Added
- Domain feature `depreciation`: `Cargo.toml` feature + `lib.rs` `#[cfg]` + Modul `depreciation.rs`
- Abschreibung: Timer + Modul vollständig; Finanzbericht-Integration als nächster Schritt
- Equipment-Filter: `find_all_filtered` erweitert (`fuel_efficiency_range`, `location_filter`); `EquipmentFilterDto` aktualisiert; Handler integriert.

### Fixed
- `domain/src/repositories.rs`: `find_all_filtered` Lifetime-Fehler (`'b'`) + `#[allow(clippy::too_many_arguments)]` behoben
- `Cargo.toml`: `[build]` entfernt (nach `.cargo/config.toml` verschoben) — `unused manifest key` behoben
- `admin-ui`: Leptos konsolidiert `0.8.6`
- `lpis-providers`: `with_retry` `Box::pin` fix
- `geometry-service`: Timeout-Struktur + worker timeout
- `weather-service`: Timeout + `tracing` import
- `api/middleware`: doppelte Imports entfernt
- `reporting-service`: `info` Import bereinigt

### Changed
- docs/tasks.md: Abschreibung als `[x]` markiert; Equipment-Suche als `[x]`
- docs/optimizations.md: nur noch P1/P3/P4 Haupt-Tasks offen
- Version bump: 0.9.22 → 0.9.23

## [0.9.22] - 2026-08-27

### Added
- Equipment-Filter: `find_all_filtered` erweitert (`fuel_efficiency_range`, `location_filter`); `EquipmentFilterDto` aktualisiert; Handler integriert.
- Abschreibung: monatlicher Timer (`tokio::spawn` in `database.rs`); Modul `depreciation.rs` (`calculate_straight_line`, `double_declining`, `schedule`)

### Changed
- docs/tasks.md: Equipment-Suche als erledigt markiert
- Version bump: 0.9.21 → 0.9.22

## [0.9.21] - 2026-08-27

### Fixed
- admin-ui (Leptos): konsolidiert auf 0.8.6 (von 0.9.0-beta) — 86 Fehler behoben
- lpis-providers: with_retry Box::pin fix
- geometry-service: Timeout-Struktur eingebaut; worker timeout aktiv
- api/middleware: doppelte Imports entfernt
- weather-service: Timeout-Struktur + tracing import
- reporting-service: Paginierung 500, Timeout 30s, Tracing-Log

### Added
- OPT-009: Reporting-Service Paginierung 500 + Timeout 30s + Tracing
- OPT-010: Weather-Service Timeout + Geometry-Service Timeout

### Changed
- Version bump: 0.9.20 → 0.9.21

## [0.9.20] - 2026-08-27

### Added
- OPT-009: Reporting-Service Paginierung 500 (von 100), Timeout 30s aktiv, Tracing-Log bei Timeout
- OPT-010: Weather-Service Timeout (30s, tokio-timeout) + Geometry-Service Timeout (30s)
- OPT-008: Domain Mock lazy-loading (OnceLock) eingebaut
- Makro prüft `AGROCORE_METRICS_ENABLED` vor Messung

### Changed
- docs/optimizations.md bereinigt — alle offenen Tasks (009, 010) als erledigt; nur 010 als abgeschlossen dokumentiert
- Version bump: 0.9.20 → 0.9.21

## [0.9.17] - 2026-08-27

### Added
- OPT-005: LPIS Cache nutzt Arc<[u8]>, kein doppelter Klon; retry via with_retry aktiv
- OPT-007: db_exec Makro verbessert (Referenz-Klon statt direkter Referenz)

### Changed
- Version bump: 0.9.16 → 0.9.17

## [0.9.16] - 2026-08-27

### Added
- OPT-004 Metrics Makro `measure_sqlx_query!` vollständig integriert (nur aktiv wenn `is_enabled()`); Middleware `MetricsMiddleware` als actix-web Middleware eingebunden
- OPT-006 Messaging: Webhook-Event-Handler `handle_webhook_event()` mit exponentiellem Retry-Backoff (max 3 Versuche) ergänzt; retry für NATS `publish` und `publish_raw` aktiv (`with_retry` aus shared, 3 Versuche, exponentiell)
- Makro prüft `AGROCORE_METRICS_ENABLED` vor Messung

### Changed
- Version bump: 0.9.14 → 0.9.16

## [0.9.13] - 2026-08-27

### Fixed
- OPT-001 Dashboard TUI: process.rs Timeout (30s) und explizite Fehlerbehandlung eingebaut
- Monitoring Toggle (`AGROCORE_METRICS_ENABLED`) aktiv
- OPT-005 LPIS-Providers Cache und retry: Arc<[u8]> nutzt, kein doppelter Klon; retry via with_retry aktiv. Status: erledigt.

### Changed
- Version bump: 0.9.16 → 0.9.17

- OPT-005 LPIS-Providers: Cache nutzt Arc<[u8]>, kein doppelter Klon; retry aktiv

## [0.9.12] - 2026-08-27

### Added
- Dashboard TUI v0.9.12: Service control overlay + sparkline graphs + full tab navigation completed
- Process manager for docker compose services
- Monitoring (`DbMetrics`/`BusinessMetrics`) per `AGROCORE_METRICS_ENABLED` umschaltbar
- Performance-Build: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip`, `force-frame-pointers=yes`

### Changed
- Version bump: 0.9.11 → 0.9.12

## [0.9.11] - 2026-08-24

### Added
- **Equipment Depreciation (Backend + Admin UI)**
  - Migration `20260824_004_add_equipment_depreciation.sql`: creates `equipment_depreciation` table with `equipment_id`, `tenant_id`, `purchase_price`, `salvage_value`, `accumulated_depreciation`, `depreciation_method`, `useful_life_years`, `net_book_value`, and `created_at` columns
  - `EquipmentDepreciationDto` and `DepreciationScheduleEntry` domain entities in `crates/domain/src/entities/equipment.rs`
  - `DepreciationMethod` enum: `StraightLine` and `DoubleDeclining`
  - `EquipmentRepository::get_depreciation()` — calculates net book value and accumulated depreciation
  - `EquipmentRepository::get_depreciation_schedule()` — generates year-by-year depreciation schedule
  - `GET /api/v1/equipments/{id}/depreciation` endpoint — returns depreciation summary
  - `GET /api/v1/equipments/{id}/depreciation-schedule` endpoint — returns year-by-year schedule
  - Admin UI: Depreciation section in `EquipmentDetailPage` with summary card (cost basis, salvage, accumulated, net book value) and schedule table (year, amount, accumulated, net book value)
  - Admin UI: `fetch_depreciation()` and `fetch_depreciation_schedule()` API client functions
  - New i18n keys: `depreciation`, `depreciation_loading`, `cost_basis`, `salvage_value`, `accumulated_depreciation`, `net_book_value`, `annual_depreciation`, `useful_life_years`, `depreciation_year`, `no_depreciation_records`

- **Dashboard Crate (Ratatui TUI)**
  - New `crates/dashboard/` crate: live TUI dashboard using ratatui 0.29 + crossterm 0.29
  - Four-panel layout: Services (PostgreSQL, NATS, MQTT port checks), Build (cargo check status), Git (branch + dirty state), System (CPU/memory/disk via /proc)
  - `ProcessManager` for managing dev services
  - Scripts: `scripts/dev.sh` and `scripts/server.sh`

- **Dashboard Updates**
  - Added sparkline graphs (Unicode block chars: ▁▂▃▄▅▆▇█) for CPU and memory history in System panel
  - Added Service Control menu (press 's'): restart PostgreSQL (r), NATS (n), MQTT (m), Redis (i), restart all (a), stop all (x)
  - Added 3 more service checks: Redis (6379), API (8080), Admin UI (80) to Services panel
  - Color-coded service status: green (up), red (down)
  - Fixed `restart_service` thread escape: `service: &str` cloned to `String` for `move` closure
  - Added `docker-compose.dev.yml` with all 6 services (postgres, nats, mqtt, redis, api, admin-ui)
  - Updated `dev.sh` to start all services + wait 5s for boot
  - Updated `server.sh` to use dev compose and include Redis/MQTT
  - Dockerfile.api: rust 1.82 → 1.85 for edition2024 support

### Fixed
- `returning the result of a let binding` in `crates/infrastructure/src/postgres/equipment.rs` `get_depreciation_schedule` — replaced `let dep = ...; dep` with inline expression
- Depreciation schedule table field names: corrected `annual_depreciation_amount` → `depreciation_amount` to match `DepreciationScheduleEntry` DTO
- Depreciation schedule table: removed non-existent `depreciation_method` column, added `net_book_value` column

### Fixed
- Mosquitto MQTT broker: fixed `per_listener_settings` must be set before security settings, removed unsupported `sys_topic_prefix` and `auto_save_interval` variables, replaced deprecated `message_size_limit` with `max_packet_size`
- Mosquitto healthcheck: added `-h 127.0.0.1` for reliable port connectivity check
- docker-compose dev/prod: mount mosquitto config read-write (fixes `chown: Read-only file system` error during container init)

### Changed
- Version bump: 0.9.10 → 0.9.11

## [0.9.10] - 2026-08-24

### Added
- **Equipment Usage Logging (Backend + Admin UI)**
  - Migration `20260824_003_add_equipment_usage_log.sql`: creates `equipment_usage_log` table (id, equipment_id, tenant_id, worker_id, task_id, operation_type, started_at, ended_at, hours_operated, note, created_at, updated_at) with indexes on equipment_id, tenant_id, worker_id, started_at
  - `UsageLogDto` and `UsageSummaryDto` domain entities in `crates/domain/src/entities/equipment.rs`
  - `EquipmentRepository::get_usage_log()` — retrieves usage log history per equipment, newest first
  - `EquipmentRepository::get_usage_summary()` — aggregates usage stats (total_hours, total_sessions, avg_hours/session, first/last used)
  - `EquipmentRepository::record_usage()` — inserts usage log entry with auto-calculated hours_operated from start/end time
  - `GET /api/v1/equipments/{id}/usage` endpoint — returns usage log history
  - `POST /api/v1/equipments/{id}/usage` endpoint — records a usage log entry
  - `GET /api/v1/equipments/{id}/usage-summary` endpoint — returns aggregated usage summary
  - API DTOs: `UsageLogDto`, `UsageSummaryDto`, `CreateUsageLogRequest` in `crates/api/src/dto/equipment.rs`
  - Admin UI: `fetch_usage_log()`, `fetch_usage_summary()`, `record_usage()` API client functions + DTOs in `api.rs`
  - Admin UI: Usage Logging card in `EquipmentDetailPage` with summary grid, history table, and record form
  - New i18n keys: `usage_logging`, `usage_worker`, `usage_task`, `usage_operation`, `usage_started_at`, `usage_ended_at`, `usage_hours_operated`, `usage_recorded_at`, `usage_total_hours`, `usage_total_sessions`, `usage_avg_hours`, `usage_first_used`, `usage_last_used`, `no_usage_records`, `usage_summary_loading`, `record_usage`

### Fixed
- `duplicate import DateTime/Utc` in `crates/domain/src/repositories.rs` — removed redundant `use chrono::{DateTime, Utc}` (already imported at line 744)
- `E0308/E0369` in `crates/infrastructure/src/postgres/equipment.rs` `record_usage` — fixed `ended_at: Option<DateTime<Utc>>` by adding `.unwrap_or(started)` for arithmetic comparison

### Changed
- Version bump: 0.9.9 → 0.9.10

## [0.9.9] - 2026-08-24

### Added
- **Equipment Fuel Consumption Tracking (Backend + Admin UI)**
  - Migration `20260824_002_add_equipment_fuel_consumption.sql`: adds `fuel_capacity_liters`, `fuel_type` columns to `equipment` table + new `equipment_fuel_consumption` table (id, equipment_id, tenant_id, liters, cost_per_liter, total_cost, operation_type, field_id, hours_operated, consumed_at, notes, created_at) with indexes
  - `FuelConsumptionDto` domain entity in `crates/domain/src/entities/equipment.rs`
  - `EquipmentRepository::get_fuel_consumption()` — retrieves fuel consumption history per equipment, newest first
  - `EquipmentRepository::record_fuel_consumption()` — inserts fuel consumption entry with auto-calculated total_cost
  - `Equipment` entity extended with `fuel_capacity_liters` and `fuel_type` fields
  - `GET /api/v1/equipments/{id}/fuel-consumption` endpoint — returns fuel consumption history
  - `POST /api/v1/equipments/{id}/fuel-consumption` endpoint — records a fuel consumption entry
  - API DTOs: `FuelConsumptionDto` + `CreateFuelConsumptionRequest` in `crates/api/src/dto/equipment.rs`
  - Admin UI: `fetch_fuel_consumption()` + `record_fuel_consumption()` API client functions in `api.rs`
  - Admin UI: `EquipmentDto` extended with `fuel_capacity_liters` + `fuel_type` fields
- **i18n v2.0 Architecture Improvements**
  - Fixed `needless-borrow` clippy error in `build.rs` (`to_screaming_snake(raw_key)` — removed unnecessary `&`)
  - Fixed `unreachable_patterns` in generated `tr()` method (`#[allow(unreachable_patterns)]` added to match the fallback `_ => default_text()` arm)
  - Replaced manual `impl Default for Locale` with `#[derive(Default)]` + `#[default]` attribute on `Locale::De` (fixes `derivable_impls` clippy error)
  - Fixed build.rs template escaping for `impl Translatable` block (`{{` for literal braces in `format!` template)

### Fixed
- Removed deprecated `crates/i18n-codegen` crate (build logic consolidated in `crates/i18n-shared/build.rs`)

### Changed
- Version bump: 0.9.8 → 0.9.9

## [0.9.8] - 2026-08-24

### Added
- **i18n v2.0: Shared Translation Architecture (Backend + Frontend)**
  - `crates/i18n-shared`: Shared `Locale` enum (10 locales: De, En, Es, Fr, Pt, It, Pl, Ro, Uk, Nl) with `FromStr`, `Display`, `Default` impls — usable by both backend and frontend
  - `crates/i18n-codegen/build.rs`: Build-time code generation that reads `crates/admin-ui/locales/app.yml` and generates `Msg` enum (450 variants) with `Locale → &'static str` match-arms in `OUT_DIR/translation.rs`
  - `Translatable` trait with `.tr(locale)` method for typsafe compile-time-translation lookups
  - 5 unit tests verifying `Locale`, `Msg::YES.tr(Locale::De)` → "Ja", `Msg::YES.tr(Locale::En)` → "Yes"

### Changed
- Version bump: 0.9.7 → 0.9.8

## [0.9.7] - 2026-08-24

### Added
- **Equipment Maintenance Cost Tracking (Backend + Admin UI)**
  - Migration `20260824_001_add_equipment_maintenance_costs.sql`: adds `parts_cost`, `labor_hours`, `downtime_hours` columns to `equipment_maintenance_log` table (+ indexes for performance)
  - `MaintenanceCostSummaryDto` domain entity — aggregated cost summary (total_parts_cost, total_labor_hours, total_downtime_hours, total_cost, total_maintenance_count)
  - `EquipmentRepository::get_maintenance_cost_summary()` trait method — aggregates cost data via SQL SUM/COALESCE
  - `EquipmentRepository::update_maintenance_costs()` trait method — updates parts_cost, labor_hours, downtime_hours on a specific log entry
  - `MaintenanceLogDto` extended with cost fields (parts_cost, labor_hours, downtime_hours)
  - `GET /api/v1/equipments/{id}/maintenance-cost-summary` endpoint — returns aggregated cost summary for an equipment
  - `PUT /api/v1/equipment-maintenance/{log_id}/costs` endpoint — updates maintenance cost fields on a specific log entry
  - Admin UI: Maintenance Cost Summary card in `EquipmentDetailPage` with 4-column grid (Parts Cost, Labor Hours, Downtime, Total Cost) + total maintenance count
  - Admin UI: Maintenance log table now displays Parts Cost, Labor Hours, Downtime columns alongside Hours/Note
  - `on_record` handler now refreshes cost summary alongside equipment + maintenance log
  - New i18n keys: `cost_summary`, `parts_cost`, `labor_hours`, `downtime_hours`, `total_cost`, `cost_summary_loading`, `total_maintenance_count`, `performed_at`, `none`, `yes`, `no`

### Changed
- Version bump: 0.9.6 → 0.9.7

## [0.9.6] - 2026-08-24

### Added
- **Equipment Detail View (Backend + Admin UI)**
  - `GET /api/v1/equipments/{id}/maintenance` — retrieves maintenance log history for a specific equipment
  - `EquipmentRepository::get_maintenance_log()` trait method
  - `MaintenanceLogDto` domain entity with id, equipment_id, tenant_id, hours, note, performed_at, created_at fields
  - `MaintenanceLogDto` API DTO and `fetch_equipment_maintenance_log()` frontend API client
  - `EquipmentDetailPage` component at route `/equipment/:id` — shows equipment details, maintenance history table, and record-maintenance form
  - Navigation link from EquipmentManagement to EquipmentDetailPage
  - New i18n keys: `equipment_detail`, `basic_info`, `maintenance_info`, `maintenance_history`, `no_maintenance_records`, `maintenance_recorded_success`, `invalid_equipment_id`, `equipment_not_found`, `maintenance_intervals`, `days`

### Changed
- Version bump: 0.9.5 → 0.9.6

## [0.9.5] - 2026-08-22

### Added
- **Equipment Search & Filtering (Backend + Admin UI)**
  - `EquipmentRepository::find_all_filtered()` trait method with dynamic SQL WHERE clauses for searchable, filtered equipment listing
  - `GET /api/v1/equipments/search` endpoint accepting query parameters: `search` (fulltext on label/code), `equipment_type`, `in_usage`, `needs_maintenance`, `page`, `per_page`
  - `EquipmentFilterDto` and `MaintenanceIntervalDto` in API DTO layer
  - Admin UI: `EquipmentFilter` struct + `fetch_equipment_filtered()` API client function
  - EquipmentManagement component with search bar, type dropdown, in-usage checkbox, needs-maintenance checkbox, and clear-filters button
  - New i18n keys: `search_filter`, `search`, `type`, `all_types`, `in_usage`, `needs_maintenance`, `clear_filters`, `maintenance_hours`, `next_maintenance`, `no_deadline`

- **Maintenance Planning (Backend + Admin UI)**
  - `GET /api/v1/equipments/maintenance` — lists equipment with due maintenance (`next_maintenance_date <= now`)
  - `POST /api/v1/equipments/{id}/maintenance` — records a maintenance event transactionally: inserts log entry in `equipment_maintenance_log`, updates `last_maintenance_hours`, and recalculates `next_maintenance_date` based on the shortest maintenance interval
  - `find_maintenance_due()` and `record_maintenance()` methods on `EquipmentRepository` trait
  - `MaintenanceRecordDto` with `hours` (validated `range(min=0.0)`) and `note` fields
  - `equipment_maintenance_log` database table migration (id, equipment_id, tenant_id, hours, note, performed_at)
  - Admin UI: maintenance modal with operating-hours input + note field, per-equipment maintenance button (wrench icon), next-maintenance column with badge indicators
  - New i18n keys: `maintenance_due`, `wrench`, `record_maintenance`, `hours`, `note`, `cancel`, `save`

### Fixed
- Clippy `unnecessary_unwrap` in `find_all_filtered` — replaced `.is_some()` + `.unwrap()` pattern with direct `if let Some(...)` bindings in both count and items query sections
- Clippy `manual_map` in `record_maintenance` — replaced `if let Some(...) { Some(...) } else { None }` with `.map()`
- `f64` not implementing `Ord` — replaced `Vec<f64>::min()` with `fold()` using `<=` comparison to avoid NaN issues

## [0.9.4] - 2026-08-24

### Added
- **Admin UI Full-Stack Feature Coverage System — API Contract Validation Test**

  - New `crates/api/tests/api_contract_test.rs` with 6 contract tests validating all 62 UI API paths against actual API routes at build time
  - `test_api_contract_all_ui_paths_have_routes` — verifies every UI helper path has a matching API route
  - `test_api_contract_no_orphaned_routes` — detects dead API endpoints with no UI consumer
  - `test_crud_coverage` — validates all 10 major resources (Sites, Orders, Workers, Users, Equipment, Inventory, Livestock, Finance, Compliance, Tasks) have full Create/Read/Update/Delete coverage
  - `test_ui_routes_have_pages` — ensures all sidebar navigation routes resolve to page components (20/20 checked)
  - `test_role_based_access_consistency` — validates Admin/Manager/Worker role routes are consistent between backend and frontend
  - `test_all_ui_components_exist` — verifies all 20 page component files exist on disk
  - CI integration: contract test runs as fail-fast step in quality gate phase before other tests

- **Admin UI Mock Test Framework — Runtime Error Handling Tests**
  - `crates/admin-ui/src/tests/mock_test_framework.rs` — 20 mock-based tests
  - `MockApiClient` with `with_error()` builder for simulating API failures
  - `ApiError` enum (Network/Http/JsonParse) with `is_server_error()`, `is_client_error()`, `is_network_error()`, `user_message()` methods
  - 4 pre-built scenarios: `all_500()` (total backend outage), `network_failure()` (offline mode), `auth_expired()` (401 session expiry), `json_malformed()` (schema change)
  - `test_all_api_functions_covered_by_scenarios` — ensures 11 core fetch functions are all covered by mock scenarios
  - `test_crud_operations_can_error` — validates 15 CRUD/mutation operations return proper errors
  - Tests for HTTP status classification (11 status codes: 400, 401, 403, 404, 409, 422, 500, 502, 503, 504, default)

- **Admin UI Error Boundary — Graceful Error Handling**
  - `crates/admin-ui/src/components/error_boundary.rs` — `user_friendly_error()` converts technical error strings to localized German messages, preventing stack traces from leaking to users
  - `handle_api_result()` utility for safe API result handling
  - Integrated into Dashboard component: `fetch_tasks()` error handler now logs `user_friendly_error(&err)` instead of raw error
  - 500 errors no longer leak technical details (file paths, DB internals) to end users

- **Admin UI API Helper — `fetch_task(id)`**
  - Added missing `fetch_task(id)` helper function for Task detail pages (was previously missing, causing 404s on `/api/v1/tasks/{id}`)

- **Helper Scripts**
  - `scripts/update_contract_tests.sh` — scans for new `fetch_*` functions and verifies mock scenario coverage; exits with error if new functions aren't tested
  - `scripts/generate_coverage_report.sh` — generates `docs/admin-ui-coverage-report.md` with test statistics and coverage checklist for new features

### Fixed
- **9 API path mismatches eliminated (preventing 500 errors):**
  - `/api/v1/workforce/workers` → `/api/v1/workers` (8 paths corrected — double-prefix scope was wrong)
  - `/api/v1/sigpac/parcels{query}` → `/api/v1/sigpac/parcels?{query}` (query param separator was missing `?`)
  - `/api/v1/animals` → `/api/v1/livestock/animals` (tests corrected)
  - `/api/v1/animals/{id}/treatments` → `/api/v1/livestock/animals/{id}/treatments` (tests corrected)
- **`customer_id` field missing in test Order/DTO instantiations** — 7 instances fixed across `order_tests.rs`, `workflow_tests.rs`, `validation_tests.rs`
- **`Database` enum `large_enum_variant` clippy error** — added `#[allow(clippy::large_enum_variant)]` (Postgres variant is 680+ bytes vs 8-byte Mock; performance tuning, not a bug)

## [0.9.3] - 2026-08-22

### Added
- **Admin UI — TaskDetailPage Component**
  - New `crates/admin-ui/src/components/task_detail.rs` — `TaskDetailPage` component for viewing task details with start/stop/order actions
  - Route `/tasks/:id` now renders `TaskDetailPage` instead of `OrderList`
  - Task detail shows: label, description, order type, status, planned/deadline dates
  - Action buttons: Start Task (for worker), Stop Task, Start Order, Complete Order
  - Consumes API routes: `POST /api/v1/tasks/{id}/start-for-worker`, `POST /api/v1/tasks/{id}/stop-for-worker`, `POST /api/v1/orders/{id}/start`, `POST /api/v1/orders/{id}/complete`
  - Uses `window().location().set_href()` navigation (avoiding `use_navigate()` handle ownership issues in reactive closures)
  - `TaskData` extended with order-related fields (label, order_type, status, planned_date, deadline_date)

### Fixed
- **`task_detail.rs` compilation**: `FnOnce` vs `FnMut` closure issues in Leptos 0.8 `view!` macro — resolved by using `window().location().set_href()` instead of `use_navigate()` handle, and removing unnecessary `WriteSignal::clone()` calls (WriteSignal implements Copy in Leptos 0.8)

## [0.9.2] - 2026-08-22

### Added
- **Admin UI — Customer Management Page**
  - New `crates/admin-ui/src/components/customers.rs` — `CustomersPage` component with search-by-name and search-by-number, customer detail view, and order lookup
  - New `crates/admin-ui/src/components/task_detail.rs` — Task detail view component
  - Route `/customers` registered in `main.rs` Router
  - Consumes orphaned API routes: `GET /api/v1/customers/search/{query}`, `GET /api/v1/customers/number/{number}`, `GET /api/v1/customers/{id}/orders`

### Changed
- **Admin UI WASM build toolchain**
  - Updated GitHub CI `ci-cd.yml` wasm-pack build command to use `--package admin-ui` flag for correct workspace resolution
- **Tokio wasm-compatibility fix**
  - Reduced workspace-level tokio features (removed `fs`, `io-std`, `test-util`, `rt-multi-thread`) to be wasm-safe
  - Added `io-std` feature to `api` crate (server-only) for actix-web compatibility
  - Added explicit tokio dependency in `admin-ui/Cargo.toml` with wasm-compatible features only

### Fixed
- **Admin UI compilation error (E0308)** — `if/else` branches with incompatible view types in `customers.rs` resolved via `.into_any()` pattern
- **Clippy `bool_comparison`** — replaced `== false` with negation `!` for idiomatic code
- **Unused imports** — removed redundant `icondata_lu::*` import and unused `CustomersPage` import in `main.rs`

## [0.9.1] - 2026-08-21

### Changed
- **Admin UI Stub Replacement — Full Documentation Stub Removal**
  - Replaced all 33 placeholder stubs (`"−"` and `placeholder` attributes) across 9 Admin UI components with functional, data-driven content:
    - `analytics.rs`: Profitability chart now displays real revenue/cost/net/margin from `financial_records` API; forecast reference derived from `weather_data`; site select populated from `fetch_sites()`
    - `compliance.rs`: Compliance Score computed from checklist items; Next Audit Date from audit endpoint
    - `dashboard.rs`: Active tasks count from API instead of empty placeholder
    - `finance.rs`: Balance display from financial records API
    - `livestock.rs`: Treatment and Grazing counts from animal API
    - `resources.rs`: Hours Today from time entries API
    - `weather.rs`: Temperature, humidity, wind, precipitation, and phenology observation data from weather API
    - `sigpac.rs`: All input placeholders replaced with meaningful example values
    - `setup.rs`: Input placeholders replaced with descriptive hints
  - Added helper functions `sum_revenue()` and `sum_cost()` for profitability calculations
  - Added i18n keys: `no_records`, `profit_margin_percent`, `revenue`, `cost`, `net_profit`, `element_n/p/k/mg`, `forecast_confidence_label`, `no_weather_data`
  - Added `required-features = ["mocks"]` to 11 integration test targets in `Cargo.toml` for proper test compilation

## [0.8.22] - 2026-08-20

### Added
- **Job & Arbeitskräfte-Management: Arbeitszeiterfassung (Clock-In/Clock-Out mit GPS)**
  - New `ClockEntry` entity with `ClockEntryType` (ClockIn/ClockOut), GPS coordinates (lat/lng), task_id, notes, and timestamp
  - `ClockSession` convenience struct combining clock-in + clock-out with computed duration_hours
  - `ClockEntryRepo` trait with 9 methods (find_by_id, find_all, find_by_worker, find_active_session, find_sessions, create, update, delete, total_hours_worked)
  - PostgreSQL implementation `PgClockEntryRepo` with SQL queries for all CRUD + session pairing logic
  - REST API endpoints: `/clock-entries` (list/create), `/clock-entries/{id}` (get/update/delete), `/workers/{id}/clock-entries` (worker-specific list), `/workers/{id}/clock-active` (active session), `/workers/{id}/clock-sessions` (session history), `/workers/{id}/hours-worked` (total hours)
  - Admin UI: WorkersPage component with worker list, hourly rate display, clock-in/out buttons, and route registration at `/workers`

- **Job & Arbeitskräfte-Management: Arbeitskosten-Tracking (Stundensatz pro Arbeiter)**
  - Added `hourly_rate: Option<f64>` field to `Worker` entity, `CreateWorkerDto`, and `UpdateWorkerDto`
  - Migration adds `hourly_rate NUMERIC(10,2)` column to workers table
  - Updated PostgreSQL repo INSERT/UPDATE queries to include `hourly_rate`
  - Admin UI WorkerDto includes `hourly_rate` field

- **Migration: `2026081404_workforce_clock_entries.sql`**
  - Creates `clock_entries` table with all fields
  - Adds `hourly_rate` column to existing `workers` table
  - Indexes for tenant, worker, entry_type, timestamp, and composite worker+timestamp

### Changed
- Version bump: 0.8.16 → 0.8.17

## [0.8.16] - 2026-08-14

### Added
- **Inventory Management Complete Implementation**
  - Full PostgreSQL-backed inventory module: items, locations, transactions, balances
  - Multi-location inventory management (Silo, Scheune, Werkstatt) with stock transfers
  - Lot/batch number tracking (batch_number field on all transactions)
  - Expiration date tracking with warning thresholds (LuTriangleAlert icon in UI)
  - FIFO/FEFO inventory method selection per item (FEFO = earliest expiry first)
  - Stock valuation tracking (average_unit_cost, total_value, total_cost)
  - REST API endpoints: CRUD items/locations, stock_in, stock_out, transfer, adjust, balances
  - Admin UI: inventory items table, balances, locations tab, add item form, transactions modal

### Changed
- Version bump: 0.8.15 → 0.8.16
- Updated docs/tasks.md: marked all Inventory Management tasks as complete

## [0.8.15] - 2026-08-13

### Fixed
- **Workspace Edition Configuration (Cargo.toml)**
  - Moved `edition = "2024"` into `[workspace.package]` section to fix "unused manifest key" warning
  - Fixed intermittent "async fn is not permitted in Rust 2015" errors caused by Cargo caching stale edition info
- **Inventory Management Module**
  - Fixed icon imports in admin-ui: `LuAlertTriangle` → `LuTriangleAlert` (correct icondata_lu name), added `LuBox`
  - Fixed `view! {}` type mismatches in if/else branches by using `.into_any()` pattern
  - Added `#[derive(Default)]` to `PaginatedInventoryResponse<T>` and `InventoryItemDto`
  - Fixed `t!` macro usage in `format!()` calls (added `()` to call the closure)
  - Replaced `view! {}.into_any()` with `().into_any()` to fix clippy `unit_arg` warnings
  - Removed `utoipa::ToSchema` from `UnitOfMeasure` and `InventoryCategory` enums (utoipa doesn't support enums with internal data)
  - Added custom serde serialization and `#[schema(value_type = String)]` annotations for enum fields in OpenAPI schemas
  - Added `Display` and `Default` impls for `InventoryCategory` and `UnitOfMeasure` enums
- **Infrastructure Layer**
  - Fixed repository imports in PostgreSQL implementations (`crate::entities` → `agrocore_domain::entities`)
  - Removed lifetime issue in `find_below_minimum` caused by unused `self.clone()` reference
  - Fixed `&None::<f64>()` → `None::<f64>` and `&None::<String>()` → `None::<String>` (removed redundant references)
  - Added `#[allow(clippy::too_many_arguments)]` to `stock_in` method (10 args required for domain model)
- **API Handlers**
  - Removed unused imports (`UpdateInventoryLocationRequest`, `TransactionType`, `UpdateInventoryLocationDto`)
  - Fixed redundant closures: `.map_err(|e| SharedError::Validation(e))` → `.map_err(SharedError::Validation)`
  - Removed unused imports in test module
- **Shared Crate**
  - Fixed needless_borrow in `jwt_secret()` function (removed unneeded `&`)

## [0.8.14] - 2026-08-12

### Fixed
- **Docker Healthcheck Fix (Task 3.2a)**
  - Added `curl` to `debian:bookworm-slim` runtime images in both `Dockerfile.api` and `Dockerfile.service` — the healthcheck was failing because `curl` was not installed
  - Added `--no-install-recommends` to `apt-get install` in runtime stages for smaller image size
  - Fixed redundant `RUN mkdir -p /app/config` after `COPY config/` in `Dockerfile.api`

### Changed
- **Multi-stage Build Optimization (Task 3.2b)**
  - Documented existing caching strategy in `Dockerfile.api` (dummy source files layer for dependency-only caching)
  - Added `curl` dependency to runtime stages for healthcheck support

## [0.8.13] - 2026-08-12

### Changed
- **Repository Boilerplate Macros Applied to All Repos (Task 3.1b follow-up)**
  - Applied `pg_repo!` macro to all 30 PostgreSQL repository structs, replacing 5+ lines of boilerplate per repo with a single macro invocation
  - Total reduction: ~300 lines of boilerplate eliminated across `crates/infrastructure/src/postgres/`

### Added
- **Configuration Management (Task 3.1d)**
  - New `AgroCoreConfig` struct in `crates/shared/src/config.rs` centralizing all configuration from environment variables
  - Fields: `jwt_secret`, `redis_url`, `token_blacklist_ttl_secs`, `database_url`, `database_max_connections/min_connections`, `database_idle_timeout/max_lifetime/acquire_timeout/connect_timeout_secs`, `nats_url`, `mqtt_broker`, `rust_log`
  - `from_env()` loads from environment with sensible defaults; `global()` provides thread-safe singleton access via `OnceLock`
  - `init_global()` for explicit initialization (used in `main.rs`)
  - Added `token_blacklist_ttl_secs()` convenience function
  - All existing config functions (`jwt_secret()`, `pg_pool_options()`, `connect_timeout()`, `validate_jwt_secret()`) now delegate to `AgroCoreConfig::global()`
- **Retry Logic Unification (Task 3.1a)**
  - New generic `with_retry()` function in `crates/shared/src/lib.rs` with exponential backoff
  - Parameters: `operation_name`, `max_retries`, `base_delay_secs`, async closure
  - Replaced 4 duplicated retry loops: DB connect (database.rs), NATS connect (messaging/src/lib.rs), NATS publish + publish_raw (messaging/src/lib.rs)
  - All use exponential backoff: `base_delay * 2^(attempt-1)` seconds
- **Repository Boilerplate Macros (Task 3.1b)**
  - `pg_repo!` macro generates PostgreSQL repository struct + constructor boilerplate
  - `db_exec!` macro wraps pool cloning + `Box::pin(async move { ... })` pattern
  - Applied to `PgSiteRepo` and `PgTenantRepo` as examples

### Changed
- `init_token_revocation()` in `lib.rs` now uses centralized `AgroCoreConfig` instead of direct `std::env::var`
- `PostgresDb::connect()` uses `AgroCoreConfig::global()` for pool options instead of standalone functions
- `crates/messaging/Cargo.toml`: added `agrocore-shared` dependency for `with_retry` access

## [0.8.11] - 2026-08-12

### Added
- **Token Revocation System (Task 2.3a)**
  - `TokenRevocationList` struct with dual backend: Redis (when `REDIS_URL` is set) or in-memory `DashMap` with TTL
  - Added `jti` (JWT ID) claim to JWT tokens for unique identification
  - New `POST /api/v1/auth/logout` endpoint that revokes the current JWT token by adding its `jti` to the revocation list with matching TTL, and clears the server-side refresh token
  - Added `token_revocation: Arc<TokenRevocationList>` to `AppState`
  - In-memory fallback uses `RevocationMemoryStore` with `DashMap` for thread-safe revocation checks
  - Redis backend uses `SETEX` for atomic set-with-expiry, preventing stale entries
- **Differentiated Rate Limiting (Task 2.3b)**
  - Auth endpoints (`/auth/login`, `/auth/refresh`, `/auth/logout`) now have stricter rate limit: 10 requests per 60 seconds per IP
  - Other endpoints retain the default: 120 requests per minute per IP
  - Implemented via scoped `Governor` middleware wrapper on auth routes in `handlers::configure`

### Changed
- `Claims` struct in both `jwt.rs` and `middleware.rs`: added `jti: String` field
- `AuthenticatedUser` struct: added `jti: String` field
- `generate_jwt`: generates `jti` as UUID v4
- Auth endpoint tests updated with `jti` in test claim structs

## [0.8.10] - 2026-08-12

### Changed
- **Security: MQTT TLS Encryption (Task 2.2a)**
  - Enabled `use-rustls-no-provider` feature on `rumqttc` for TLS support
  - Added `tls_ca_cert`, `tls_client_cert`, `tls_client_key` fields to `MqttConfig`
  - Implemented `build_tls_config()` helper that builds a `TlsConfiguration::Simple` with CA certs
  - `MqttClient::connect()` and `attempt_reconnect()` now set `Transport::Tls(tls_config)` when `use_tls` is enabled
  - Falls back to system default TLS config with warning log on configuration errors
- **Security: Refresh Token Error Handling (Task 2.2b)**
  - `login` handler: `update_refresh_token` result now properly handled with `map_err` instead of `let _ =`
  - `refresh_token` handler: `update_refresh_token` result now properly handled with `map_err` instead of `let _ =`
  - This prevents stale/inconsistent refresh token state where a failed DB update would go unnoticed, potentially causing token mismatch between client and server

## [0.8.9] - 2026-08-12

### Changed
- **Dependency Security Upgrade (Task 2.1)**
  - Upgraded `argon2` from pre-release `0.6.0-rc.8` to stable `0.5.1`, eliminating release-candidate risk in production
  - Added `password-hash = "0.5"` as a workspace dependency
  - Added `rand` as a direct dependency to `agrocore-infrastructure` (was only available via workspace but not referenced)
  - Updated `hash_password` calls to explicitly generate and pass a `SaltString::generate(&mut rand::thread_rng())` (argon2 0.5.x requires explicit salt)
  - Updated `verify_password` to parse stored hash strings via `PasswordHash::new()` before verification (argon2 0.5.x API change)
  - All password hashing/verification in `PgUserRepo` and `initial_setup` handler adapted to the stable API

## [0.8.8] - 2026-08-12

### Added
- **Modular OpenAPI Documentation (Task 1.3b)**
  - Extracted the monolithic `#[derive(OpenApi)]` `ApiDoc` from `lib.rs` into a dedicated `openapi.rs` module
  - 13 per-module `OpenApi` structs (`AuthApiDoc`, `SitesApiDoc`, `OrdersApiDoc`, `UsersApiDoc`, `TasksApiDoc`, `WeatherApiDoc`, `FinanceApiDoc`, `ReportingApiDoc`, `LivestockApiDoc`, `SigpacApiDoc`, `SettingsApiDoc`, `IotApiDoc`, `ErrorApiDoc`)
  - Combined `ApiDoc` struct merges all modules at startup via `utoipa::OpenApi::merge()`
  - `openapi_with_security()` adds bearer-JWT security scheme at runtime
  - Improves incremental compilation: editing one module's OpenAPI spec only recompiles that module

## [0.8.7] - 2026-08-12

### Added
- **Selective DTO Validation (Task 1.3a)**
  - `IoTCommandRequestDto` now has `#[validate(length(...))]` on `command_type` and `#[validate(range(...))]` on `timeout_seconds`
  - `send_command` handler now calls `dto.validate()` before processing the command
  - Added `test_iot_command_request_dto_validation` test covering all validation scenarios

### Note
- `serde_json::Value` fields (like `payload`) cannot use `#[validate]` directly — they require `#[validate(nested)]` for struct-based validation, or are left unvalidated since arbitrary JSON payloads cannot be meaningfully validated by the `validator` crate

## [0.8.6] - 2026-08-12

### Added
- **Repository Pre-instantiation (Task 1.1c)**
  - Pre-instantiate all PostgreSQL repositories in `PostgresDb::connect()` and `from_pool()`
  - Repositories are cached as `Arc` fields in the struct, eliminating repeated `Arc::new(Repo::new(pool.clone()))` heap allocations on every method call
  - `Arc::clone` (refcount increment) replaces allocation on every repository access
  - Repository access methods (`site_repo()`, `user_repo()`, etc.) now return `self.xxx_repo.clone()` instead of `Arc::new(PgXxxRepo::new(self.pool.clone()))`

### Changed
- `PostgresDb` struct now has 33 fields: `pool` + 32 pre-instantiated repository Arcs
- `Database::tenant_repo()` now delegates to `db.tenant_repo.clone()` instead of `Arc::new(PgTenantRepo::new(db.pool.clone()))`
- Test fixtures updated to use `PostgresDb::from_pool()` instead of struct literal construction

## [0.8.5] - 2026-08-12

### Added
- **Security: CORS Configuration (Task 2.1)**
  - **Explicit CORS Whitelist**: Replaced `Cors::permissive()` with `build_cors()` function that reads `CORS_ALLOWED_ORIGINS` environment variable for a comma-separated list of allowed origins
  - Falls back to permissive mode with a warning log when `CORS_ALLOWED_ORIGINS` is not set (development convenience)
  - Production environments should set `CORS_ALLOWED_ORIGINS=https://your-domain.com` for proper origin restriction

### Changed
- Version bump: 0.8.4 → 0.8.5

## [0.8.4] - 2026-08-12

### Added
- **Database Optimization (Task 1.1)**
  - **Batch INSERTs via UNNEST**: Replaced N+1 individual INSERT statements in `PgUserRepo::update` with a single batch insert using `SELECT $1, unnest($2::uuid[])` for user_sites, reducing database round-trips from O(n) to O(1)
  - **LEFT JOIN + GROUP BY**: Replaced correlated subqueries `COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json)` with `LEFT JOIN user_sites us ON u.id = us.user_id` + `GROUP BY u.id` + `json_agg` across all user queries (find_by_id, find_by_email, find_all, authenticate, update, find_by_refresh_token), eliminating per-row subquery execution
  - **Shared SQL Constants**: Extracted the user SELECT query fragment into a `USER_SELECT_FIELDS` constant to reduce code duplication and ensure consistency across all user queries
  - **Enhanced Pool Configuration**: Added `DATABASE_ACQUIRE_TIMEOUT_SECS` (default: 30) and `DATABASE_CONNECT_TIMEOUT_SECS` (default: 10) environment variables for fine-grained connection pool timeout control, applied via URL query parameters at connection establishment time

### Changed
- Version bump: 0.8.3 → 0.8.4

## [0.8.3] - 2026-08-11

### Fixed
- **CI/CD Pipeline**: Updated GitHub Actions and GitLab CI to run tests with `--features=mocks` so integration tests compile and pass
- Both pipelines now explicitly enable the `mocks` feature for `cargo test --workspace --features=mocks`

### Changed
- Version bump: 0.8.2 → 0.8.3

## [0.8.2] - 2026-08-11

### Added
- **Performance Optimizations (Task 4 Quick Wins)**
  - **DecodingKey Caching**: Cached JWT DecodingKey in AuthExtractor middleware using OnceLock for improved auth performance
  - **PgPoolOptions Configuration**: Configurable database connection pool via environment variables (DATABASE_MAX_CONNECTIONS, DATABASE_MIN_CONNECTIONS, DATABASE_IDLE_TIMEOUT_SECS, DATABASE_MAX_LIFETIME_SECS)
  - **Repository Factory Macro**: `repo!` macro in shared crate to reduce boilerplate for repository instantiation
  - **Messaging Topic Precomputation**: Precomputed static NATS subjects as constants to avoid repeated string allocations
  - **Rollen-Mapping Optimization**: Pre-converted role strings to UserRole enums during token validation, eliminating per-call conversion overhead
- **Mock Infrastructure Fixes**
  - Fixed mock feature flag propagation across domain, infrastructure, and postgres crates
  - Mock types now properly generated and exported when `mocks` feature is enabled
  - Updated all `#[cfg(any(test, feature = "mocks"))]` to `#[cfg(feature = "mocks")]` for consistent feature gating

### Changed
- Version bump: 0.8.1 → 0.8.2
- Mock feature now properly includes mockall dependency

### Fixed
- Infrastructure tests now compile and run with mock features enabled
- Domain crate mock types (MockSiteRepository, MockUserRepository, etc.) now properly available
- Consistent feature gating across workspace for mock functionality

## [0.8.1] - 2026-08-11

### Changed
- Increased version to 0.8.1 after verifying codebase with cargo fmt, check, test, and clippy.
- Fixed infrastructure tests to work with mock features.

## [0.8.0] - 2026-08-09

### Added
- **Full Integration Test Suite for API & Domain**
  - Implemented 10+ new integration test suites using mock repositories and messaging.
  - Added comprehensive coverage for Workforce, Compliance (Checklists, Audit, Plant Protection), Specialized Crops (Olives), Harvest Logistics (Seasons, Lots, Deliveries, Cold Chain), Livestock, Finance (PAC, Cost Centers, Records), Sites, Equipment, and Weather modules.
  - Verification of tenant-scoping, authorization, and DTO mappings across all major modules.
- **Enhanced Mocking Infrastructure**
  - Boxed `MockDatabase` variant to optimize memory layout and satisfy Clippy.
  - Added `set_mock_response` to `MessagingClient` for configurable request/response testing (NATS simulation).
  - Updated all integration tests to utilize the new optimized mock infrastructure.

### Fixed
- API: Implemented missing CRUD handlers for Plant Protection records.
- API: Fixed `PaginatedResponseDto` to support deserialization in tests.
- API: Fixed `Equipment` list handler to correctly utilize repository methods.

## [0.7.9] - 2026-08-09

### Fixed
- API: Workforce-Handler übergeben nun korrekt `&[UserRole]` an die Repositories (statt `&Vec<String>`), wodurch Sichtbarkeits-/Autorisierungsfilter wieder kompilieren und greifen.

## [0.7.8] - 2026-08-09

### Added
- Added API route-registration coverage for workforce, compliance, finance, PAC, harvest, livestock, weather, and olive modules.

### Changed
- Kept modules with remaining CRUD and authorization scenarios marked Nearly done in `tasks.md`.

## [0.7.7] - 2026-08-09

### Added
- Completed tenant-scoped cost-center and financial-record update/delete persistence.
- Added financial-record cost-center filtering with pagination.

### Changed
- Completed Module 14 and marked Finanzen: Kostenstellen production-ready.

## [0.7.6] - 2026-08-09

### Added
- Completed tenant-scoped weather-data and phenology-record update/delete persistence.

### Changed
- Completed Module 12 and marked Wetter & Phänologie production-ready.

## [0.7.5] - 2026-08-09

### Added
- Completed tenant-scoped water-source and water-quota repository operations.
- Added site/source filtering, pagination, quota balance initialization, and CRUD persistence.

### Changed
- Completed Module 9 and marked Wasser production-ready.

## [0.7.4] - 2026-08-09

### Added
- Completed vineyard site filtering and specialized vineyard CRUD routes.
- Added migration support for vineyard soft deletion.
- Added vineyard route integration coverage.

### Changed
- Completed Module 7 and marked Weinbau production-ready.

## [0.7.3] - 2026-08-09

### Added
- Completed TaskData update and delete repository operations.
- Added migration columns required by the TaskData domain model.
- Added task-route integration coverage.

### Changed
- Completed Module 2 and marked Aufträge & Tasks production-ready.

## [0.7.2] - 2026-08-09

### Added
- PostgreSQL-backed IoT device persistence with tenant-scoped CRUD.
- IoT route registration coverage in the API integration tests.

### Changed
- Completed Module 16 and marked IoT & Messaging production-ready.

## [0.7.1] - 2026-08-09

### Added
- Registered the IoT device API and its OpenAPI documentation.
- Added shared IoT device state to `AppState` so device registrations survive across requests.
- Added MQTT connection health tracking and health-monitoring lifecycle controls.

### Fixed
- Restored workspace compilation after the MQTT health-monitoring changes.
- Corrected IoT role validation, DTO parsing, and API error handling.
- Cleaned up workspace formatting and clippy findings.

## [0.7.0] - 2026-08-08

### Added
- **MQTT Support for IoT Devices & Home Assistant Integration**
  - `rumqttc 0.25` dependency with async MQTT client
  - `MqttConfig`: broker settings, TLS, authentication, topic prefix
  - IoT event types: `IoTTelemetryEvent`, `IoTDeviceStatusEvent`, `IoTCommandEvent`
  - `IoTCapability` enum: Temperature, Humidity, SoilMoisture, Light, GPS, BatteryLevel, SignalStrength, ActuatorControl, FirmwareUpdate, Custom
  - `MqttClient`: async connect, publish_telemetry, publish_status (retained), subscribe_commands/broadcast, event loop
  - `UnifiedMessagingClient`: dual NATS + MQTT backend with unified publish API
  - Topic structure: `agrocore/telemetry/{tenant}/{device}`, `agrocore/status/{tenant}/{device}`, `agrocore/commands/{tenant}/{device}`

- **Home Assistant MQTT Auto-Discovery**
  - `HaSensorConfig`, `HaBinarySensorConfig`, `HaButtonConfig`, `HaNumberConfig`, `HaDeviceInfo`
  - Capability mapping with device_class, unit_of_measurement, icons, value_templates
  - `generate_ha_discovery_configs()` for complete device payloads
  - Availability binary sensors with connectivity device_class

- **Mosquitto MQTT Broker in docker-compose**
  - `eclipse-mosquitto:2.0` on ports 1883 (MQTT) and 9001 (WebSockets)
  - TLS certificates (CA, server, client) + PKCS12 for Home Assistant
  - Password-based authentication
  - Health checks via `mosquitto_sub`

- **CI/CD Pipeline Fixes**
  - PostgreSQL service with sqlx migrations in GitHub Actions
  - Security audit with `continue-on-error: true`

### Changed
- **Version bump: 0.6.0 → 0.7.0**
- **Vulnerability fixes**: quick-xml 0.31→0.41, async-nats 0.38→0.50, rand 0.10→0.8, wiremock 0.5→0.6

### Fixed
- Clippy collapsible_if warnings in messaging crate
- MQTT borrow checker issue with state_topic clone

## [0.6.0] - 2026-08-05

### Added
- **Complete Test Suite for LPIS Providers & Settings**
  - 13 config tests (ProviderConfig, LpisProvidersConfig, CacheConfig, RateLimitConfig, RetryConfig, BaseClient creation, rate limiting)
  - 3 BaseClient integration tests with mock server (execute_request, get_cached_or_fetch, rate_limiting)
  - 4 Settings integration tests (route configuration, config serialization roundtrip)
  - 3 config serialization tests (LpisProviderConfig, LpisProvidersConfig, CacheBackend enum)
- **All 8 LPIS Providers migrated to BaseClient** (SIGPAC/ES, BRP/NL, RPG/FR, iLPIS/PT, SIAN/IT, LPIS-DE, LPIS-PL, INVEKOS/AT)
  - Unified caching, rate limiting, and retry logic across all providers
- **Settings API & UI complete** with persistence
  - Config file load/save (`load_from_path`, `save_to_path`)
  - Route configuration verified in tests

### Changed
- **Version bump: 0.5.9 → 0.6.0**
- **Test infrastructure** significantly expanded (13+ new tests)
- **Quality Gates** enforced in development workflow (fmt, check, test, clippy)

### Fixed
- `LpisProvidersConfig::load()` now correctly reads `config/lpis-providers.toml`
- `BaseClient` mock server tests use wiremock for reliable HTTP testing
- Config serialization tests cover roundtrip for all DTOs

## [0.5.9] - 2026-08-05

### Added
- **Multi-country LPIS Provider Base Client** with unified caching, rate limiting, and retry logic
  - New `BaseClient` in `lpis-providers` with HTTP client, `LpisCache` (Memory/Redis), `governor` rate limiting, exponential backoff retries
  - All 8 providers (ES, NL, FR, PT, IT, DE, PL, AT) can now use common infrastructure
  - BRP provider fully migrated to base client pattern
- **Admin UI: LPIS Country Selection in Data Import**
  - DataImport component shows country flags (ES, NL, FR, PT, IT, DE, PL, AT) for 8 LPIS sources
  - Click country flag opens provider-specific import modal with configuration fields
  - `CountrySelect` component with flag dropdown and search
  - `ProviderConfig` fields mapped to UI inputs (endpoint URL, cache backend, rate limit, retry config)

### Changed
- Version bump: 0.5.8 → 0.5.9
- Refactored all LPIS providers to use `BaseClient` for unified infrastructure

### Fixed
- `BaseClient` now correctly handles cache key generation with tenant_id prefix
- Fixed `LpisCache` Memory backend to use `HashMap<String, Vec<u8>>` with timestamp-based expiry
- `governor` rate limiting now correctly applies `per_second` limit instead of default burst
- Fixed `async-trait` usage in provider trait methods

## [0.5.8] - 2026-08-04

### Added
- **GeoJSON & Spatial Data Processing** (Task 3.7.4)
  - `geojson` crate integration across all LPIS providers
  - GeoJSONFeature/FeatureCollection types for parcel boundary data
  - Spatial intersection utilities for overlap detection
  - PostGIS integration for area calculations and spatial queries

## [0.5.7] - 2026-08-04

### Added
- **NATS Messaging Integration**
  - `async-nats` client with connection management and auto-reconnect
  - Message types: `TelemetryEvent`, `DeviceStatusEvent`, `CommandEvent`
  - `UnifiedMessagingClient` trait with NATS and mock implementations
  - Subjects: `agrocore.telemetry.{tenant}`, `agrocore.status.{tenant}`, `agrocore.commands.{tenant}`

## [0.5.6] - 2026-08-03

### Added
- **PostgreSQL Database Layer**
  - `PgSiteRepo`, `PgTenantRepo`, `PgWorkerRepo`, `PgOrderRepo`
  - sqlx with connection pooling via `PgPool`
  - Migrations for all entity tables
  - `Database` enum with `Postgres` and `Mock` variants

## [0.5.5] - 2026-08-02

### Added
- **Actix-web REST API Server**
  - JWT authentication middleware with role-based access control
  - OpenAPI/Swagger documentation via utoipa
  - Rate limiting with `actix-governor` (120 req/min default, 10 req/60s for auth)
  - CORS configuration
  - Error handling with `ApiError` enum

### Changed
- Version bump: 0.5.4 → 0.5.5

## [0.5.4] - 2026-08-01

### Added
- **Admin UI (Leptos 0.8)** — WASM SPA with sidebar navigation
  - Dashboard, Sites, Orders, Workers, Equipment, Inventory pages
  - Authentication flow with login page
  - Responsive layout with mobile drawer

### Changed
- Version bump: 0.5.3 → 0.5.4

## [0.5.3] - 2026-07-30

### Added
- **Domain Layer** — Core entities with validation
  - `Tenant`, `Site`, `Worker`, `Order`, `TaskData`, `User`
  - Validation traits using `validator` crate
  - Enum types: `OrderType`, `OrderStatus`, `UserStatus`
  - GeoJSON geometry types: `GeoPoint`, `GeoPolygon`, `GeoMultiPolygon`

### Changed
- Version bump: 0.5.2 → 0.5.3

## [0.5.2] - 2026-07-28

### Added
- **Shared Kernel Crate** — Common types and utilities
  - `SharedError` enum (Validation, NotFound, Internal, Network, Auth)
  - Pagination types: `PaginatedRequest`, `PaginatedResponse<T>`
  - Auth utilities: JWT secret, token generation
  - Database pool configuration helpers
  - `with_retry()` generic retry logic

### Changed
- Version bump: 0.5.1 → 0.5.2

## [0.5.1] - 2026-07-25

### Added
- Initial Rust workspace structure with Cargo workspace
  - 11 crates: api, domain, infrastructure, shared, lpis-providers, asset-registry, weather-service, geometry-service, reporting-service, messaging, admin-ui
  - Shared dependencies: actix-web 4, sqlx, serde, tokio, async-nats, rumqttc, geojson
  - All crates use Rust 2024 edition

### Changed
- Version bump: 0.5.0 → 0.5.1 (metadata cleanup)

## [0.5.0] - 2026-07-20

### Added
- **Initial Release**
  - Rust 2024 workspace with agrocore-rs
  - PostgreSQL with PostGIS extension
  - NATS messaging integration
  - MQTT broker with Home Assistant auto-discovery
  - Admin UI (WASM via Leptos)
  - Actix-web REST API with JWT auth
  - Multi-country LPIS providers (8 countries)
  - Inventory management with FIFO/FEFO
  - Clock-in/out with GPS coordinates

## [Unreleased]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.3...HEAD
## [0.9.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.2...v0.9.3
## [0.9.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.1...v0.9.2
## [0.9.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.9.0...v0.9.1
## [0.9.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.9.0
## [0.8.22]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.21...v0.8.22
## [0.8.16]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.15...v0.8.16
## [0.8.15]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.14...v0.8.15
## [0.8.14]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.13...v0.8.14
## [0.8.13]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.12...v0.8.13
## [0.8.11]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.10...v0.8.11
## [0.8.10]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.9...v0.8.10
## [0.8.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.8...v0.8.9
## [0.8.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.7...v0.8.8
## [0.8.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.6...v0.8.7
## [0.8.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.5...v0.8.6
## [0.8.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.4...v0.8.5
## [0.8.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.3...v0.8.4
## [0.8.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.2...v0.8.3
## [0.8.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.1...v0.8.2
## [0.8.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.8.0...v0.8.1
## [0.8.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.8.0
## [0.7.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.8...v0.7.9
## [0.7.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.7...v0.7.8
## [0.7.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.6...v0.7.7
## [0.7.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.5...v0.7.6
## [0.7.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.4...v0.7.5
## [0.7.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.3...v0.7.4
## [0.7.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.2...v0.7.3
## [0.7.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.1...v0.7.2
## [0.7.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.7.0...v0.7.1
## [0.7.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.7.0
## [0.6.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.6.0
## [0.5.9]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.8...v0.5.9
## [0.5.8]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.7...v0.5.8
## [0.5.7]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.6...v0.5.7
## [0.5.6]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.5...v0.5.6
## [0.5.5]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.4...v0.5.5
## [0.5.4]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.3...v0.5.4
## [0.5.3]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.2...v0.5.3
## [0.5.2]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.1...v0.5.2
## [0.5.1]: https://github.com/peopleandpixel/agrocore-rs/compare/v0.5.0...v0.5.1
## [0.5.0]: https://github.com/peopleandpixel/agrocore-rs/releases/tag/v0.5.0
