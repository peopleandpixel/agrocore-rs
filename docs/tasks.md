# Agrocore-RS Open Tasks

Letztes Update: 2026-10-01

Nur offene Arbeit. Erledigte Phasen und Module sind entfernt; die Historie steht im
CHANGELOG. Reihenfolge nach Priorität, innerhalb einer Priorität nach Abhängigkeit.

**Task 0 ist vorrangig vor allen Feature-Themen.** Er enthält sechs sofort ausnutzbare
Sicherheitslücken und die Voraussetzungen dafür, dass Konfiguration und Backend-
Funktionen überhaupt über die AdminUI bedienbar werden. Die Feature-Listen P0–P4 gelten
als Arbeit, die erst nach Task 0 sinnvoll begonnen werden kann.

Legende: **P0** blockiert den Betrieb · **P1** MVP · **P2** wichtig · **P3** Komfort ·
**P4** Zukunft.

---

## Task 0 — Sicherheits-, Vollständigkeits- und Performance-Audit (AKUT)

Code-Audit vom 2026-10-01 über den gesamten Workspace (154 API-Routen, 33 Handler-Module,
17 Crates). Befunde sind am Code verifiziert, nicht nur gesichtet.

**Zielbild:** Das System muss im normalen Modus **und** im Demo-Modus vollständig
funktionsfähig sein: alle Konfigurationen über die AdminUI einstellbar, alle
Backend-Funktionen über die AdminUI steuerbar.

**Reihenfolge:** erst Korrektheit (I0, J1, J2, D1), dann Sicherheit (A–E), dann
Persistenz (F), dann fehlende Backend-Funktionen (G), dann die AdminUI (H), dann
Performance (I3–I16).

**Übergeordnetes Muster:** Bei fast jedem Problem existiert die Infrastruktur, aber die
Verdrahtung fehlt. `is_revoked()` ist definiert und wird nie aufgerufen. `add_treatment`
existiert, die Route dahinter ist nicht registriert. `stock_in` existiert, wird nie
aufgerufen. `find_by_refresh_token` existiert, gibt `None` zurück. `mocks`-Feature
existiert, wird nie aktiviert. Das sind Verdrahtungsdefekte, keine Featuredefekte — und
tendenziell deutlich billiger zu beheben als neue Funktionen zu schreiben.

**Stand:** Alle vier Analysen sind ausgewertet — Blöcke A–J, 149 offene Punkte. Jeder
Befund wurde am Code oder gegen das Schema verifiziert, nicht nur gesichtet. `cargo check
--workspace` läuft sauber durch: es sind durchweg **Laufzeit-Bugs**, keine Compile-Fehler.

**Die drei Clusters, die zuerst zu beheben sind:**

1. **I1 — acht Tabellen fehlen im Schema.** Sieben Repos sind zur Laufzeit tot. `spatial_objects` wird von jedem GPS-Ping abgefragt, der Fehler wird per `unwrap_or_default()` geschluckt — die Standortzuordnung funktioniert nie und fällt nicht auf.
2. **A1 — Privilege Escalation** per `PUT /users/{eigene_id}`.
3. **J1/J2 — nicht registrierte Handler.** `/sigpac/parcels`, `/livestock/animals` und alle drei Site-Import-Endpunkte existieren nicht; der 719-Zeilen-`ImportService` ist toter Code.

**Hinweis zu I1:** Der Schema-Drift war der schwerwiegendste Einzelbefund des Audits.
I1, I2 und J6 sind erledigt (Migration `0000000003_missing_domain_tables.sql`). Damit
sind sechs Repos wieder funktionsfähig, der GPS-Ping liefert wieder Daten, und die
Fehler werden nicht mehr verschluckt.

**Weiterer Fund aus I1:** In `tree.rs`, `group.rs`, `building.rs` und `livestock.rs`
fehlte `tenant_id` im INSERT, obwohl alle SELECTs danach filtern — neu angelegte
Datensätze wären nicht auffindbar gewesen. Behoben.

### Block A — Sofort ausnutzbar, existenzielle Folgen (P0)

- [x] **A1 — Privilege Escalation: jeder User kann sich zum Admin machen** — erledigt (2026-10-01). `update_user` prüft jetzt `if dto.0.roles.is_some() || dto.0.is_active.is_some() { auth.require_admin()?; }` — Rollenwechsel und Aktivitätsstatus sind Admin-only, unabhängig davon, ob der eigene Account betroffen ist. Dazu ein neues `PUT /api/v1/users/me` mit `UpdateOwnProfileDto`, das nur `firstname`, `lastname`, `password`, `language`, `color` annimmt; alle übrigen Domain-Felder werden explizit auf `None` gesetzt, damit das Repository die Spalten unangetastet lässt. Die Route ist **vor** `/users/{id}` registriert, sonst würde `{id}` den Pfad „me" schlucken. `UpdateUserDto.password` verlangt jetzt `min = 12` statt der 8 aus `CreateUserDto` — über die alte Lücke ließ sich ein bestehendes Passwort auf einen leeren String setzen, das ist mit A1 und D3 behoben. 9 Regressionstests in `crates/api/tests/privilege_escalation_tests.rs`. — `handlers/users.rs:153-157`. `if let Err(e) = auth.require_admin() && auth.0.user_id != user_id` hebt den Admin-Check auf, sobald die eigene ID angesprochen wird. Der Body enthält `roles`, und `postgres/user.rs:318` bindet es ungeprüft. Angriff: `PUT /api/v1/users/{eigene_id}` mit `{"roles":["Admin"]}` → voller Admin-Zugriff. Rollenwechsel und `is_active` strikt admin-only; eigenes Profil über ein `/me`-Endpoint mit Feld-Whitelist (`firstname`, `lastname`, `password`, `language`).
- [x] **A2 — Demo-Routen löschen echte Tenants, unauthentifiziert** — erledigt (2026-10-01). Zwei unabhängige Barrieren: `require_demo_access()` in `handlers/demo.rs` prüft `AppState.demo_endpoints_enabled` (aus `ALLOW_DEMO_ENDPOINTS`, Default **aus**) **und** `auth.require_admin()`. Alle drei Endpunkte (`/seed`, `/reset`, `/summary`) nehmen jetzt einen `AuthExtractor`; vorher tat das keiner, auch `/summary` nicht. Die OpenAPI-Deklarationen tragen jetzt `security(("bearer_auth"))` sowie 401/403, vorher fehlten beide. Zusätzlich das hartkodierte `b"demo123"` entfernt: das Passwort kommt aus `DEMO_ADMIN_PASSWORD` mit Default `demo1234-agrocore`, und wird beim Fehlen der Variablen geloggt. Siehe G2 für die Passwort-Vereinheitlichung. 6 Regressionstests in `crates/api/tests/demo_endpoint_auth_tests.rs`. — `handlers/demo.rs:19-22,37,559`. `/seed`, `/reset`, `/summary` ohne `AuthExtractor`. `reset` erzwingt `reset=true` und führt `DELETE FROM tenants WHERE id = $1` mit Cascade aus; der Tenant-Slug kommt aus dem Request-Body. Zusätzlich hartkodiertes Admin-Passwort `demo123` (`:117`). Routen hinter `#[cfg(feature = "demo")]` + `require_admin()` + Env-Gate `ALLOW_DEMO_ENDPOINTS`; Passwort entfernen.
- [x] **Pin-Integration: Tenant-Pin in allen Datenbankpfaden** — erledigt (2026-10-02). A3 hatte die Policies scharf geschaltet, aber noch nichts setzte `app.current_tenant_id`. Ohne Pin liefern alle Repos null Zeilen. Behoben:

  **`crates/infrastructure/src/postgres/tenant_pool.rs`** — `TenantPool` pinnt vor jeder Query:
  - `set_config('app.current_tenant_id', ...)` und `set_config('app.is_superadmin','false')` auf **derselben** Verbindung, die die Query ausführt.
  - implementiert sqlx `Executor`, damit 317 Aufrufstellen in 47 Repos unverändert bleiben und der Pin nicht vergessen werden kann.
  - `begin()` sendet zuerst ein explizites `BEGIN` vor `set_config(..., true)`. `SET LOCAL` außerhalb eines Transaktionsblocks ist ein No-op — die andere Reihenfolge sieht funktionierend aus und setzt den Pin dann beim ersten Statement zurück.
  - `unscoped()` nutzt die Nil-UUID: passt zu keinem Tenant, verweigert also alles. Fail-closed für Bootstrap-Arbeit.
  - 7 Tests in `tests/tenant_pin_tests.rs`. Der entscheidende: `checkout_does_not_inherit_previous_tenant` — eine wiederverwendete Poolverbindung darf keinen Tenant eines vorherigen Requests übernehmen.

  **Bootstrap-Pfade bewusst ungepinnt**, mit Begründung im Code: `system/setup` und `demo/seed` erzeugen den ersten Tenant, können also nichts pinnen. `demo/summary` ermittelt den Tenant erst ungepinnt und pinnt danach auf dessen ID. `demo/seed` pinnt seine Transaktion direkt nach der Tenant-Anlage um.

  **Ein Auth-Pfad braucht eine Ausnahme, weil er den Pin noch nicht kennt:** Login liest den Tenant *aus* der User-Zeile. `users_select` verlangt `tenant_id = get_current_tenant_id()`, was dort nicht existiert. Rolle `agrocore_auth` (NOLOGIN, SELECT auf `users` + `user_sites`, Policy nur für diese Rolle) löst das. Verifiziert: unter `agrocore_app` liefert die Abfrage weiterhin 0 Zeilen, die Ausnahme ist also begrenzt.

  **Durch das Wirksamschalten vier weitere echte Bugs sichtbar geworden**, alle gegen eine frisch migrierte Datenbank verifiziert:
  - **Login komplett kaputt.** Fehlende Grants für 10 nach der RLS-Migration angelegte Tabellen; `user_sites` wird vom Login-Join gebraucht, also schlug jeder Login mit `permission denied` fehl.
  - **Refresh-Token-Write wirkungslos.** `update_refresh_token` schrieb ungepinnt, `users_update` verlangt den Pin → UPDATE traf 0 Zeilen → Handler meldete ehrlich `false`. Das war vorher als stilles Scheitern übersehen worden.
  - **`#[sqlx(json)]` auf `Option<T>` ist falsch.** sqlx kennt `json` (erzeugt `Json<T>`, nicht-null) und `json(nullable)` (erzeugt `Option<Json<T>>`). 29 Felder in 10 Dateien waren falsch annotiert. Ursache für `unexpected null; try decoding as an Option`.
  - **Schema-Drift bei `orders`:** `order_type` war `VARCHAR` statt JSONB, `planned_date`/`deadline_date` waren `DATE` gegen `DateTime`, und `started_at`/`completed_at` existierten in der Tabelle überhaupt nicht. `GET /api/v1/orders` war dadurch unerreichbar. 5 Konformance-Tests in `tests/order_schema_tests.rs`.
  - **37 `NUMERIC`-Spalten gegen `f64` in Rust.** Jede Entity, die eine davon berührte, scheiterte am Decode — `GET /api/v1/customers` an `vat_rate`. Auf `DOUBLE PRECISION` normalisiert, iterativ statt per Liste, damit die Lücke nicht zurückkommt.

  **Demo-Seed war fachlich falsch** und hätte nach den Typkorrekturen weiterhin 500er erzeugt: `seeding` und `fertilizing` existieren als `OrderType` nicht (korrekt: `soil_work`, `fertilization`), und `{"mode":"Manual"}` schrieb PascalCase, wo snake_case erwartet wird.

  **NATS war ein harter Startup-Blocker.** Ein fehlender Broker brach den gesamten API-Start ab, obwohl die Publisher ohnehin `let _ = …publish()` taten. Messaging ist jetzt optional; `MESSAGING_REQUIRED=1` erzwingt es, wenn es deploymentskritisch ist. Reports brauchen den Broker weiterhin und sagen das explizit.

  **Verifiziert gegen eine frische Datenbank, alle Migrationen von null, Demo-Seed geladen:**
  ```text
  POST /api/v1/auth/login       -> 200
  GET  /api/v1/health          -> 200
  GET  /api/v1/sites           -> 200
  GET  /api/v1/users           -> 200
  GET  /api/v1/orders          -> 200
  GET  /api/v1/orders/my-tasks -> 200
  GET  /api/v1/customers       -> 200
  GET  /api/v1/inventory/items -> 200
  GET  /api/v1/tasks           -> 200
  ```

- [x] **A3 — RLS existiert, war aber wirkungslos** — erledigt (2026-10-01). Drei Ursachen, alle behoben:

  **1. Die Verbindungsrolle war Superuser mit BYPASSRLS.** Gemessen auf dieser Installation: `agrocore` hat `rolsuper = true` **und** `rolbypassrls = true`. PostgreSQL exemptet solche Rollen von RLS **bedingungslos** — `FORCE ROW LEVEL SECURITY` ändert daran nichts. Solange die Anwendung als diese Rolle verbindet, sind 190 Policies Dekoration. Migration `0000000004_force_rls.sql` führt `agrocore_app` ein (NOSUPERUSER, NOBYPASSRLS, NOLOGIN) und der Pool schaltet per `after_connect` mit `SET ROLE agrocore_app` darauf um.

  **2. `FORCE ROW LEVEL SECURITY` fehlte** (0 Treffer). Die Migration setzt es auf alle 62 Tabellen mit RLS — verifiziert: 62 Tabellen, alle mit `relforcerowsecurity`.

  **3. `app.current_tenant_id` wurde nirgends gesetzt** (0 Treffer im Rust-Code). `get_current_tenant_id()` lieferte NULL, jede Policy verglich `tenant_id = NULL`. Ich habe die Policies empirisch geprüft: mit Pin auf Tenant A sieht die Rolle nur A-Daten, Tenant B nur B, unbekannte UUID und kaputter Wert ergeben 0 Zeilen.

  **Dabei zwei echte Bugs gefunden, die erst durch das wirksam Schalten sichtbar wurden:**
  - `sigpac_parcels` hatte RLS aktiviert, aber **keine Policy**. Mit FORCE hätte das jede Zeile für jeden verweigert.
  - `tenants` hatte Policies für SELECT/UPDATE/DELETE, aber **keine für INSERT**. Mit FORCE wäre `POST /api/v1/system/setup` — der Endpunkt, der den ersten Tenant anlegt — für jede Installation gescheitert. Policy `tenants_insert ... WITH CHECK (true)` ergänzt: Tenant-Anlage ist eine Setup-Aktion, die vor dem Bestehen eines Tenants stattfindet, also nicht mandantengefiltert.
  - 6 Tabellen mit `tenant_id`-Spalte hatten gar kein RLS; für alle außer `tenants` ergänzt.

  **Geschaltet über `AGROCORE_RLS_ENABLED`**, nicht per Release: die Policies vor dem Pin zu aktivieren würde alle Repos null Zeilen liefern lassen. Der Schalter ist jetzt eine Konfigurationsänderung, kein koordiniertes Release. 5 Tests in `crates/infrastructure/tests/rls_tests.rs` belegen die Wirkung. — Init-Migration + Workspace. 190 Policies, 50× `ENABLE ROW LEVEL SECURITY`, aber `app.current_tenant_id` wird nirgends gesetzt (0 Treffer im Rust-Code) und `FORCE ROW LEVEL SECURITY` fehlt (0 Treffer). Verbindung als Tabellen-Owner umgeht RLS. Die gesamte Tenant-Isolation hängt damit zu 100 % an den Repo-Queries. `SET LOCAL app.current_tenant_id` pro Transaktion setzen, `FORCE ROW LEVEL SECURITY` ergänzen, eigene App-Role ohne `LOGIN`. Die `agrocore_app`-Rolle wird derzeit nur für Views grantet und ist für Isolation nutzlos.
- [x] **A4 — JWT-Revocation wird nie geprüft, Logout ist wirkungslos** — erledigt (2026-10-01). `AuthExtractor` prüft nach erfolgreichem `decode` die `jti` gegen `AppState.token_revocation.is_revoked(...)` und lehnt mit 401 „Token revoked" ab. Dafür musste der `FromRequest`-Future von `Ready` auf einen `Pin<Box<dyn Future>>` umgestellt werden, weil die Revocation-Liste asynchron ist (Redis oder In-Memory); `from_request` klont Header und State-Handle, damit der Future nichts leiht. Vorher take nur `revoke()` auf, nie `is_revoked()` — ein gestohlener Token blieb bis zum Ablauf gültig, und die UI sendete den Logout-Request gar nicht (H6). — `middleware.rs:90`. `is_revoked()` ist definiert, hat aber null Aufrufe; nur `revoke()` wird genutzt. Logout und Passwortwechsel widerrufen nichts — ein gestohlener Token bleibt 30 Minuten gültig. In `AuthExtractor::from_request` nach erfolgreichem `decode` prüfen.
- [x] **A5 — JWT-Secret fällt auf `"dev-secret"` zurück** — erledigt (2026-10-01). `run_server` bricht jetzt mit einem `io::Error` ab, bevor der Server lauscht. `validate_jwt_secret()` liefert nicht mehr nur `bool`, sondern `Result<(), JwtSecretError>` mit zwei Varianten: `DevSecret` (unset) und `TooShort { length, minimum }` — die Behebung unterscheidet sich, also muss die Meldung es auch. Zusätzlich zur bisherigen Prüfung gegen das Literal gilt jetzt eine Mindestlänge von 32 Zeichen (`JWT_SECRET_MIN_LENGTH`, Breite eines SHA-256-Digests als übliche Untergrenze für einen symmetrischen HMAC-Schlüssel). Der Entwicklungs-Ausweg `ALLOW_DEV_SECRET=1` entschärft nur den Default-Secret-Check, nie die Längenprüfung. 5 Tests in `crates/api/tests/jwt_secret_tests.rs`. — `shared/config.rs:105,132-134,277`. `validate_jwt_secret()` existiert, hat aber null Aufrufer. Production ohne `JWT_SECRET` bedeutet: jeder kann HS256-Admin-Tokens fälschen und sich in beliebige Tenants setzen. In `main.rs` vor `run_server` hart abbrechen, Länge ≥ 32 Bytes erzwingen (`ALLOW_DEV_SECRET=1` als Opt-out für Entwicklung).

### Block B — Tenant-Bruch und Datenverlust (P0)

- [x] **B1 — IDOR auf `kelter_deliveries`** — erledigt (2026-10-01). **Korrektur zum Audit:** Die Tabelle existierte sehr wohl, in `0000000000` bei Zeile 875 — sie hatte nur keine `tenant_id`-Spalte, und die RLS-Policies des Init-Schemas scopen über `vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = ...)`, was das Repository vollständig umgeht. Migration `0000000003` ergänzt deshalb per `ALTER TABLE` eine `tenant_id`-Spalte statt die Tabelle neu anzulegen, mit Backfill aus dem zugehörigen Weinberg. Eine verwaiste Zeile ohne Weinbergs-Tenant bricht die Migration mit einer klaren Meldung ab, statt sie still einem beliebigen Mandanten zuzuordnen. Alle sieben Repository-Methoden filtern jetzt `WHERE tenant_id = $n`; `KelterDelivery` hat das Feld im Domain-Modell. **Verifiziert** mit vier Integrationstests in `crates/infrastructure/tests/tenant_isolation_tests.rs` und direkt gegen die Datenbank: Tenant B sieht in `find_all` nur eigene Zeilen, `find_by_id` mit korrekter UUID liefert nichts, ein Cross-Tenant-`DELETE` betrifft 0 Zeilen, `find_by_vineyard` bleibt gescoped. `test_tenant_scoped_tables_have_tenant_id`: `kelter_deliveries` hat weder eine `tenant_id`-Spalte im Schema noch einen Filter im Repository — `postgres/kelter_delivery.rs:18,37,43,73,148,169`. Alle sieben Methoden nehmen `tid: TenantId` entgegen und verwenden es in keiner einzigen Query; `find_all` liefert global über alle Mandanten. Die Tabelle existiert in keiner Migration, die Queries würden zur Laufzeit fehlschlagen. `AND tenant_id = $n` in allen Queries ergänzen; Migration mit `tenant_id NOT NULL` + FK anlegen.
- [ ] **B2 — Restore überschreibt eine vom Client benannte Datenbank** — `handlers/backup.rs:218`. `target_database` kommt ungeprüft aus dem Request, `RestoreRequest` wird nicht validiert (anders als `CreateBackupRequest`). Serverseitig aus der Backup-Konfiguration ableiten, Feld aus dem DTO entfernen.
- [ ] **B3 — `delete_device` prüft die Tenant-Zugehörigkeit nicht** — `handlers/iot.rs:345-361`. Die Bedingung `device.tenant_id != auth.tenant_id && !auth.is_admin()` ist nach dem vorangestellten `require_admin()` immer `false`; das `DELETE` hat selbst keine Tenant-Bedingung. Ein Mandanten-Admin kann Geräte anderer Mandanten löschen. Admin darf die Tenant-Grenze nicht aufheben — Superadmin als separates, global vergebenes Flag.

### Block C — Informationsleck und Abusbarkeit (P1)

- [ ] **C1 — DB-Fehlermeldungen mit Schema-Details gehen an den Client** — `api/error.rs:103-106`, `postgres/error_mapper.rs:9-11`. `self.0.to_string()` für alle Fehlertypen inklusive `Database`; Postgres-Meldungen enthalten Tabellen-, Constraint- und Spaltennamen. Dazu explizit durchgereichte Details: `backup.rs:112,220`, `settings.rs:198`, `auth.rs:60,113,124,157,195,216`, `reporting.rs:66,103,136,173`, `system.rs:121`. Bei 500 generische Message plus Korrelations-ID ausgeben und nur serverseitig loggen; `map_db_error` auf Constraint-Codes statt -Messages abbilden.
- [ ] **C2 — `/metrics/db` und `/metrics/business` ohne Authentifizierung** — `api/lib.rs:158-162`, global registriert, ohne `AuthExtractor`. Enthält Query-Timings, Pool-Statistiken und Datensatzanzahlen pro Tenant. Eigenen Governor-Scope plus `require_admin()`, oder an ein separates Port/Mesh-Netz binden.
- [ ] **C3 — Kein Payload-Limit, DoS über den Import** — `api/main.rs`. Kein `JsonConfig`/Payload-Limit gesetzt; der GeoJSON-/Shapefile-Import nimmt unbegrenzte Bodies in den Speicher.
- [ ] **C4 — CORS fällt still auf `permissive` zurück** — `api/lib.rs:55-75`. Ohne `CORS_ALLOWED_ORIGINS` gilt `Cors::permissive()`, im anderen Zweig `allow_any_header` mit `supports_credentials`. Nur ein `warn!` als Schutz. In Production hart fehlschlagen statt `permissive` zu setzen.

### Block D — Fehlende Authentifizierungsmechanik (P1)

- [x] **D1 — Refresh-Token-Mechanik ist komplett tot, drei echte Stubs** — erledigt (2026-10-01). Die drei Methoden in `postgres/user.rs` sind implementiert: `find_by_refresh_token` filtert Ablauf und `is_active` bereits im SQL, damit ein abgelaufener Token von einem unbekannten nicht unterscheidbar ist; `invalidate_refresh_token` setzt beide Spalten auf NULL und meldet über `WHERE refresh_token IS NOT NULL`, ob wirklich etwas widerrufen wurde; `update_refresh_token` schreibt Token und Ablauf plus `updated_at`. In `auth.rs` prüfen Login und Refresh jetzt den Rückgabewert — vorher prüfte nur `map_err`, ein `Ok(false)` wäre durchgerutscht, der Client hätte 200 mit einem Token bekommen, der nie in der Datenbank stand. Der Refresh rotiert den Token, wodurch Wiederverwendung erkennbar wird. — `postgres/user.rs:402-420`, aufgerufen aus `handlers/auth.rs:51-58` und `:122-124`. `find_by_refresh_token` gibt `Ok(None)` zurück (`:405`, mit dem Kommentar „Simplified - not fully implemented"), `invalidate_refresh_token` und `update_refresh_token` geben je `Ok(false)` (`:410`, `:419`). Die Spalten `refresh_token` und `refresh_token_expires_at` existieren (Migration `:301-302`) und werden in `authenticate()` (`:372`) korrekt geschrieben. **Der Fehler bleibt still:** `auth.rs:60` prüft nur mit `map_err`, der Stub liefert aber `Ok(false)` — kein Fehler. Der Login antwortet also 200 und übergibt dem Client einen Refresh-Token, der nie in der Datenbank steht; `/auth/refresh` gibt daraufhin immer 401 (`:97`), `/auth/logout` (`:163`) widerruft nichts, weil `Ok(false)` kein Fehler ist. Drei echte Queries ergänzen (`WHERE refresh_token = $1 AND refresh_token_expires_at > NOW() AND is_active = true`, Invalidierung auf `NULL`, Update mit Token und Ablauf) **und** die drei Rückgabewerte in `auth.rs:58-60`/`:122-124`/`:163` explizit prüfen. Opaque 32-Byte-Zufallswert hashen (SHA-256), Rotation und Reuse-Detection ergänzen.
- [ ] **D2 — Impersonation ist tot und potentiell zu weit** — `handlers/auth.rs:177-184`. Der Rollenvergleich prüft case-sensitiv auf `"admin"`/`"superadmin"`, JWT-Rollen werden aber als `"Admin"` erzeugt → niemand kann impersonieren. Würde der Vergleich case-insensitiv, blieben drei Probleme: der Impersonations-Token landet nicht in der Revocation-Liste, es gibt keinen Audit-Log-Eintrag, der Impersonations-JWT trägt keinen `impersonator_id`-Claim. Absichtlich deaktivieren (`ALLOW_IMPERSONATION`-Gate), Audit-Log-Pflicht.
- [x] **D3 — Passwort-Update ohne Mindestlänge** — mit A1 erledigt (2026-10-01). `UpdateUserDto.password` hat jetzt `#[validate(length(min = 12, max = 128))]`; `UpdateOwnProfileDto.password` ebenfalls. `CreateUserDto` bleibt bei 8, um bestehende Konten nicht zu brechen. Tests dafür in `privilege_escalation_tests.rs`. — `dto/user.rs:96`. `password: Option<String>` ohne `#[validate(...)]`, während `CreateUserDto:67` `min = 8` setzt. Über A1 ausnutzbar, damit ist das Passwort auf einen leeren String setzbar. `length(min = 12, max = 128)` plus Rehash beim Login.

### Block E — Fehlende Validierung (P1)

- [ ] **E1 — 132 POST/PUT-Routen, nur 36 `.validate()`-Aufrufe** — ohne jedes `validate()`: `agriculture.rs` (8), `weather.rs` (10), `workforce.rs` (9), `harvest.rs` (8), `compliance.rs` (7), `finance.rs` (6), `water.rs` (6), `livestock.rs` (4), `breed/building/group/tree/variety/livestock_new.rs` (je 2), `nutrition.rs` (2). `validator::Validate` mit Längen-, Wertebereichs- und Enum-Constraints auf alle Command-DTOs; `require_manager()` als Minimum auf allen schreibenden Routen.
- [ ] **E2 — `per_page` ohne Cap, Memory-DoS** — `shared` Pagination wird ohne Obergrenze durchgereicht (`postgres/user.rs:128`, `equipment.rs:128`); nur `sigpac.rs:92` begrenzt mit `.min(200)`. Cap im Deserializer auf maximal 200, `page` ebenfalls begrenzen.

### Block F — Konfiguration ist nicht persistierbar (P0, Voraussetzung für das Zielbild)

Ohne diese Punkte ist das Zielbild „alle Konfigurationen über die AdminUI einstellbar"
nicht erreichbar. Es gibt derzeit **keine Stelle im Schema**, an der Konfiguration
gespeichert werden könnte.

- [x] **F1 — Keine Settings-/Config-Tabelle im gesamten Schema** — `migrations/*.sql`. Einziger Treffer ist `soil_moisture_configs`; es existiert keine `system_settings`, `tenant_settings` oder allgemeine `config`-Tabelle. Migration anlegen: Schlüssel, Wert (JSONB), Tenant-Bezug, `updated_at`, `updated_by`; Versionierung und Defaults vorsehen.
- [x] **F2 — Backup-Konfiguration ist hartkodiert, das Update speichert nichts** — `handlers/backup.rs:41-70`. `get_backup_config` gibt fixe Werte zurück (`schedule_db: "0 2 * * *"`, `enabled: true`, `targets_count: 0`). `update_backup_config` enthält `// TODO: Persist config changes`, antwortet aber mit Erfolg — der Admin glaubt, es sei gespeichert. Auf `system_settings` umstellen; Ziele, Zeitplan, Retention und Verifikation wirklich persistieren und beim Start laden.
- [ ] **F3 — Settings-API kennt nur LPIS** — `handlers/settings.rs:34-44`. Registriert sind ausschließlich `/settings/lpis` (GET/PUT) und `/settings/lpis/providers`. Der Rest des Backends hat keine Konfigurationsendpunkte. Ressourcen für Backup, Scheduler, Benachrichtigungen, Wetter, LPIS-Cache, Mandant, Sicherheit und Darstellung ergänzen.
- [ ] **F4 — Nur-LPIS-Settings schreibt in eine TOML-Datei im Dateisystem** — `handlers/settings.rs:15-32,196-198`. `find_config_file()` sucht relativ von `current_dir` nach oben; `update_lpis_settings` schreibt nach `config/lpis-providers.toml`. Das funktioniert nicht containerisiert und nicht mandantenfähig. In die Datenbank verlagern; Dateipfad und Config-Fehler dürfen nicht an den Client gehen (siehe C1).
- [ ] **F5 — Debug-Ausgaben im Produktivcode** — `handlers/settings.rs:77,82,89,104`. Viermal `eprintln!("DEBUG HANDLER: ...")` mit Dateipfad und Config-Details; geht bei jedem Settings-Aufruf auf stdout. Entfernen, stattdessen strukturiert loggen.
- [ ] **F6 — LPIS-Provider-Credentials liegen in einer versionierten TOML-Datei** — Provider-URLs und Zugangsdaten stehen in `config/lpis-providers.toml` statt mandantenfähig in der DB. Als Secrets behandeln, Credentials nicht in eine versionierte Datei schreiben.

### Block G — Backend-Funktionen fehlen (P0, nicht nur in der UI)

Die AdminUI kann diese Funktionen nicht anbieten, weil sie im Backend nicht existieren.

- [ ] **G1 — Kein Update für die Kern-Entities** — `handlers/{sites,equipment,orders,users,inventory,tasks}.rs`. `web::put()` und `web::patch()` kommen im gesamten Backend **null Mal** vor; es gibt nur Create und Delete. Sites, Equipment, Orders, Tasks und Inventar sind darüber nicht bearbeitbar. PUT-Routen plus Repo-`update` mit demselben Tenant-Filter wie `create` ergänzen.
- [x] **G2 — Die beiden Demo-Seed-Wege erzeugen unterschiedliche Passwörter** — erledigt (2026-10-01) im Zuge von A2. Eine Quelle definiert jetzt das Passwort: `DEMO_ADMIN_PASSWORD`, Default `demo1234-agrocore` (17 Zeichen, erfüllt das seit v0.25.0 geltende Minimum von 12). Alle drei Stellen lesen daraus beziehungsweise verwenden denselben Wert: `handlers/demo.rs` (`DEMO_DEFAULT_PASSWORD`), `scripts/demo_seed.sql` (Argon2id-Hash neu erzeugt) und `scripts/dev.sh` (Anzeige nutzt die Variable statt eines Literals). Empirisch verifiziert: der Hash akzeptiert `demo1234-agrocore` und lehnt `demo1234` sowie `demo123` ab. — `scripts/demo_seed.sql` speichert für `admin@demo.local` einen Argon2id-Hash, der mit `demo1234` verifiziert (empirisch gegen `argon2 0.6`/`password-hash 0.6` geprüft); `handlers/demo.rs:117` hasht dagegen `b"demo123"`, und `scripts/dev.sh:471` zeigt `demo1234` an. Wer über den API-Seed geht, kann sich mit dem angezeigten Passwort nicht anmelden und umgekehrt. Eine Quelle definieren und beide Wege daraus ableiten.
- [ ] **G3 — Demo-Modus ist kein Modus, sondern ein einmaliger Seed** — `DEMO_MODE` kommt im gesamten Rust-Code **nirgends** vor, weder in der API noch in der AdminUI. Der Modus existiert nur als Shell-Seed in `scripts/dev.sh` und `scripts/demo_seed.sql`. Durchgängig verdrahten: Flag in `AppState`, Guard-Middleware, abweichendes Verhalten (schreibgeschützt für Produktionsdaten, Resets erlaubt), Anzeige in der UI.
- [ ] **G4 — Tote Repository-Stubs entfernen** — siehe D1. Die Refresh-Token-Stubs zusätzlich aus dem `UserRepository`-Trait entfernen, damit tote Signaturen nicht erhalten bleiben.

### Block H — AdminUI erreicht große Teile des Backends nicht (P1)

33 Backend-Bereiche werden von der UI überhaupt nicht angesprochen. Verifiziert durch
Abgleich aller 154 Routen gegen die 102 von der UI genutzten Pfade.

- [x] **H1 — Die Settings-Seite ist reine Anzeige** — `admin-ui/src/components/settings.rs` (395 Zeilen). Null `Input`, null `Checkbox`, null `Select`, null `on_input`, null API-Schreibaufrufe. Keine einzige Konfiguration ist editierbar. Formulare gegen F3 bauen.
- [ ] **H2 — Keine Update-Funktionen in der UI** — `admin-ui/src/api.rs`, 46 Schreibfunktionen. Nur `user` und `worker` haben vollständiges CRUD; alle anderen Entities nur Create und Delete. Selbst wo das Backend PUT anbietet, wird es nicht aufgerufen. Funktionssatz zu G1 ergänzen und Edit-Formulare in den Detailseiten ergänzen.
- [x] **H3 — Backups sind in der UI nicht vorhanden** — `handlers/backup.rs` bietet Config, Liste, Detail, Status, Restore und Delete; die UI ruft davon nichts auf. Backup-Seite bauen: Konfiguration, manueller Backup-Lauf, Liste mit Status und Fortschritt, Restore inklusive `dry_run`, Fehlerprotokoll. Hängt an F2, sonst ist die Konfiguration nicht speicherbar.
- [ ] **H4 — Fünf komplette Seiten sind Attrappen** — `admin-ui/src/components/`. `groups.rs` (23 Z.), `trees.rs` (19 Z.), `livestock.rs` (19 Z.) und `buildings.rs` (34 Z.) haben Formulare, deren Submit-Handler nur `let _ = (label.get(), ...)` ausführen — es geht nichts an das Backend. **Schlimmer: `buildings.rs:21` zeigt eine Erfolgsmeldung „Gebäude angelegt" ohne jeden HTTP-Aufruf**, der Nutzer glaubt also, es sei gespeichert. `plot_subentity.rs:11-14` ist eine hartkodierte Beispieltabelle („Herde 1", „Ziegen", „Korkeiche"). Die serverseitigen CRUD-Pendanten `/groups`, `/trees`, `/buildings`, `/livestock` sind fertig und werden nie benutzt. Alle fünf Seiten verdrahten oder entfernen; eine gefälschte Erfolgsmeldung darf nicht bleiben.
- [ ] **H5 — Neun API-Wrapper sind tot** — `admin-ui/src/api.rs`. Definiert, aber ohne jeden Aufrufer in `components/`: `stock_in`, `stock_out`, `transfer_inventory`, `adjust_inventory` (die gesamte Lager-Logik), `update_inventory_item`, `update_worker`, `refresh_token`, `logout`, `fetch_health`. Der Bestand kann darüber weder aufgefüllt noch entnommen noch umgelagert noch korrigiert werden.
- [x] **H6 — Der Logout-Button sendet keinen Request** — mit A4 behoben (2026-10-01). Die serverseitige Seite (Revocation-Prüfung im Extractor) ist damit abgeschlossen; der clientseitige Aufruf von `api::logout()` aus `lib.rs:343-347` ist noch offen und als solcher in H5 vermerkt. — `admin-ui/src/lib.rs:343-347`. Er ruft nur `api::clear_auth_token()` und `clear_user_role()` im Browser und lädt neu; `/api/v1/auth/logout` wird nie aufgerufen. Zusammen mit A4 bleibt der JWT serverseitig gültig — nach dem Logout kann der Token weiterverwendet werden. Auf `api::logout()` umstellen.
- [ ] **H7 — `livestock_new::configure` ist doppelt registriert, `livestock::configure` gar nicht** — `api/handlers/mod.rs:244` und `:250` rufen beide `livestock_new::configure` auf; `livestock::configure` kommt nirgends vor. Damit sind die Routen aus `livestock.rs` — darunter `/livestock/animals/{id}/treatments` (`:212`) und `/grazing` (`:213`) — **nicht erreichbar**, obwohl die UI die Wrapper `add_treatment` und die Grazing-Aufrufe besitzt. Zusätzlich ist `/nutrition/*` doppelt registriert (`calculation.rs:34` und `nutrition.rs:12`). Beide Konsolidieren und prüfen, welche Scope gewinnt.
- [ ] **H8 — Die Settings-Seite täuscht Bedienbarkeit vor** — `admin-ui/src/components/settings.rs`. Die LPIS-Sektion (`:302-354`) zeigt acht Länderkarten mit `<input>` **ohne `prop:value`, ohne `on:input` und ohne Signal** (`:325,329,333,337,342`); der Button (`:351`) hat **kein `on:click`**. Sieht wie ein Formular aus, ist aber eine Attrappe. Die Zeitzone (`:293-296`) hat einen `<select>` ohne `on:change`. Systemstatus (`:361-373`) sind statische Badges, die Version ist als `"v0.4.2-stable"` hartkodiert (`:372`), obwohl das Projekt bei `0.23.0` steht. Das Firmenprofil (`:53-66`) liegt ausschließlich in `localStorage` (`api.rs:133-140`) — pro Gerät, nicht mandantenfähig, bei Tenant-Wechsel verloren. `fetch_lpis_providers` (`api.rs:2090`) existiert, wird aber nie aufgerufen.
- [ ] **H9 — Vier große Feature-Blöcke sind serverseitig fertig und clientseitig nicht vorhanden** — Für diese Bereiche existieren die Routen vollständig, die UI hat nichts: **Backup** (12 Routen, `backup.rs`), **Wasser** (12, `water.rs`), **Harvest** (16, `harvest.rs`), **Agrar-Spezial** (16, `agriculture.rs` mit `olive-groves`, `olive-oil-records`, `vineyards`, `kelter-deliveries`). Dazu IoT (`iot.rs`, `send_device_command`-Wrapper vorhanden) und 11 von 13 `/calculate/*`-Endpunkten. Das sind rund 80 Routen ohne jede Bedienoberfläche.
- [ ] **H10 — Workforce Task-Fortschritt ist unsichtbar** — Alle 5 Routen unter `/workforce/tasks/{id}/status*` werden nicht aufgerufen, obwohl sechs Wrapper in `api.rs:1351-1409` existieren. Task-Fortschritt, Aggregation und Worker-Status sind damit über die UI nicht einsehbar; `/workforce/logs` und `/workforce/locations` ebenfalls komplett ungenutzt.
- [ ] **H11 — Lagerbestand ist nicht buchbar** — Alle vier Buchungspfade `/inventory/stock-in`, `stock-out`, `transfer`, `adjust` werden nicht aufgerufen (Wrapper vorhanden, `api.rs:1697-1710`). Ohne sie lässt sich kein Bestand verändern. Zusätzlich sind `fetch_all_inventory_transactions` (`api.rs:2126`), die Transaktionen je Item (`api.rs:1671`) und `POST /inventory/locations` ungenutzt.
- [ ] **H12 — Compliance- und Wetterbereiche ungenutzt** — `/compliance/fertilizer` komplett, `/compliance/applicator-licenses` komplett, `/compliance/plant-protection` nur lesend; `/weather/frost-warnings` (+ `/active`), `/weather/gdd`, `/gdd/accumulated`, `/pest-risks` komplett; `create_weather_station` (`api.rs:1086`) und `create_weather_data` (`api.rs:1105`) tot. Kein Stations-Management.
- [ ] **H13 — Weitere nicht genutzte Bereiche** — `profitability`, `seasons`, `quotas`, `cold-chain`, `deliveries`, `olive-groves`, `vineyards`, `pest-risks`, `phenology`, `harvest`, `breed`, `variety`, `building`, `lots`, `water-rate`, `workflow`, `groups`, `logs`, `config`, `demo`, `material`, `stations`, `sources`, `data`, `summary`, `usage`, `difficulty-surcharge`, `tree-crown-volume`, `nitrogen-demand`, `agriculture`. Für jedes prüfen: bewusst ausgeblendet (dokumentieren) oder fehlende Seite ergänzen.

### Block I — Performance (P0/P1)

Audit vom 2026-10-01, Indizes gegen die Queries abgeglichen. Effektangaben sind
Erwartungen aus Query-Shape- und Index-Analyse, nicht gemessen.

#### I-0 Korrektheitsdefekt vor Performance: acht Tabellen fehlen im Schema (P0)

Acht Tabellen werden von Repos abgefragt, existieren aber in **keiner** Migration
(52 Queries betroffen). Die betroffenen Funktionen sind zur Laufzeit tot:

| Tabelle | Queries | Repo |
|---|---|---|
| `spatial_objects` | 5 | `site.rs` |
| `buildings` | 8 | `building.rs` |
| `trees` | 9 | `tree.rs` |
| `groups` | 9 | `group.rs` |
| `livestock` | 9 | `livestock.rs` |
| `water_usages` | 9 | `water_usage.rs` |
| `animal_treatments` | 2 | `animal.rs` |
| `animal_grazing_records` | 1 | `animal.rs` |

- [x] **I1 — Migrationen für die acht fehlenden Tabellen schreiben** — erledigt in `migrations/0000000003_missing_domain_tables.sql` (2026-10-01). **Dabei zusätzlich gefunden und behoben:** die INSERTs in `tree.rs`, `group.rs`, `building.rs` und `livestock.rs` haben `tenant_id` nicht gebunden, obwohl alle SELECTs danach filtern — ein neu angelegter Datensatz wäre nicht auffindbar gewesen. Migration ist dreimal hintereinander auf derselben Datenbank gelaufen (Idempotenz bestätigt), alle vier Migrationen laufen in einer frischen Datenbank in Reihenfolge durch, der Demo-Seed läuft danach durch. — Die Repos sind implementiert, das Schema nicht. Der Fehler wird zusätzlich verschluckt: `api/handlers/workforce.rs:353,364` ruft `spatial_object_repo().find_containing_point(...)` mit `.unwrap_or_default()`. Jeder GPS-Ping eines Workers läuft damit zweimal in eine nicht existierende Tabelle, das Ergebnis ist immer leer, und die Standort-zu-Feld-Zuordnung **funktioniert nie, ohne zu auffallen**. Erst die Migrationen schreiben, dann I8 (N+1 dort) — jede darauf aufgebaute Optimierung ist vorher wirkungslos.
- [x] **I2 — Fehler nicht mehr verschlucken** — erledigt in `handlers/workforce.rs` (2026-10-01). Die beiden `.unwrap_or_default()` auf `spatial_object_repo().find_containing_point(...)` sind durch `match` mit `warn!` ersetzt: Tenant, Koordinaten und Fehlertext werden geloggt, der Location-Ping läuft weiter, aber der Fehler ist nicht mehr unsichtbar. — Alle `.unwrap_or_default()` auf Repo-Aufrufen in `workforce.rs` und ähnlichen Stellen durch Logging mit Tenant- und Objektbezug ersetzen; ein leeres Ergebnis darf nicht von einem Fehler ununterscheidbar sein.

#### I-A Sofort, hohe Wirkung, kleiner Aufwand (P0)

- [ ] **I3 — SIGPAC-Near-Point-Suche nutzt den GIST-Index nicht (PostGIS-Typ-Mismatch)** — `handlers/sigpac.rs:353-354`. Die Query filtert auf `ST_DWithin(geography(geometry), geography(...))`, der vorhandene Index ist `USING GIST(geometry)` (Migration `:1496`), also auf `geometry`, nicht `geography`. Die Umhüllung macht den Index unbenutzbar → Seq-Scan über alle SIGPAC-Parzellen, zusätzlich `ORDER BY ST_Distance` als K-Sort-Sortierung ohne Distanzindex. `CREATE INDEX idx_sigpac_parcels_geog ON sigpac_parcels USING GIST (geography(geometry));` — größter Einzelhebel im Repo.
- [ ] **I4 — Argon2 blockiert den Async-Runtime-Thread** — `postgres/user.rs:183-187`. `Argon2::default().hash_password(...)` läuft direkt im `async move`-Block, 50–100 ms CPU pro Login. Im gesamten Workspace gibt es null `spawn_blocking`-Aufrufe. In `tokio::task::spawn_blocking` wrappen — ein Login blockiert aktuell alle Worker des Tokio-Runtime-Threads.
- [ ] **I5 — LPIS-HTTP-Cache ist implementiert, aber nie aktiviert** — `lpis-providers/src/base.rs:76` setzt `cache: None`; `create_default_registry()` (`lpis-providers/src/lib.rs:23-63`) ruft nie `.with_cache(...)`. `get_cached_or_fetch` (`:122-134`) umgeht den Cache-Zweig also immer. Jeder Import-Site löst einen HTTP-Request zu SIGPAC/BRP aus. `LpisCache` (moka) in der Registry bauen und je Provider setzen.
- [ ] **I6 — Audit-Trigger auf Hochfrequenz-Tabellen** — Migration `:1735-1759` installiert `audit_trigger_*` auf jede Tabelle mit `tenant_id`, auch auf `worker_locations`, `clock_entries`, `worker_task_statuses`, `task_data`, `weather_data`, `soil_moisture_readings`. `audit_trigger_function` schreibt pro Zeile `to_jsonb(OLD)+to_jsonb(NEW)`. Beim Location-Ping: zwei Writes plus zwei JSONB-Serialisierungen statt einem. Trigger für Hochfrequenz-Tabellen droppen; dort ist Audit fachlich über `work_logs` bereits abgedeckt.

#### I-B N+1-Muster (P1)

- [ ] **I7 — Clock-In→Clock-Out-Auflösung pro Zeile** — `postgres/clock_entry.rs:164-190` (`find_sessions`) und `:281-298` (`total_hours_worked`). Beide laden alle Clock-Ins im Zeitfenster und fragen **pro Clock-In** erneut ab. Bei 30 Tagen × 2 Events ≈ 60 sequenzielle Roundtrips. `LEFT JOIN LATERAL` auf den nächsten ClockOut; der Index `idx_clock_entries_worker_time(worker_id, timestamp)` (Migration `:1596`) deckt das bereits ab. 60 Roundtrips → 1.
- [ ] **I8 — Inventar-Bestände pro Artikel aggregiert** — `postgres/inventory_item.rs:106-140` (`find_below_minimum`) und `:158-191` (`find_balances`). Beide laden ohne Kompensation alle aktiven Items und fragen **pro Item** eine `SUM()`-Query. `LEFT JOIN` mit `GROUP BY i.id` statt Schleife; zusätzlich `CREATE INDEX idx_inv_txns_item_created ON inventory_transactions(item_id, created_at DESC)` fehlt (vorhanden sind nur tenant/type/created_at/batch/expiration, `:1586-1590`). 201 Queries → 1.
- [ ] **I9 — Tier-Behandlungen im Veterinär-Report** — `reporting-service/src/main.rs:192-209`. Pro Animal (bis `per_page: 500`, `:171`) eine `find_treatments_by_animal`-Query → 1 + 500 Roundtrips. Eine JOIN-Query.

#### I-C Fehlende Indizes (P1)

- [ ] **I10 — Zehn fehlende Indizes** — Abgleich Migration gegen Queries: `inventory_item.rs:58` `(item_id, created_at DESC)`; `order.rs:74,271` `(tenant_id, created_at DESC)` (nur `tenant_id`, `:1411`); `weather_data.rs:67` `(tenant_id, timestamp DESC)` (nur `(station_id, timestamp DESC)`, `:1426`); `tasks` `(tenant_id, status)` und `(status, scheduled_start)` (`:1420-1424`); `animal.rs:80` `(tenant_id, created_at DESC)` (keiner); `harvest_lot.rs:43` `(tenant_id, created_at DESC)` (nur `tenant_id`, `:1430`); `frost_warning.rs:123` partiell `(tenant_id, created_at DESC) WHERE is_active`; `tenant.rs:37` `(is_active, id)`; `customer.rs:235` trgm auf `company` und `customer_number` (trgm nur auf `name`,`email`, `:1628-1629`); `equipment.rs:144` `lower(label)`/`lower(code)` mit `gin_trgm_ops`. Jeder Punkt verwandelt Sort+Scan in Index-Scan, bei 50k–1M Zeilen Faktor 10–100 in der p95-Latenz.

#### I-D Payload, Pagination, Transaktionen (P1)

- [ ] **I11 — `SELECT *` auf breiten JSONB-/Geometrie-Tabellen (167 Fundstellen)** — `site.rs:104-112` und `:161-180` laden `boundary` (GEOMETRY), `center`, `plots`, `properties`, `custom_fields`, `lpis_data`, `sigpac_data`, `row_config`, `bbch_stage`. Bei 500 Sites mit Polygonen ergibt das eine Multi-MB-Payload pro Listenseite. Ebenso `tree.rs:45`, `group.rs:45`, `building.rs:52`, `livestock.rs:52`, `order.rs:74`, `worker_task_status.rs`, `task_data.rs:49`. Listen-Queries auf eine DTO-Spaltenliste reduzieren (das `SiteDb`-Muster in `site.rs` zeigt es für Details), Geometrie nur im Detail-Endpoint.
- [ ] **I12 — Queries ohne `LIMIT`** — `inventory_item.rs:98,150`; `equipment.rs:680` (`equipment_fuel_consumption`) und `:743` (`equipment_usage_log`) — komplette Historie pro Gerät ohne LIMIT und ohne Zeitfenster; `order.rs:111,239`; `livestock.rs:130`, `tree.rs:123`, `group.rs:123`, `breed.rs:74`, `variety.rs:83` (Stammdaten komplett); `frost_warning.rs:123`; `plant_protection_record.rs:170`; `api/handlers/iot.rs:130-145` lädt **alle** `iot_devices` des Tenants und filtert `site_id`/`status` in Rust, ohne Pagination in der Antwort. Durch `Pagination` (Default 20, max 500, `shared/src/lib.rs:71-85`) ersetzen, für Historien zusätzlich ein Zeitfenster.
- [ ] **I13 — Zwei Queries pro Listen-Request (`COUNT` + `SELECT`)** — Durchgängig in etwa 25 Repos (`site.rs:98+104`, `user.rs:132+140`, `equipment.rs:190+214`, `customer.rs:233+243`, `weather_data.rs:61+67`). Der `COUNT(*)` läuft mit demselben Filter und skaliert linear. Keyset-Paging statt `OFFSET`, `total` per `COUNT(*) OVER()` nur wenn wirklich benötigt. Halbiert die Roundtrips und eliminiert die OFFSET-Tiefenkosten.
- [ ] **I14 — Fehlende Transaktionen bei Multi-Write** — `order.rs:139-154` (INSERT + Audit-Log), `order.rs:170-208` (SELECT + UPDATE + Audit-Log, nicht atomar, TOCTOU auf `old_order`), `handlers/workforce.rs:446-560` (pro Order `update()` + `publish()` + `find_by_user_id()` + `work_log_repo().create()`), `handlers/orders.rs:328-337` (Folge-Orders in Schleife einzeln committet). Repo-Updates um eine `&mut Transaction`-Variante ergänzen; Batch-Operationen in `import_service.rs:56-95,408-493` ebenfalls transaktional plus `UNNEST`-Batch-Insert. Korrektheitsgewinn primär.
- [ ] **I15 — Keine Kompression** — `api/lib.rs:146-169`, Middleware-Kette `Prometheus → SecurityHeaders → Governor → CORS → Metrics`, **kein `Compress`**. Bei GeoJSON-, Excel- und SIGPAC-Antworten im MB-Bereich. `Compress::default()` ergänzen, ~70–85 % weniger Transferbytes für textbasierte Antworten.

#### I-E Blocking Work auf Async-Threads (P1)

- [ ] **I16 — Kein `spawn_blocking` im gesamten Workspace, fünf CPU-Schwerpunkte** — Null Treffer verifiziert. Betroffen: **LPIS-Parsing** (`lpis-providers/src/sigpac.rs:146-200`, `gml_to_polygon` mit tausenden `parse::<f64>()`, bei `per_page` bis 1000 Parzellen Sekunden CPU); **Excel-Reports** (`reporting-service/src/main.rs:134-166,180-213`, `rust_xlsxwriter` synchron in async); **Geometrie-Service** (`geometry-service/src/worker.rs:62-88`, Flächen- und Contains-Berechnung direkt in der NATS-Consumer-Loop, blockiert alle weiteren `geometry.request`); **Checksummen** (`backup-service/src/verification.rs:214-239`, SHA-256 über geladene Dateien); **Geodäsie im Domain-Layer** (`domain/src/entities/spatial/mod.rs`, Haversine pro Objekt in `site.rs:561-564`). Alle fünf in `spawn_blocking` wrappen, für Geometrie zusätzlich `rayon::par_iter`. Beim LPIS-Bulk-Import der wichtigste Punkt.

### Block J — Nicht verdrahteter Code und Schema-Drift (P0/P1)

Befunde aus der Analyse fehlender Implementierungen. `cargo check --workspace` läuft
sauber durch — es sind durchweg **Laufzeit-Bugs**, keine Compile-Fehler. Genau deshalb
sind sie bisher unentdeckt geblieben.

#### J-A Module und Handler ohne Registrierung (P0)

- [x] **J1 — `sigpac` und `livestock` sind deklariert, aber nie registriert** — erledigt (2026-10-01). Beide `.configure(...)` ergänzt, damit `/api/v1/sigpac/parcels` (3 Routen) und `/api/v1/livestock/animals` (7 Routen) existieren; sie waren in `openapi.rs` dokumentiert, gab es aber nicht. Die gleichzeitig zweimal registrierte `livestock_new::configure` ist auf einmaliges Vorkommen reduziert. Die beiden Livestock-Module teilen sich den Prefix `/livestock`, nutzen aber verschiedene Unterpfade (`/animals…` gegen `/`, `/{id}`, `/by-plot/…`), es gibt also keine Kollision. — `handlers/mod.rs`. Beide Module sind als `pub mod` deklariert und besitzen ein fertiges `configure()`, werden aber nie aufgerufen (verifiziert: 22 `.configure()`-Aufrufe, weder `sigpac::configure` noch `livestock::configure`). Damit existieren `/api/v1/sigpac/parcels` (3 Routen) und `/api/v1/livestock/animals` (7 Routen) **nicht**, obwohl sie in `openapi.rs:243-251` dokumentiert sind. Konsequenz: `animal_repo` ist komplett unerreichbar, und der Veterinär-Export bricht mit, weil `reporting-service/src/main.rs:175,196` das Repo direkt nutzt. Beide `.configure(...)` ergänzen.
- [x] **J2 — Drei Site-Import-Handler ohne Route, der gesamte `ImportService` ist toter Code** — erledigt (2026-10-01). `POST /sites/import`, `POST /sites/import/geojson` und `POST /sites/import/shapefile` registriert und dabei bewusst **vor** `/sites/{id}` platziert, sonst hätte das Id-Muster „import" als UUID zu parsen versucht. Damit sind 719 Zeilen `ImportService` (LPIS-Registry, GeoJSON, Geozero-Shapefile) und die passenden Wrapper in der Admin-UI (`api.rs`) erstmals erreichbar. — `handlers/sites.rs`. `import_sites` (`:238`), `import_geojson` (`:270`) und `import_shapefile` (`:302`) sind vollständig implementiert und nutzen `ImportService` mit LPIS-Registry und Geozero-Shapefile-Parsing (719 Zeilen Service-Code) — haben aber **keine Route** (verifiziert: keine `web::resource` mit `import` in `sites.rs`). Routen ergänzen; dabei statische Pfade **vor** `/sites/{id}` registrieren, sonst schluckt das `{id}`-Resource die Namen.
- [x] **J3 — `livestock_new::configure` doppelt registriert** — mit J1 behoben (2026-10-01). — siehe H7. `mod.rs:244` und `:250` rufen beide dieselbe Funktion auf.

#### J-B Schema-Drift (P0)

- [ ] **J4 — Rund 45 Spalten, die die Repos erwarten, existieren im Schema nicht** — teilweise abgearbeitet: `animals.identifier`, `animals.livestock_type` und `animals.status` sind in Migration `0000000003` ergänzt, mit Backfill aus `tag_number`/`species` und einem BEFORE-Trigger, der `livestock_type` bei jedem INSERT und UPDATE aus `species` ableitet. Offen bleiben die übrigen Tabellen, darunter `work_logs` mit komplett abweichendem Schema. Abgleich aller INSERT/UPDATE/FROM-Spaltenlisten gegen `information_schema` (nur drei `ALTER TABLE ADD COLUMN` im Bestand). Am schwersten: **`work_logs`** — das Repo braucht `date, hours_worked, overtime_hours, rest_period_hours, task_description, site_id, is_night_shift, breaks_taken` (`work_log.rs:85`), die Migration hat stattdessen `task_id, started_at, ended_at, duration_minutes, notes`. Komplettes Schema-Mismatch. Weitere: `weather_data` (`wind_direction_deg, solar_radiation_wm2, pressure_hpa, soil_temperature_c, soil_moisture_percent, leaf_wetness`), `growing_degree_days` (`base_temp_c, actual_mean_temp_c, gdd, accumulated_gdd, crop_type` — Migration hat `min_temp_c/max_temp_c/gdd_base_10/gdd_base_5`), `financial_records` (`amount, category, reference_id` vs. `amount_eur, currency, date, invoice_ref`), `olive_oil_records.grove_id` vs. `olive_grove_id`, dazu `harvest_lots.site_ids`, `pac_applications.documents_urls/eco_schemes/total_eligible_area`, `pest_risks.confidence`, `cost_centers.cost_center_type/reference_id`, `audit_logs.ip_address`, `animals.identifier/status`, `workers.contract_type/language`, `weather_stations.manufacturer/model/serial_number`, `worker_task_statuses.updated_at`, `task_data.ended_at/handoff_to_worker_id/is_session_complete`, `plant_protection_records.pre_harvest_days/re_entry_days/total_quantity/applicator_license`, `applicator_licenses.license_type`, `compliance_checklists.items`, `equipment_maintenance_log.downtime_hours/labor_hours`. Konsolidierte Drift-Migration mit `ADD COLUMN IF NOT EXISTS`.
- [ ] **J5 — `varieties` und `breeds` haben keine `tenant_id`-Spalte, ihre Repos filtern aber danach** — Migration: `varieties` hat nur `id, category, name, origin, created_at`, `breeds` nur `id, species, name, origin, created_at`. Die Repos filtern mit `tenant_id = $2` (`variety.rs:21,43,50,154,176`) — das ist ein weiterer Schema-Bug mit Datenwirkung: Varianten und Rassen sind **global statt mandantenisoliert**. Spalte ergänzen und mit FK auf `tenants` versehen.
- [x] **J6 — Naming-Bugs statt fehlender Tabellen: `animal_treatments`, `animal_grazing_records`, `water_usages`** — erledigt (2026-10-01). `animal.rs` fragte `animal_treatments` und `animal_grazing_records` ab; die Tabellen heißen `treatment_records` (Migration `:676`) und `grazing_records` (`:665`), zusätzlich schrieb das Repo `treatment_date` statt `date`. Auf die vorhandenen Tabellen umgestellt, kein neues Schema nötig. `water_usages` wurde per `ALTER TABLE water_usage RENAME TO water_usages` gelöst, damit alle Aufrufstellen gleichzeitig stimmen.

#### J-C Platzhalter-Daten und gelogene Rückmeldungen (P0)

- [x] **J7 — `delete_backup` löscht nichts, gibt aber Erfolg zurück** — `handlers/backup.rs:239-267`. Die Funktion prüft nur `get_job_status` und antwortet 200 mit „deleted", ohne ein Objekt aus dem Storage zu entfernen. Zusammen mit F2 und H3 eine von drei Stellen, an denen die Backup-API Erfolg vortäuscht.
- [ ] **J8 — `get_device_telemetry` liefert immer leer, `send_command` lügt über den Status** — `handlers/iot.rs:404-408` gibt `measurements: vec![]` mit dem Kommentar „would query a time-series database"; `:463-464` setzt `CommandStatus::Sent`, **ohne etwas zu publizieren**. `AppState` hat nur `messaging: Arc<MessagingClient>` (NATS); der `MqttClient` aus `agrocore-messaging` wird in `lib.rs:113` nicht instanziiert, obwohl `AgroCoreConfig.mqtt_broker` existiert. `UnifiedMessagingClient` aufnehmen, Status erst nach bestätigtem Publish setzen, Telemetrie in eine Tabelle mit `timestamp`-Index und Retention schreiben.
- [ ] **J9 — `fetch_weather` liefert hartcodierte Werte** — `handlers/calculation.rs:514-524` gibt `temperature_c: Some(20.5)`, `humidity: 65.0`, `pressure: 1013.25` zurück. Der geparste Provider wird in `_service_type` (`:508-512`) verworfen. Die drei echten Provider existieren bereits in `crates/weather-service/src/providers/`. An `agrocore_weather::WeatherAggregator` anbinden oder den Handler entfernen.
- [ ] **J10 — `list_lpis_providers` erfindet Base-URLs** — `handlers/settings.rs:236-246` gibt `https://{country}.example.com/wfs` zurück statt der echten `ProviderConfig::default()`-Werte. Platzhalter-Domains, die als Konfiguration aussehen.

#### J-D Toter Code und nicht aktivierbare Features (P1)

- [ ] **J11 — Zwei Domain-Traits ohne jede Implementierung** — `domain/src/repositories.rs:694` `SoilMoistureReadingRepo` und `:744` `SoilMoistureAlertRepo` (je vier Methoden): keine Postgres-Implementierung, kein `db`-Accessor, kein Handler-Routing. Die Tabellen existieren, und `weather-service/src/worker.rs:245` ruft `process_soil_misture_alerts(...)` auf — **der Alarm-Pfad läuft ins Leere**. Repos nach dem vorhandenen `repo!`-Muster nachziehen und in `PostgresDb` vorinstanziieren.
- [ ] **J12 — `mocks`- und `depreciation`-Feature sind nicht aktivierbar** — `api/Cargo.toml:72`, `domain/Cargo.toml:25`, `infrastructure/Cargo.toml:28` deklarieren `mocks`; kein Crate referenziert es. Damit ist `#[cfg_attr(feature = "mocks", automock)]` an 61 Repository-Traits wirkungslos — **es existiert kein einziger Mock**, alle Handler-Tests brauchen eine echte DB. `depreciation` ist über `#![cfg(feature = "depreciation")]` in `domain/src/lib.rs:1-2` ausgeschaltet, `domain/src/depreciation.rs` (58 Zeilen) wird nie kompiliert, und `run_monthly_amortization` (`:52`) hat null Aufrufer. Features aktivieren oder entfernen.
- [ ] **J13 — Toter Konfigurationscode** — `shared/src/config.rs:65` `mqtt_broker` und `:68` `rust_log` haben null Verwendungen; der MQTT-Client liest stattdessen hartcodierte Env-Vars. Verdrahten oder entfernen. Dazu `domain/src/repositories.rs:15` `VisibilityAwareEntity` — leerer Marker-Trait mit null Implementierungen.
- [ ] **J14 — `scheduler::stop()` und weitere Stubs** — `scheduler/src/service.rs:79` loggt „graceful shutdown not fully implemented" und tut sonst nichts: keine Cancel-Token, laufende Jobs werden nicht abgebrochen. `backup-service/src/manifest.rs:136` `get_schema_version()` gibt immer `Ok("unknown")`. `messaging/src/lib.rs:475` `try_deliver` ist `Ok(())` ohne Zustellbestätigung (deshalb ist J8 nicht sauber lösbar). `notification/types.rs:132` `health_check()` → `Ok(())` und `:144` `is_available()` → immer `true`: **die Notification-Kanal-Health-Checks sind blind**.
- [ ] **J15 — Compliance-Filter laden die ganze Tabelle und filtern in Rust** — `handlers/compliance.rs:212-234` und `:249-271` rufen `find_all(Pagination::default())` und filtern in Rust, mit eigenem Kommentar „would require a find_by_site method". Bei wachsendem Bestand bricht das Performance **und** Korrektheit, weil nur die erste Default-Seite berücksichtigt wird. `find_by_site`/`find_by_type` im Repo ergänzen.
- [ ] **J16 — Düngekosten sind ein fester Prozentsatz** — `handlers/nutrition.rs:101` setzt `cost_eur: total_amount * 0.5 // Placeholder cost`. Finanziell relevant für PAC- und Kostenstellenberichte.
- [ ] **J17 — `iot.rs` umgeht den Repo-Layer** — rohes `sqlx::query` auf `iot_devices` mit einem JSONB-Blob pro Gerät (`find_device:47-61`). Funktional, bricht aber das `Repository<T>`-Pattern und die `measure_sqlx_query!`-Instrumentierung.

#### J-E Fehlende Tests an genau den Stellen, an denen es gebrochen ist (P1)

- [ ] **J18 — Auth-Round-Trip ist untestet** — kein Test ruft `find_by_refresh_token` auf, deshalb blieb der Totalausfall unentdeckt. Test `login → refresh → logout` ergänzen.
- [x] **J19 — Kein Migrations-Schema-Assertion-Test** — teilweise erledigt (2026-10-01). Drei Tests in `crates/infrastructure/tests/database_setup_tests.rs` prüfen jetzt Tabellenexistenz, Mandantenbezug und die Lesbarkeit neu angelegter Zeilen; alle drei laufen grün gegen das PostGIS-Testimage. Der Teil, der die **Spaltenlisten** der Repos gegen `information_schema` vergleicht, fehlt weiterhin — er würde J4 und J5 automatisch finden.

**Befunde aus dem ersten erfolgreichen Fixture-Lauf (4 grün, 5 rot).** Die Fixture-Arbeit hat fünf vorbestehende Defekte erstmals sichtbar gemacht; die roten Tests sind nicht durch die Migration verursacht:

- `test_repository_tables_exist`, `test_new_domain_rows_are_tenant_scoped`, `test_postgis_extension_enabled`, `test_uuid_ossp_extension_enabled` — grün.
- `test_database_migrations_applied` — erwartet die Tabelle `spatial_properties`, die keine Migration anlegt; entweder Tabelle nachziehen oder Erwartung entfernen.
- `test_site_crud_operations` und `test_updated_at_trigger` — beide scheitern an `INSERT has more target columns than expressions`. Zwei Tests, eine Ursache: der gemeinsame Site-INSERT in der Testumgebung hat mehr Zielspalten als gebundene Werte.
- `test_tenant_creation_and_isolation` — `common/mod.rs:71` verwendet im `create_test_tenant` fest `slug = 'test-tenant'`; sobald ein zweiter Test denselben Slug anlegt, greift `tenants_slug_key`. Slug pro Aufruf uuid-suffixieren.
- `test_tenant_scoped_tables_have_tenant_id` — meldet `kelter_deliveries`, `order_sites`, `spatial_ref_sys`, `tenants`, `user_sites`. `kelter_deliveries` bestätigt **B1** unabhängig: die Tabelle hat weder `tenant_id` im Schema noch einen Filter im Repo. `user_sites` und `order_sites` sind reine Zuordnungstabellen und gehören in die Ausnahmeliste, ebenso die PostGIS-Systemtabelle `spatial_ref_sys` und `tenants` selbst. Konkret zu tun: Ausnahmeliste im Test um `order_sites`, `spatial_ref_sys`, `tenants`, `user_sites` erweitern und `kelter_deliveries` unter B1 beheben. — kein Test vergleicht Repo-INSERT-Spaltenlisten gegen `information_schema.columns`. Ein einziger solcher Test hätte J4, J5 und J6 sofort gefunden.
- [ ] **J20a — Der Test-Fixture lief nie erfolgreich** — `crates/infrastructure/tests/common/mod.rs`. `testcontainers_modules::postgres` ist fest auf `postgres:11-alpine` verdrahtet; Migration `0000000000` braucht aber PostGIS. Alle neun Integrationstests scheiterten an `extension "postgis" is not available` und waren damit nie grün. Behoben: `GenericImage::new("postgis/postgis", "16-3.4")` plus Connect-Retry. Vier Tests sind danach immer noch rot (fehlende Tabelle `spatial_properties`, falscher INSERT-Spaltenzahl, Slug-Unique-Constraint, Tabellen ohne `tenant_id`) — jetzt sind sie erstmals überhaupt sichtbar. Im Rahmen von J19 behoben.
- [ ] **J20 — Kein Route-Vollständigkeits-Test** — kein Test gleicht alle `#[utoipa::path]`-Handler gegen die registrierten Routen ab; J1 und J2 blieben dadurch unbemerkt.
- [ ] **J21 — Kein Multi-Tenant-Isolationstest** — `infrastructure/tests/database_setup_tests.rs` hat sechs Tests, alle `#[ignore]`, aber keiner prüft, dass Tenant A keine Daten von Tenant B sieht. Bei faktisch inaktivem RLS (A3) ist das der wichtigste fehlende Test überhaupt.

---

## P0 — Kritisch

### Notification-Kanäle vervollständigen

Aus Phase 8 Abschnitt 3. Der Dispatcher ist produktiv, aber die Kanalliste ist unvollständig.

- [ ] Push-Kanäle: Firebase (FCM), APNs, WebPush (`crates/messaging/src/notification/channel.rs`)
- [ ] Inbound-Webhooks für den Empfang (WhatsApp, Telegram, Email-Reply)
- [ ] Weitere E-Mail- und SMS-Provider: Postmark, Vonage, Plivo, Sms77
- [ ] Tenant-User-Preferences (welcher Kanal für wen, Ruhezeiten, Eskalation)

### Backup-Service abschließen

Aus Phase 8 Abschnitt 4. Betrifft `crates/backup-service/`.

- [ ] SFTP-Host-Key-Pinning gegen eine `known_hosts`-Datei — `check_server_key` akzeptiert aktuell jeden Schlüssel (`sftp_backend.rs`), das ist ein offener MITM-Risiko
- [ ] Cloud-Downloads speicherschonend machen — `object_store` 0.11 liefert keinen asynchronen Byte-Stream, `GetResult::bytes()` lädt das Objekt komplett. Betrifft S3, Azure und GCS; Local, SFTP und WebDAV streamen bereits
- [ ] Integrationstests gegen echte Cloud-Instanzen (S3/MinIO, Azure Blob, GCS) in CI
- [ ] Integrationstests gegen echte SFTP- und WebDAV-Server
- [ ] NATS-Progress-Events (0–100 %) durch Integrationstests absichern
- [ ] Age- und KMS-Verschlüsselung — aktuell nur AES-256-GCM (`encryption.rs`)
- [ ] Disaster-Recovery-Runbook: RTO < 15 Min für eine 50-GB-Datenbank, Single Tenant

---

## P1 — MVP

### Kunden & Verkauf

- [ ] CSA-Verwaltung: Abonnement-Boxen, Lieferplanung
- [ ] Großhandelsaufträge: Staffelpreise, Lieferplanung
- [ ] Direktverkauf: Onlineshop, Zahlungsabwicklung

### Tierhaltung

- [ ] Zucht-Records: Brunft, KI, Kalbung, Paarung
- [ ] Futteraufnahme-Tracking: Ration, Verschwendung, Futterwert
- [ ] Milchproduktions-Tracking: Tagesproduktion, Butterfett, Protein
- [ ] Bewegungsdokumentation: Geburten, Todesfälle, Käufe, Verkäufe
- [ ] Weide-Management: Flächenrotation, Ruheperioden, Tierstandort

### Finanzen

- [ ] Buchhaltungs-Integration: QuickBooks, Xero, doppelte Buchführung
- [ ] Budgetierung und Prognosen: geplant versus tatsächlich
- [ ] Feldkalkulation: Eingangs- versus Ausgangswerte
- [ ] Umsatz-Tracking pro Kultur
- [ ] Geldfluss-Management: Verbindlichkeiten und Zahlungseingänge planen
- [ ] Steuerberichtswesen: Schedule F, Abschreibungen, Düngerkosten-Abzug
- [ ] Eingangskosten-Tracking: Saatgut, Dünger, Chemikalien, Kraftstoff, QR-Scans
- [ ] Abschreibung in den Finanzbericht integrieren — die Berechnung (`depreciation.rs`) und der monatliche Timer laufen, aber `/financial/reports` wertet sie noch nicht aus

### Katalog-Import

- [ ] `scripts/import_catalog.py`: vollständige Kataloge als CSV erzeugen und nach `varieties`/`breeds` importieren. Quellen: VIVC-Rebsorten (>12k), Oliven-DB (>260), FAO-Tierrassen. Lazy-Load-Suche für die Admin-UI vorbereiten

---

## P2 — Wichtig

### Monitoring & Observability

Aus Phase 4, bisher nicht begonnen.

- [ ] Query-Dauer-Monitoring (sqlx-Middleware oder `sqlx-metrics`)
- [ ] Pool-Auslastung: aktive und idle Verbindungen
- [ ] Slow-Query-Erkennung mit Logging
- [ ] Span-Attribute: Tenant-ID, User-ID, Operationstyp
- [ ] Tracing über alle Service-Grenzen hinweg standardisieren

### Business Metrics

- [ ] Aktive Geräte pro Tenant
- [ ] Übertragene Telemetrie-Nachrichten pro Stunde
- [ ] Erfolgreich verarbeitete Import-Dateien

### Mobile First

- [ ] Offline-first Mobile-App mit Sync bei wiederhergestellter Konnektivität
- [ ] Barcode- und QR-Code-Scanner: Equipment, Inventar, Feld-ID
- [ ] GPS-Feld-Grenzen (Boundary Recording)
- [ ] Mobile Zeiterfassung: Clock-In/Clock-Out mit GPS
- [ ] Sprach-zu-Text-Notizen
- [ ] Foto-Dokumentation: Anhänge an Tasks, Probleme, Inspektionen
- [ ] Push-Benachrichtigungen: Wetterwarnungen, Task-Erinnerungen
- [ ] Feldaktivitäten-Recording in Echtzeit: Pflanzen, Spritzen, Ernten
- [ ] Ernte-Daten-Import von Combine-Harvestern
- [ ] Drohnen- und UAV-Integration: NDVI-Bilder

---

## P3 — Komfort

### Präzisionslandwirtschaft

- [ ] Variable Düngungspläne: VRA für Sämaschinen, Spritzer, Streuer
- [ ] GPS-Autosteuerung: Anbindung an Lenksysteme
- [ ] Drohnen-Spritzen-Integration: Management und Steuerung
- [ ] Automatisierte Bewässerungssteuerung über IoT-Ventile

### Nachhaltigkeit & Compliance

- [ ] Kohlenstoff-Gutschriften-Tracking: CO₂-Sequestrierung messen und berichten
- [ ] Wasser-Nutzungs-Monitoring: Bewässerungseffizienz, regulatorische Compliance
- [ ] Chemische-Anwendungs-Logs: REI, beschränkte Verwendung
- [ ] Biologische Zertifizierung: Eingangs-Tracking, Pufferzonen, Inspektionen
- [ ] Nachhaltigkeits-Metriken: Bodengesundheit, Biodiversität, Düngerreduktion

### Business-Features

- [ ] Multi-Betriebs-Management: Haltereien, Pachtverträge, Mieter
- [ ] Vertragslandwirtschaft: Erzeuger-Verträge, Qualitätsprämien
- [ ] Lohnarbeits-Management: Gehälter, Zertifizierungen, Planung
- [ ] Equipment-Sharing: Vermietungsmarktplatz zwischen Betrieben
- [ ] Versicherungs-Integration: Schadensdokumentation, Risikobewertung

---

## P4 — Zukunft

### KI-Analytics (Modul 17)

- [ ] Anforderungen für Ertragsprognosen definieren
- [ ] Anforderungen für Krankheits- und Schädlingsfrühwarnsysteme definieren
- [ ] Anforderungen für Bewässerungs-, Düngungs-, KPI-, Satelliten-Monitoring- und generatives Reporting definieren
- [ ] Computer Vision: Pflanzenkrankheiten, Unkrauterkennung
- [ ] KI-Beratungsassistent: Chat-Interface für agronomische Fragen
- [ ] Generative KI für die betriebliche Planung
- [ ] Implementierung erst nach Stabilisierung der Produktionsmodule und der Sync-Grundlagen

### KI & Robotik

- [ ] Roboter-Krähen: autonome Steuerung und Monitoring
- [ ] Autonome Geräte: Flotten-Management für selbstfahrende Traktoren

### Fortgeschrittene Technologien

- [ ] Augmented Reality: Feld-Daten-Overlay auf der Live-Kamera
- [ ] Digital Twin: virtuelles Betriebsmodell für Szenario-Planung
- [ ] Blockchain-Rückverfolgbarkeit: LieferkettentransparenzW