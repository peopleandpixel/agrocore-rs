# Agrocore-RS Open Tasks

Last updated: 2026-10-01

Open work only. Completed phases and modules have been removed; the history lives in the
CHANGELOG. Ordered by priority, within a priority by dependency.

**Task 0 takes precedence over all feature topics.** It contains six immediately exploitable
security holes plus the prerequisites for configuration and backend functions to be operable
through the AdminUI at all. The P0–P4 feature lists count as work that can only sensibly be
started after Task 0.

Legend: **P0** blocks operation · **P1** MVP · **P2** important · **P3** convenience ·
**P4** future.

---

## Task 0 — Security, completeness and performance audit (URGENT)

Code audit from 2026-10-01 across the entire workspace (154 API routes, 33 handler modules,
17 crates). Findings were verified against the code, not merely sighted.

**Target state:** the system must be fully functional in normal mode **and** in demo mode:
all configurations settable via the AdminUI, all backend functions controllable via the
AdminUI.

**Order:** first correctness (I0, J1, J2, D1), then security (A–E), then persistence (F),
then missing backend functions (G), then the AdminUI (H), then performance (I3–I16).

**Overarching pattern:** for almost every problem the infrastructure exists but the wiring is
missing. `is_revoked()` is defined and never called. `add_treatment` exists, the route behind
it is not registered. `stock_in` exists, never called. `find_by_refresh_token` exists, returns
`None`. The `mocks` feature exists, is never enabled. These are wiring defects, not feature
defects — and tend to be considerably cheaper to fix than writing new functionality.

**Status:** all four analyses have been evaluated — blocks A–J, 149 open items. Every finding
was verified against the code or against the schema, not merely sighted. `cargo check
--workspace` passes cleanly: these are **runtime bugs** throughout, not compile errors.

**The three clusters to fix first:**

1. **I1 — eight tables are missing from the schema.** Seven repos are dead at runtime. `spatial_objects` is queried by every GPS ping, the error is swallowed via `unwrap_or_default()` — location assignment never works and never surfaces.
2. **A1 — privilege escalation** via `PUT /users/{own_id}`.
3. **J1/J2 — unregistered handlers.** `/sigpac/parcels`, `/livestock/animals` and all three site-import endpoints do not exist; the 719-line `ImportService` is dead code.

**Note on I1:** the schema drift was the most severe single finding of the audit.
I1, I2 and J6 are done (migration `0000000003_missing_domain_tables.sql`). Six repos are
functional again, the GPS ping returns data again, and the errors are no longer swallowed.

**Further finding from I1:** in `tree.rs`, `group.rs`, `building.rs` and `livestock.rs`,
`tenant_id` was missing from the INSERT although all SELECTs filter on it afterwards — newly
created records would have been unfindable. Fixed.

### Block A — Immediately exploitable, existential consequences (P0)

- [x] **A1 — Privilege escalation: any user can make themselves admin** — done (2026-10-01). `update_user` now checks `if dto.0.roles.is_some() || dto.0.is_active.is_some() { auth.require_admin()?; }` — role changes and active status are admin-only, regardless of whether the own account is affected. Plus a new `PUT /api/v1/users/me` with `UpdateOwnProfileDto`, which only accepts `firstname`, `lastname`, `password`, `language`, `color`; all other domain fields are explicitly set to `None` so the repository leaves the columns untouched. The route is registered **before** `/users/{id}`, otherwise `{id}` would swallow the path "me". `UpdateUserDto.password` now requires `min = 12` instead of the 8 from `CreateUserDto` — via the old gap an existing password could be set to an empty string; this is fixed together with A1 and D3. 9 regression tests in `crates/api/tests/privilege_escalation_tests.rs`. — `handlers/users.rs:153-157`. `if let Err(e) = auth.require_admin() && auth.0.user_id != user_id` cancels the admin check as soon as the own ID is addressed. The body contains `roles`, and `postgres/user.rs:318` binds it unchecked. Attack: `PUT /api/v1/users/{own_id}` with `{"roles":["Admin"]}` → full admin access. Role changes and `is_active` strictly admin-only; own profile via a `/me` endpoint with a field whitelist (`firstname`, `lastname`, `password`, `language`).
- [x] **A2 — Demo routes delete real tenants, unauthenticated** — done (2026-10-01). Two independent barriers: `require_demo_access()` in `handlers/demo.rs` checks `AppState.demo_endpoints_enabled` (from `ALLOW_DEMO_ENDPOINTS`, default **off**) **and** `auth.require_admin()`. All three endpoints (`/seed`, `/reset`, `/summary`) now take an `AuthExtractor`; previously not one of them did, not even `/summary`. The OpenAPI declarations now carry `security(("bearer_auth"))` as well as 401/403, both of which were previously missing. Additionally the hardcoded `b"demo123"` was removed: the password comes from `DEMO_ADMIN_PASSWORD` with default `demo1234-agrocore`, and is logged when the variable is missing. See G2 for the password unification. 6 regression tests in `crates/api/tests/demo_endpoint_auth_tests.rs`. — `handlers/demo.rs:19-22,37,559`. `/seed`, `/reset`, `/summary` without `AuthExtractor`. `reset` forces `reset=true` and runs `DELETE FROM tenants WHERE id = $1` with cascade; the tenant slug comes from the request body. Additionally hardcoded admin password `demo123` (`:117`). Routes behind `#[cfg(feature = "demo")]` + `require_admin()` + env gate `ALLOW_DEMO_ENDPOINTS`; remove the password.
- [x] **Pin integration: tenant pin in all database paths** — done (2026-10-02). A3 had armed the policies, but nothing set `app.current_tenant_id` yet. Without a pin all repos return zero rows. Fixed:

  **`crates/infrastructure/src/postgres/tenant_pool.rs`** — `TenantPool` pins before every query:
  - `set_config('app.current_tenant_id', ...)` and `set_config('app.is_superadmin','false')` on the **same** connection that executes the query.
  - implements sqlx `Executor`, so 317 call sites in 47 repos stay unchanged and the pin cannot be forgotten.
  - `begin()` sends an explicit `BEGIN` before `set_config(..., true)`. `SET LOCAL` outside a transaction block is a no-op — the other order looks like it works and then resets the pin on the first statement.
  - `unscoped()` uses the nil UUID: matches no tenant and therefore denies everything. Fail-closed for bootstrap work.
  - 7 tests in `tests/tenant_pin_tests.rs`. The decisive one: `checkout_does_not_inherit_previous_tenant` — a reused pool connection must not inherit a previous request's tenant.

  **Bootstrap paths deliberately unpinned**, with the reasoning in the code: `system/setup` and `demo/seed` create the first tenant and so cannot pin anything. `demo/summary` determines the tenant unpinned first and then pins to its ID. `demo/seed` switches its transaction over to the pin right after the tenant is created.

  **One auth path needs an exception because it does not yet know the pin:** login reads the tenant *from* the user row. `users_select` requires `tenant_id = get_current_tenant_id()`, which does not exist there. The role `agrocore_auth` (NOLOGIN, SELECT on `users` + `user_sites`, policy only for this role) solves it. Verified: under `agrocore_app` the query still returns 0 rows, so the exception is bounded.

  **Making it effective exposed four further real bugs**, all verified against a freshly migrated database:
  - **Login completely broken.** Missing grants for 10 tables created by the RLS migration; `user_sites` is needed by the login join, so every login failed with `permission denied`.
  - **Refresh-token write had no effect.** `update_refresh_token` wrote unpinned, `users_update` requires the pin → UPDATE hit 0 rows → the handler honestly reported `false`. This had previously been overlooked as a silent failure.
  - **`#[sqlx(json)]` on `Option<T>` is wrong.** sqlx knows `json` (produces `Json<T>`, non-null) and `json(nullable)` (produces `Option<Json<T>>`). 29 fields in 10 files were annotated incorrectly. Cause of `unexpected null; try decoding as an Option`.
  - **Schema drift on `orders`:** `order_type` was `VARCHAR` instead of JSONB, `planned_date`/`deadline_date` were `DATE` against `DateTime`, and `started_at`/`completed_at` did not exist in the table at all. `GET /api/v1/orders` was therefore unreachable. 5 conformance tests in `tests/order_schema_tests.rs`.
  - **37 `NUMERIC` columns against `f64` in Rust.** Every entity touching one of them failed to decode — `GET /api/v1/customers` at `vat_rate`. Normalized to `DOUBLE PRECISION`, iteratively rather than by list, so the gap does not come back.

  **Demo seed was substantively wrong** and would have kept producing 500s after the type fixes: `seeding` and `fertilizing` do not exist as `OrderType` (correct: `soil_work`, `fertilization`), and `{"mode":"Manual"}` wrote PascalCase where snake_case is expected.

  **NATS was a hard startup blocker.** A missing broker aborted the entire API start, even though the publishers did `let _ = …publish()` anyway. Messaging is now optional; `MESSAGING_REQUIRED=1` enforces it when it is deployment-critical. Reports still need the broker and say so explicitly.

  **Verified against a fresh database, all migrations from zero, demo seed loaded:**
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

- [x] **A3 — RLS exists, but had no effect** — done (2026-10-01). Three causes, all fixed:

  **1. The connection role was a superuser with BYPASSRLS.** Measured on this installation: `agrocore` has `rolsuper = true` **and** `rolbypassrls = true`. PostgreSQL exempts such roles from RLS **unconditionally** — `FORCE ROW LEVEL SECURITY` changes nothing. As long as the application connects as that role, 190 policies are decoration. Migration `0000000004_force_rls.sql` introduces `agrocore_app` (NOSUPERUSER, NOBYPASSRLS, NOLOGIN) and the pool switches to it via `after_connect` with `SET ROLE agrocore_app`.

  **2. `FORCE ROW LEVEL SECURITY` was missing** (0 hits). The migration sets it on all 62 tables with RLS — verified: 62 tables, all with `relforcerowsecurity`.

  **3. `app.current_tenant_id` was set nowhere** (0 hits in the Rust code). `get_current_tenant_id()` returned NULL, every policy compared `tenant_id = NULL`. I checked the policies empirically: with a pin on tenant A the role sees only A's data, tenant B only B's, an unknown UUID and a broken value yield 0 rows.

  **Two real bugs found along the way, only visible once it actually took effect:**
  - `sigpac_parcels` had RLS enabled but **no policy**. With FORCE that would have denied every row to everyone.
  - `tenants` had policies for SELECT/UPDATE/DELETE but **none for INSERT**. With FORCE, `POST /api/v1/system/setup` — the endpoint that creates the first tenant — would have failed for every installation. Added policy `tenants_insert ... WITH CHECK (true)`: tenant creation is a setup action that happens before a tenant exists, so it is not tenant-filtered.
  - 6 tables with a `tenant_id` column had no RLS at all; added for all except `tenants`.

  **Switched via `AGROCORE_RLS_ENABLED`**, not per release: activating the policies before the pin would make all repos return zero rows. The switch is now a configuration change, not a coordinated release. 5 tests in `crates/infrastructure/tests/rls_tests.rs` demonstrate the effect. — Init migration + workspace. 190 policies, 50× `ENABLE ROW LEVEL SECURITY`, but `app.current_tenant_id` is set nowhere (0 hits in the Rust code) and `FORCE ROW LEVEL SECURITY` is missing (0 hits). Connecting as the table owner bypasses RLS. The entire tenant isolation therefore depends 100 % on the repo queries. Set `SET LOCAL app.current_tenant_id` per transaction, add `FORCE ROW LEVEL SECURITY`, add a dedicated app role without `LOGIN`. The `agrocore_app` role is currently only granted for views and is useless for isolation.
- [x] **A4 — JWT revocation is never checked, logout is ineffective** — done (2026-10-01). After a successful `decode`, `AuthExtractor` checks the `jti` against `AppState.token_revocation.is_revoked(...)` and rejects with 401 "Token revoked". This required switching the `FromRequest` future from `Ready` to a `Pin<Box<dyn Future>>`, because the revocation list is asynchronous (Redis or in-memory); `from_request` clones the headers and the state handle so the future borrows nothing. Previously only `revoke()` was called, never `is_revoked()` — a stolen token stayed valid until expiry, and the UI did not even send the logout request (H6). — `middleware.rs:90`. `is_revoked()` is defined but has zero call sites; only `revoke()` is used. Logout and password change revoke nothing — a stolen token remains valid for 30 minutes. Check in `AuthExtractor::from_request` after a successful `decode`.
- [x] **A5 — JWT secret falls back to `"dev-secret"`** — done (2026-10-01). `run_server` now aborts with an `io::Error` before the server listens. `validate_jwt_secret()` no longer returns just `bool`, but `Result<(), JwtSecretError>` with two variants: `DevSecret` (unset) and `TooShort { length, minimum }` — the remedy differs, so the message must too. In addition to the previous check against the literal there is now a minimum length of 32 characters (`JWT_SECRET_MIN_LENGTH`, the width of a SHA-256 digest as the usual lower bound for a symmetric HMAC key). The development escape hatch `ALLOW_DEV_SECRET=1` only softens the default-secret check, never the length check. 5 tests in `crates/api/tests/jwt_secret_tests.rs`. — `shared/config.rs:105,132-134,277`. `validate_jwt_secret()` exists but has zero callers. Production without `JWT_SECRET` means: anyone can forge HS256 admin tokens and put themselves into arbitrary tenants. Hard-abort in `main.rs` before `run_server`, enforce length ≥ 32 bytes (`ALLOW_DEV_SECRET=1` as opt-out for development).

### Block B — Tenant breakage and data loss (P0)

- [x] **B1 — IDOR on `kelter_deliveries`** — done (2026-10-01). **Correction to the audit:** the table did exist, in `0000000000` at line 875 — it simply had no `tenant_id` column, and the RLS policies of the init schema scope over `vineyard_id IN (SELECT id FROM vineyards WHERE tenant_id = ...)`, which the repository bypasses entirely. Migration `0000000003` therefore adds a `tenant_id` column via `ALTER TABLE` instead of recreating the table, with a backfill from the associated vineyard. An orphaned row without a vineyard's tenant aborts the migration with a clear message instead of silently assigning it to an arbitrary tenant. All seven repository methods now filter `WHERE tenant_id = $n`; `KelterDelivery` has the field in the domain model. **Verified** with four integration tests in `crates/infrastructure/tests/tenant_isolation_tests.rs` and directly against the database: tenant B sees only its own rows in `find_all`, `find_by_id` with a correct UUID returns nothing, a cross-tenant `DELETE` affects 0 rows, `find_by_vineyard` stays scoped. `test_tenant_scoped_tables_have_tenant_id`: `kelter_deliveries` has neither a `tenant_id` column in the schema nor a filter in the repository — `postgres/kelter_delivery.rs:18,37,43,73,148,169`. All seven methods take `tid: TenantId` and use it in not a single query; `find_all` returns globally across all tenants. The table exists in no migration, the queries would fail at runtime. Add `AND tenant_id = $n` to all queries; create the migration with `tenant_id NOT NULL` + FK.
- [x] **B2 — Restore overwrites a database named by the client** — `handlers/backup.rs:218`. `target_database` comes unchecked from the request, `RestoreRequest` is not validated (unlike `CreateBackupRequest`). Done (2026-10-03): the field is gone from the DTO, so it is absent from the OpenAPI schema and cannot be sent. `BackupService::restore` and `PgDump::restore_from_storage` no longer take a database name and always use the configured `DATABASE_URL`. `pg_restore` runs with `--clean`, which empties the destination before writing, so a caller-chosen target was a data-destruction primitive. Verification restores into a throwaway database it created itself — the one legitimate case — and that is now the separately named `restore_into_named_database`, so the narrow purpose is visible at the call site. The handler also rejects a `backup_id` in the body that disagrees with the path.
- [x] **B3 — `delete_device` does not check tenant membership** — `handlers/iot.rs:345-361`. The condition `device.tenant_id != auth.tenant_id && !auth.is_admin()` is always `false` after the preceding `require_admin()`; the `DELETE` itself has no tenant condition. A tenant admin can delete devices of other tenants. Done (2026-10-03). The root cause was wider than this one handler: `is_admin()` was treated as a superadmin flag in six places, and every guard shaped `x != auth.0.tenant_id && !auth.is_admin()` was constant-false, because it was evaluated after a `require_*` call. All six are gone. The tenant predicate now lives in the statements — `DELETE FROM iot_devices WHERE device_id = $1 AND tenant_id = $2`. The same reasoning fixed two more holes in the same file: `create_device` accepted a foreign `tenant_id`, and `list_devices` honoured a `tenant_id` query parameter, so a tenant admin could both write into and read another tenant's device registry. `is_admin()` now appears nowhere in `handlers/iot.rs`, and a test forbids the pattern from returning.

### Block C — Information leak and abusability (P1)

- [ ] **C1 — DB error messages with schema details go to the client** — `api/error.rs:103-106`, `postgres/error_mapper.rs:9-11`. `self.0.to_string()` for all error types including `Database`; Postgres messages contain table, constraint and column names. Plus explicitly passed-through details: `backup.rs:112,220`, `settings.rs:198`, `auth.rs:60,113,124,157,195,216`, `reporting.rs:66,103,136,173`, `system.rs:121`. On 500 emit a generic message plus a correlation ID and log only server-side; map `map_db_error` to constraint codes instead of messages.
- [ ] **C2 — `/metrics/db` and `/metrics/business` without authentication** — `api/lib.rs:158-162`, registered globally, without `AuthExtractor`. Contains query timings, pool statistics and record counts per tenant. Own governor scope plus `require_admin()`, or bind to a separate port/mesh network.
- [ ] **C3 — No payload limit, DoS via the import** — `api/main.rs`. No `JsonConfig`/payload limit set; the GeoJSON/shapefile import takes unbounded bodies into memory.
- [ ] **C4 — CORS silently falls back to `permissive`** — `api/lib.rs:55-75`. Without `CORS_ALLOWED_ORIGINS` `Cors::permissive()` applies, in the other branch `allow_any_header` with `supports_credentials`. Only a `warn!` as protection. Hard-fail in production instead of setting `permissive`.

### Block D — Missing authentication mechanism (P1)

- [x] **D1 — Refresh-token mechanism completely dead, three real stubs** — done (2026-10-01). The three methods in `postgres/user.rs` are implemented: `find_by_refresh_token` filters expiry and `is_active` already in SQL, so an expired token is indistinguishable from an unknown one; `invalidate_refresh_token` sets both columns to NULL and reports via `WHERE refresh_token IS NOT NULL` whether something was really revoked; `update_refresh_token` writes token plus expiry plus `updated_at`. In `auth.rs`, login and refresh now check the return value — previously only `map_err` was checked, an `Ok(false)` would have slipped through and the client would have received a 200 with a token that was never in the database. The refresh rotates the token, which makes reuse detectable. — `postgres/user.rs:402-420`, called from `handlers/auth.rs:51-58` and `:122-124`. `find_by_refresh_token` returns `Ok(None)` (`:405`, with the comment "Simplified - not fully implemented"), `invalidate_refresh_token` and `update_refresh_token` return `Ok(false)` each (`:410`, `:419`). The columns `refresh_token` and `refresh_token_expires_at` exist (migration `:301-302`) and are written correctly in `authenticate()` (`:372`). **The bug stays silent:** `auth.rs:60` only checks via `map_err`, but the stub returns `Ok(false)` — no error. Login therefore answers 200 and hands the client a refresh token that is never in the database; `/auth/refresh` then always returns 401 (`:97`), `/auth/logout` (`:163`) revokes nothing, because `Ok(false)` is not an error. Add three real queries (`WHERE refresh_token = $1 AND refresh_token_expires_at > NOW() AND is_active = true`, invalidation to `NULL`, update with token and expiry) **and** explicitly check the three return values in `auth.rs:58-60`/`:122-124`/`:163`. Hash an opaque 32-byte random value (SHA-256), add rotation and reuse detection.
- [ ] **D2 — Impersonation is dead and potentially too far** — `handlers/auth.rs:177-184`. The role comparison checks case-sensitively for `"admin"`/`"superadmin"`, but JWT roles are generated as `"Admin"` → nobody can impersonate. If the comparison were case-insensitive, three problems would remain: the impersonation token does not land in the revocation list, there is no audit log entry, the impersonation JWT carries no `impersonator_id` claim. Deliberately disable (`ALLOW_IMPERSONATION` gate), require an audit log.
- [x] **D3 — Password update without a minimum length** — done together with A1 (2026-10-01). `UpdateUserDto.password` now has `#[validate(length(min = 12, max = 128))]`; `UpdateOwnProfileDto.password` likewise. `CreateUserDto` stays at 8, in order not to break existing accounts. Tests for this in `privilege_escalation_tests.rs`. — `dto/user.rs:96`. `password: Option<String>` without `#[validate(...)]`, while `CreateUserDto:67` sets `min = 8`. Exploitable via A1, meaning the password can be set to an empty string. `length(min = 12, max = 128)` plus rehash on login.

### Block E — Missing validation (P1)

- [ ] **E1 — 132 POST/PUT routes, only 36 `.validate()` calls** — without any `validate()`: `agriculture.rs` (8), `weather.rs` (10), `workforce.rs` (9), `harvest.rs` (8), `compliance.rs` (7), `finance.rs` (6), `water.rs` (6), `livestock.rs` (4), `breed/building/group/tree/variety/livestock_new.rs` (2 each), `nutrition.rs` (2). `validator::Validate` with length, range and enum constraints on all command DTOs; `require_manager()` as a minimum on all writing routes.
- [ ] **E2 — `per_page` without a cap, memory DoS** — `shared` pagination is passed through without an upper bound (`postgres/user.rs:128`, `equipment.rs:128`); only `sigpac.rs:92` limits with `.min(200)`. Cap in the deserializer at 200 maximum, limit `page` as well.

### Block F — Configuration is not persistable (P0, prerequisite for the target state)

Without these items the target state "all configurations settable via the AdminUI" is not
reachable. There is currently **no place in the schema** where configuration could be stored.

- [x] **F1 — No settings/config table anywhere in the schema** — `migrations/*.sql`. The only hit is `soil_moisture_configs`; there is no `system_settings`, `tenant_settings` or general `config` table. Create a migration: key, value (JSONB), tenant reference, `updated_at`, `updated_by`; plan for versioning and defaults.
- [x] **F2 — Backup configuration is hardcoded, the update stores nothing** — `handlers/backup.rs:41-70`. `get_backup_config` returns fixed values (`schedule_db: "0 2 * * *"`, `enabled: true`, `targets_count: 0`). `update_backup_config` contains `// TODO: Persist config changes`, but answers with success — the admin believes it was saved. Switch to `system_settings`; really persist targets, schedule, retention and verification, and load them at startup.
- [x] **F3 — Settings API only knows LPIS** — `handlers/settings.rs:34-44`. Only `/settings/lpis` (GET/PUT) and `/settings/lpis/providers` are registered. The rest of the backend has no configuration endpoints. Add resources for backup, scheduler, notifications, weather, LPIS cache, tenant, security and appearance.
- [x] **F4 — LPIS-only settings write to a TOML file in the filesystem** — `handlers/settings.rs:15-32,196-198`. `find_config_file()` searches relatively upwards from `current_dir`; `update_lpis_settings` writes to `config/lpis-providers.toml`. This does not work containerized and is not tenant-capable. Move it into the database; the file path and config errors must not go to the client (see C1).
- [x] **F5 — Debug output in production code** — `handlers/settings.rs:77,82,89,104`. Four times `eprintln!("DEBUG HANDLER: ...")` with file path and config details; goes to stdout on every settings call. Remove, log structured instead.
- [ ] **F6 — LPIS provider credentials live in a versioned TOML file** — provider URLs and credentials are in `config/lpis-providers.toml` instead of tenant-capable in the DB. Treat as secrets, do not write credentials into a versioned file.

### Block G — Backend functions missing (P0, not only in the UI)

The AdminUI cannot offer these functions because they do not exist in the backend.

- [ ] **G1 — No update for the core entities** — `handlers/{sites,equipment,orders,users,inventory,tasks}.rs`. `web::put()` and `web::patch()` occur **zero times** in the entire backend; there is only create and delete. Sites, equipment, orders, tasks and inventory are not editable that way. Add PUT routes plus repo `update` with the same tenant filter as `create`.
- [x] **G2 — The two demo seed paths produce different passwords** — done (2026-10-01) in the course of A2. One source now defines the password: `DEMO_ADMIN_PASSWORD`, default `demo1234-agrocore` (17 characters, meets the minimum of 12 in force since v0.25.0). All three places read from it or use the same value: `handlers/demo.rs` (`DEMO_DEFAULT_PASSWORD`), `scripts/demo_seed.sql` (Argon2id hash regenerated) and `scripts/dev.sh` (the display uses the variable instead of a literal). Empirically verified: the hash accepts `demo1234-agrocore` and rejects `demo1234` as well as `demo123`. — `scripts/demo_seed.sql` stores an Argon2id hash for `admin@demo.local` that verifies with `demo1234` (empirically checked against `argon2 0.6`/`password-hash 0.6`); `handlers/demo.rs:117` hashes `b"demo123"` instead, and `scripts/dev.sh:471` displays `demo1234`. Anyone going through the API seed cannot log in with the displayed password and vice versa. Define one source and derive both paths from it.
- [ ] **G3 — Demo mode is not a mode but a one-time seed** — `DEMO_MODE` occurs **nowhere** in the entire Rust code, neither in the API nor in the AdminUI. The mode exists only as a shell seed in `scripts/dev.sh` and `scripts/demo_seed.sql`. Wire it through consistently: flag in `AppState`, guard middleware, divergent behavior (read-only for production data, resets allowed), display in the UI.
- [ ] **G4 — Remove dead repository stubs** — see D1. Additionally remove the refresh-token stubs from the `UserRepository` trait so dead signatures are not preserved.

### Block H — AdminUI does not reach large parts of the backend (P1)

33 backend areas are not addressed by the UI at all. Verified by comparing all 154 routes
against the 102 paths used by the UI.

- [x] **H1 — The settings page is pure display** — `admin-ui/src/components/settings.rs` (395 lines). Zero `Input`, zero `Checkbox`, zero `Select`, zero `on_input`, zero API write calls. Not a single configuration is editable. Build forms against F3.
- [ ] **H2 — No update functions in the UI** — `admin-ui/src/api.rs`, 46 write functions. Only `user` and `worker` have full CRUD; all other entities only create and delete. Even where the backend offers PUT, it is not called. Add the function set to G1 and add edit forms to the detail pages.
- [x] **H3 — Backups do not exist in the UI** — `handlers/backup.rs` offers config, list, detail, status, restore and delete; the UI calls none of it. Build a backup page: configuration, manual backup run, list with status and progress, restore including `dry_run`, error log. Depends on F2, otherwise the configuration is not persistable.
- [ ] **H4 — Five complete pages are dummies** — `admin-ui/src/components/`. `groups.rs` (23 lines), `trees.rs` (19 lines), `livestock.rs` (19 lines) and `buildings.rs` (34 lines) have forms whose submit handlers only execute `let _ = (label.get(), ...)` — nothing goes to the backend. **Worse: `buildings.rs:21` shows a success message "Building created" without any HTTP call**, so the user believes it was saved. `plot_subentity.rs:11-14` is a hardcoded example table ("Herd 1", "Goats", "Cork oak"). The server-side CRUD counterparts `/groups`, `/trees`, `/buildings`, `/livestock` are finished and never used. Wire up or remove all five pages; a faked success message must not remain.
- [ ] **H5 — Nine API wrappers are dead** — `admin-ui/src/api.rs`. Defined but without any caller in `components/`: `stock_in`, `stock_out`, `transfer_inventory`, `adjust_inventory` (the entire stock logic), `update_inventory_item`, `update_worker`, `refresh_token`, `logout`, `fetch_health`. Stock can neither be replenished, withdrawn, transferred nor corrected that way.
- [x] **H6 — The logout button sends no request** — fixed together with A4 (2026-10-01). The server-side part (revocation check in the extractor) is thus complete; the client-side call of `api::logout()` from `lib.rs:343-347` is still open and is noted as such in H5. — `admin-ui/src/lib.rs:343-347`. It only calls `api::clear_auth_token()` and `clear_user_role()` in the browser and reloads; `/api/v1/auth/logout` is never called. Together with A4 the JWT remains valid server-side — after logout the token can still be reused. Switch to `api::logout()`.
- [ ] **H7 — `livestock_new::configure` is registered twice, `livestock::configure` not at all** — `api/handlers/mod.rs:244` and `:250` both call `livestock_new::configure`; `livestock::configure` occurs nowhere. Thus the routes from `livestock.rs` — among them `/livestock/animals/{id}/treatments` (`:212`) and `/grazing` (`:213`) — are **unreachable**, even though the UI has the wrappers `add_treatment` and the grazing calls. Additionally `/nutrition/*` is registered twice (`calculation.rs:34` and `nutrition.rs:12`). Consolidate both and check which scope wins.
- [ ] **H8 — The settings page fakes operability** — `admin-ui/src/components/settings.rs`. The LPIS section (`:302-354`) shows eight country cards with `<input>` **without `prop:value`, without `on:input` and without a signal** (`:325,329,333,337,342`); the button (`:351`) has **no `on:click`**. It looks like a form but is a dummy. The timezone (`:293-296`) has a `<select>` without `on:change`. System status (`:361-373`) are static badges, the version is hardcoded as `"v0.4.2-stable"` (`:372`) although the project is at `0.23.0`. The company profile (`:53-66`) lives exclusively in `localStorage` (`api.rs:133-140`) — per device, not tenant-capable, lost on a tenant switch. `fetch_lpis_providers` (`api.rs:2090`) exists but is never called.
- [ ] **H9 — Four large feature blocks are finished server-side and absent client-side** — for these areas the routes exist completely, the UI has nothing: **Backup** (12 routes, `backup.rs`), **Water** (12, `water.rs`), **Harvest** (16, `harvest.rs`), **Agriculture special** (16, `agriculture.rs` with `olive-groves`, `olive-oil-records`, `vineyards`, `kelter-deliveries`). Plus IoT (`iot.rs`, `send_device_command` wrapper present) and 11 of 13 `/calculate/*` endpoints. That is around 80 routes without any user interface.
- [ ] **H10 — Workforce task progress is invisible** — all 5 routes under `/workforce/tasks/{id}/status*` are not called, although six wrappers exist in `api.rs:1351-1409`. Task progress, aggregation and worker status are therefore not viewable via the UI; `/workforce/logs` and `/workforce/locations` are likewise completely unused.
- [ ] **H11 — Stock cannot be booked** — all four booking paths `/inventory/stock-in`, `stock-out`, `transfer`, `adjust` are not called (wrappers present, `api.rs:1697-1710`). Without them no stock can be changed. Additionally `fetch_all_inventory_transactions` (`api.rs:2126`), the transactions per item (`api.rs:1671`) and `POST /inventory/locations` are unused.
- [ ] **H12 — Compliance and weather areas unused** — `/compliance/fertilizer` completely, `/compliance/applicator-licenses` completely, `/compliance/plant-protection` read-only only; `/weather/frost-warnings` (+ `/active`), `/weather/gdd`, `/gdd/accumulated`, `/pest-risks` completely; `create_weather_station` (`api.rs:1086`) and `create_weather_data` (`api.rs:1105`) dead. No station management.
- [ ] **H13 — Further unused areas** — `profitability`, `seasons`, `quotas`, `cold-chain`, `deliveries`, `olive-groves`, `vineyards`, `pest-risks`, `phenology`, `harvest`, `breed`, `variety`, `building`, `lots`, `water-rate`, `workflow`, `groups`, `logs`, `config`, `demo`, `material`, `stations`, `sources`, `data`, `summary`, `usage`, `difficulty-surcharge`, `tree-crown-volume`, `nitrogen-demand`, `agriculture`. For each, check: deliberately hidden (document) or add the missing page.

### Block I — Performance (P0/P1)

Audit from 2026-10-01, indexes compared against the queries. Effect figures are expectations
from query-shape and index analysis, not measured.

#### I-0 Correctness defect before performance: eight tables missing from the schema (P0)

Eight tables are queried by repos but exist in **no** migration (52 queries affected). The
affected functions are dead at runtime:

| Table | Queries | Repo |
|---|---|---|
| `spatial_objects` | 5 | `site.rs` |
| `buildings` | 8 | `building.rs` |
| `trees` | 9 | `tree.rs` |
| `groups` | 9 | `group.rs` |
| `livestock` | 9 | `livestock.rs` |
| `water_usages` | 9 | `water_usage.rs` |
| `animal_treatments` | 2 | `animal.rs` |
| `animal_grazing_records` | 1 | `animal.rs` |

- [x] **I1 — Write migrations for the eight missing tables** — done in `migrations/0000000003_missing_domain_tables.sql` (2026-10-01). **Additionally found and fixed along the way:** the INSERTs in `tree.rs`, `group.rs`, `building.rs` and `livestock.rs` did not bind `tenant_id`, although all SELECTs filter on it afterwards — a newly created record would have been unfindable. The migration has been run three times in a row on the same database (idempotency confirmed), all four migrations run through in order in a fresh database, and the demo seed runs through afterwards. — The repos are implemented, the schema is not. The error is additionally swallowed: `api/handlers/workforce.rs:353,364` calls `spatial_object_repo().find_containing_point(...)` with `.unwrap_or_default()`. Every GPS ping of a worker therefore hits a non-existent table twice, the result is always empty, and the location-to-field assignment **never works, without ever surfacing**. First write the migrations, then I8 (the N+1 is there) — every optimization built on that is ineffective before.
- [x] **I2 — Stop swallowing errors** — done in `handlers/workforce.rs` (2026-10-01). The two `.unwrap_or_default()` on `spatial_object_repo().find_containing_point(...)` are replaced by `match` with `warn!`: tenant, coordinates and error text are logged, the location ping continues, but the error is no longer invisible. — Replace all `.unwrap_or_default()` on repo calls in `workforce.rs` and similar places with logging including tenant and object context; an empty result must not be indistinguishable from an error.

#### I-A Immediate, high impact, low effort (P0)

- [x] **I3 — SIGPAC near-point search does not use the GIST index (PostGIS type mismatch)** — `handlers/sigpac.rs:353-354`. The query filters on `ST_DWithin(geography(geometry), geography(...))`, the existing index is `USING GIST(geometry)` (migration `:1496`), so on `geometry`, not `geography`. The wrapping makes the index unusable → seq scan over all SIGPAC parcels, plus `ORDER BY ST_Distance` as a K-sort sort without a distance index. `CREATE INDEX idx_sigpac_parcels_geog ON sigpac_parcels USING GIST (geography(geometry));` — the single biggest lever in the repo. Done (2026-10-03): migration `0000000007_geography_spatial_indexes.sql`. `ST_DWithin(geography, geography, distance)` has no `geometry` overload — the distance argument is in metres, which only the geodetic type takes — so `geography(geometry)` on the column is a function call the planner cannot match against `GIST(geometry)`. Measured on this schema with 500 parcels: sequential scan 50.0 ms, index scan 0.33 ms. Both indexes are kept, because predicates that stay in the `geometry` type still need the existing one. The same mismatch was present on `lpis_reference_parcels`, so the index was added there too.
- [x] **I4 — Argon2 blocks the async runtime thread** — `postgres/user.rs:183-187`. `Argon2::default().hash_password(...)` runs directly in the `async move` block, 50–100 ms CPU per login. Across the entire workspace there are zero `spawn_blocking` calls. Wrap in `tokio::task::spawn_blocking` — one login currently blocks all workers of the Tokio runtime thread. Done (2026-10-03): four call sites, not one — hashing in `PgUserRepo::create`, in `handlers/system.rs` and in `handlers/demo.rs`, plus verification in the login path. All four went through a new `agrocore_infrastructure::password` module, because a call site that forgets the wrapper is the original bug and is invisible in review: `hash_password` looks correct until you notice the missing `spawn_blocking`. Five unit tests, including one that asserts two concurrent hashes on a single-threaded runtime finish in the time of one rather than the sum — that test fails if the work is moved back onto the async worker.
- [ ] **I5 — LPIS HTTP cache is implemented but never enabled** — `lpis-providers/src/base.rs:76` sets `cache: None`; `create_default_registry()` (`lpis-providers/src/lib.rs:23-63`) never calls `.with_cache(...)`. So `get_cached_or_fetch` (`:122-134`) always bypasses the cache branch. Every import site triggers an HTTP request to SIGPAC/BRP. Build an `LpisCache` (moka) into the registry and set it per provider.
- [ ] **I6 — Audit triggers on high-frequency tables** — migration `:1735-1759` installs `audit_trigger_*` on every table with `tenant_id`, including on `worker_locations`, `clock_entries`, `worker_task_statuses`, `task_data`, `weather_data`, `soil_moisture_readings`. `audit_trigger_function` writes `to_jsonb(OLD)+to_jsonb(NEW)` per row. On a location ping: two writes plus two JSONB serializations instead of one. Drop the trigger for high-frequency tables; audit is substantively covered there by `work_logs` anyway.

#### I-B N+1 patterns (P1)

- [ ] **I7 — Clock-in→clock-out resolution per row** — `postgres/clock_entry.rs:164-190` (`find_sessions`) and `:281-298` (`total_hours_worked`). Both load all clock-ins in the time window and query again **per clock-in**. At 30 days × 2 events ≈ 60 sequential roundtrips. `LEFT JOIN LATERAL` on the next clock-out; the index `idx_clock_entries_worker_time(worker_id, timestamp)` (migration `:1596`) already covers this. 60 roundtrips → 1.
- [ ] **I8 — Inventory balances aggregated per item** — `postgres/inventory_item.rs:106-140` (`find_below_minimum`) and `:158-191` (`find_balances`). Both load all active items without compensation and run a `SUM()` query **per item**. `LEFT JOIN` with `GROUP BY i.id` instead of the loop; additionally `CREATE INDEX idx_inv_txns_item_created ON inventory_transactions(item_id, created_at DESC)` is missing (only tenant/type/created_at/batch/expiration exist, `:1586-1590`). 201 queries → 1.
- [ ] **I9 — Animal treatments in the veterinary report** — `reporting-service/src/main.rs:192-209`. Per animal (up to `per_page: 500`, `:171`) a `find_treatments_by_animal` query → 1 + 500 roundtrips. One JOIN query.

#### I-C Missing indexes (P1)

- [ ] **I10 — Ten missing indexes** — migration compared against queries: `inventory_item.rs:58` `(item_id, created_at DESC)`; `order.rs:74,271` `(tenant_id, created_at DESC)` (only `tenant_id`, `:1411`); `weather_data.rs:67` `(tenant_id, timestamp DESC)` (only `(station_id, timestamp DESC)`, `:1426`); `tasks` `(tenant_id, status)` and `(status, scheduled_start)` (`:1420-1424`); `animal.rs:80` `(tenant_id, created_at DESC)` (none); `harvest_lot.rs:43` `(tenant_id, created_at DESC)` (only `tenant_id`, `:1430`); `frost_warning.rs:123` partial `(tenant_id, created_at DESC) WHERE is_active`; `tenant.rs:37` `(is_active, id)`; `customer.rs:235` trgm on `company` and `customer_number` (trgm only on `name`,`email`, `:1628-1629`); `equipment.rs:144` `lower(label)`/`lower(code)` with `gin_trgm_ops`. Each item turns sort+scan into index scan; at 50k–1M rows a factor of 10–100 in p95 latency.

#### I-D Payload, pagination, transactions (P1)

- [ ] **I11 — `SELECT *` on wide JSONB/geometry tables (167 hit sites)** — `site.rs:104-112` and `:161-180` load `boundary` (GEOMETRY), `center`, `plots`, `properties`, `custom_fields`, `lpis_data`, `sigpac_data`, `row_config`, `bbch_stage`. At 500 sites with polygons that yields a multi-MB payload per list page. Same for `tree.rs:45`, `group.rs:45`, `building.rs:52`, `livestock.rs:52`, `order.rs:74`, `worker_task_status.rs`, `task_data.rs:49`. Reduce list queries to a DTO column list (the `SiteDb` pattern in `site.rs` shows it for details), geometry only in the detail endpoint.
- [ ] **I12 — Queries without `LIMIT`** — `inventory_item.rs:98,150`; `equipment.rs:680` (`equipment_fuel_consumption`) and `:743` (`equipment_usage_log`) — complete history per device without LIMIT and without a time window; `order.rs:111,239`; `livestock.rs:130`, `tree.rs:123`, `group.rs:123`, `breed.rs:74`, `variety.rs:83` (master data completely); `frost_warning.rs:123`; `plant_protection_record.rs:170`; `api/handlers/iot.rs:130-145` loads **all** `iot_devices` of the tenant and filters `site_id`/`status` in Rust, without pagination in the response. Replace with `Pagination` (default 20, max 500, `shared/src/lib.rs:71-85`), for histories additionally a time window.
- [ ] **I13 — Two queries per list request (`COUNT` + `SELECT`)** — consistently in about 25 repos (`site.rs:98+104`, `user.rs:132+140`, `equipment.rs:190+214`, `customer.rs:233+243`, `weather_data.rs:61+67`). The `COUNT(*)` runs with the same filter and scales linearly. Keyset paging instead of `OFFSET`, `total` via `COUNT(*) OVER()` only if genuinely needed. Halves the roundtrips and eliminates the deep OFFSET costs.
- [ ] **I14 — Missing transactions on multi-write** — `order.rs:139-154` (INSERT + audit log), `order.rs:170-208` (SELECT + UPDATE + audit log, not atomic, TOCTOU on `old_order`), `handlers/workforce.rs:446-560` (per order `update()` + `publish()` + `find_by_user_id()` + `work_log_repo().create()`), `handlers/orders.rs:328-337` (follow-up orders committed individually in a loop). Add a `&mut Transaction` variant to the repo updates; make the batch operations in `import_service.rs:56-95,408-493` transactional too, plus `UNNEST` batch insert. Primarily a correctness gain.
- [ ] **I15 — No compression** — `api/lib.rs:146-169`, middleware chain `Prometheus → SecurityHeaders → Governor → CORS → Metrics`, **no `Compress`**. For GeoJSON, Excel and SIGPAC responses in the MB range. Add `Compress::default()`, ~70–85 % fewer transfer bytes for text-based responses.

#### I-E Blocking work on async threads (P1)

- [ ] **I16 — No `spawn_blocking` anywhere in the workspace, five CPU hotspots** — zero hits verified. Affected: **LPIS parsing** (`lpis-providers/src/sigpac.rs:146-200`, `gml_to_polygon` with thousands of `parse::<f64>()`, seconds of CPU at `per_page` up to 1000 parcels); **Excel reports** (`reporting-service/src/main.rs:134-166,180-213`, `rust_xlsxwriter` synchronous in async); **geometry service** (`geometry-service/src/worker.rs:62-88`, area and contains computation directly in the NATS consumer loop, blocks all further `geometry.request`); **checksums** (`backup-service/src/verification.rs:214-239`, SHA-256 over loaded files); **geodesy in the domain layer** (`domain/src/entities/spatial/mod.rs`, Haversine per object in `site.rs:561-564`). Wrap all five in `spawn_blocking`, for geometry additionally `rayon::par_iter`. The most important one during LPIS bulk import.

### Block J — Unwired code and schema drift (P0/P1)

Findings from the analysis of missing implementations. `cargo check --workspace` passes
cleanly — these are **runtime bugs** throughout, not compile errors. Precisely for that reason
they have gone unnoticed so far.

#### J-A Modules and handlers without registration (P0)

- [x] **J1 — `sigpac` and `livestock` are declared but never registered** — done (2026-10-01). Both `.configure(...)` added, so that `/api/v1/sigpac/parcels` (3 routes) and `/api/v1/livestock/animals` (7 routes) exist; they were documented in `openapi.rs` but did not exist. The simultaneously double-registered `livestock_new::configure` is reduced to a single occurrence. The two livestock modules share the prefix `/livestock` but use different subpaths (`/animals…` vs. `/`, `/{id}`, `/by-plot/…`), so there is no collision. — `handlers/mod.rs`. Both modules are declared as `pub mod` and have a finished `configure()`, but are never called (verified: 22 `.configure()` calls, neither `sigpac::configure` nor `livestock::configure`). Thus `/api/v1/sigpac/parcels` (3 routes) and `/api/v1/livestock/animals` (7 routes) **do not exist**, although they are documented in `openapi.rs:243-251`. Consequence: `animal_repo` is completely unreachable, and the veterinary export breaks because `reporting-service/src/main.rs:175,196` uses the repo directly. Add both `.configure(...)`.
- [x] **J2 — Three site-import handlers without a route, the entire `ImportService` is dead code** — done (2026-10-01). `POST /sites/import`, `POST /sites/import/geojson` and `POST /sites/import/shapefile` registered and deliberately placed **before** `/sites/{id}`, otherwise the id pattern would have tried to parse "import" as a UUID. Thus 719 lines of `ImportService` (LPIS registry, GeoJSON, Geozero shapefiles) and the matching wrappers in the Admin UI (`api.rs`) are reachable for the first time. — `handlers/sites.rs`. `import_sites` (`:238`), `import_geojson` (`:270`) and `import_shapefile` (`:302`) are fully implemented and use `ImportService` with the LPIS registry and Geozero shapefile parsing (719 lines of service code) — but have **no route** (verified: no `web::resource` with `import` in `sites.rs`). Add routes; when doing so register the static paths **before** `/sites/{id}`, otherwise the `{id}` resource swallows the names.
- [x] **J3 — `livestock_new::configure` registered twice** — fixed together with J1 (2026-10-01). — see H7. `mod.rs:244` and `:250` both call the same function.

#### J-B Schema drift (P0)

- [ ] **J4 — Around 45 columns the repos expect do not exist in the schema** — partially worked through: `animals.identifier`, `animals.livestock_type` and `animals.status` were added in migration `0000000003`, with a backfill from `tag_number`/`species` and a BEFORE trigger that derives `livestock_type` from `species` on every INSERT and UPDATE. The remaining tables stay open, among them `work_logs` with a completely divergent schema. Compare all INSERT/UPDATE/FROM column lists against `information_schema` (only three `ALTER TABLE ADD COLUMN` exist in the current state). The most severe: **`work_logs`** — the repo needs `date, hours_worked, overtime_hours, rest_period_hours, task_description, site_id, is_night_shift, breaks_taken` (`work_log.rs:85`), the migration instead has `task_id, started_at, ended_at, duration_minutes, notes`. Complete schema mismatch. Further: `weather_data` (`wind_direction_deg, solar_radiation_wm2, pressure_hpa, soil_temperature_c, soil_moisture_percent, leaf_wetness`), `growing_degree_days` (`base_temp_c, actual_mean_temp_c, gdd, accumulated_gdd, crop_type` — migration has `min_temp_c/max_temp_c/gdd_base_10/gdd_base_5`), `financial_records` (`amount, category, reference_id` vs. `amount_eur, currency, date, invoice_ref`), `olive_oil_records.grove_id` vs. `olive_grove_id`, plus `harvest_lots.site_ids`, `pac_applications.documents_urls/eco_schemes/total_eligible_area`, `pest_risks.confidence`, `cost_centers.cost_center_type/reference_id`, `audit_logs.ip_address`, `animals.identifier/status`, `workers.contract_type/language`, `weather_stations.manufacturer/model/serial_number`, `worker_task_statuses.updated_at`, `task_data.ended_at/handoff_to_worker_id/is_session_complete`, `plant_protection_records.pre_harvest_days/re_entry_days/total_quantity/applicator_license`, `applicator_licenses.license_type`, `compliance_checklists.items`, `equipment_maintenance_log.downtime_hours/labor_hours`. Consolidated drift migration with `ADD COLUMN IF NOT EXISTS`.
- [ ] **J5 — `varieties` and `breeds` have no `tenant_id` column, but their repos filter on it** — migration: `varieties` has only `id, category, name, origin, created_at`, `breeds` only `id, species, name, origin, created_at`. The repos filter with `tenant_id = $2` (`variety.rs:21,43,50,154,176`) — this is a further schema bug with data impact: varieties and breeds are **global instead of tenant-isolated**. Add the column and give it an FK to `tenants`.
- [x] **J6 — Naming bugs instead of missing tables: `animal_treatments`, `animal_grazing_records`, `water_usages`** — done (2026-10-01). `animal.rs` queried `animal_treatments` and `animal_grazing_records`; the tables are called `treatment_records` (migration `:676`) and `grazing_records` (`:665`), and the repo additionally wrote `treatment_date` instead of `date`. Switched to the existing tables, no new schema needed. `water_usages` was solved with `ALTER TABLE water_usage RENAME TO water_usages`, so that all call sites are correct at the same time.

#### J-C Placeholder data and false feedback (P0)

- [x] **J7 — `delete_backup` deletes nothing but returns success** — `handlers/backup.rs:239-267`. The function only checks `get_job_status` and answers 200 with "deleted", without removing an object from storage. Together with F2 and H3 one of three places where the backup API pretends success.
- [ ] **J8 — `get_device_telemetry` always returns empty, `send_command` lies about the status** — `handlers/iot.rs:404-408` returns `measurements: vec![]` with the comment "would query a time-series database"; `:463-464` sets `CommandStatus::Sent`, **without publishing anything**. `AppState` only has `messaging: Arc<MessagingClient>` (NATS); the `MqttClient` from `agrocore-messaging` is not instantiated in `lib.rs:113`, although `AgroCoreConfig.mqtt_broker` exists. Include `UnifiedMessagingClient`, only set the status after a confirmed publish, write telemetry into a table with a `timestamp` index and retention.
- [ ] **J9 — `fetch_weather` returns hardcoded values** — `handlers/calculation.rs:514-524` returns `temperature_c: Some(20.5)`, `humidity: 65.0`, `pressure: 1013.25`. The parsed provider is discarded in `_service_type` (`:508-512`). The three real providers already exist in `crates/weather-service/src/providers/`. Hook up to `agrocore_weather::WeatherAggregator` or remove the handler.
- [x] **J10 — `list_lpis_providers` invents base URLs** — `handlers/settings.rs:236-246` returns `https://{country}.example.com/wfs` instead of the real `ProviderConfig::default()` values. Placeholder domains that look like configuration.

#### J-D Dead code and features that cannot be enabled (P1)

- [ ] **J11 — Two domain traits without any implementation** — `domain/src/repositories.rs:694` `SoilMoistureReadingRepo` and `:744` `SoilMoistureAlertRepo` (four methods each): no Postgres implementation, no `db` accessor, no handler routing. The tables exist, and `weather-service/src/worker.rs:245` calls `process_soil_misture_alerts(...)` — **the alert path leads nowhere**. Add the repos following the existing `repo!` pattern and pre-instantiate them in `PostgresDb`.
- [ ] **J12 — The `mocks` and `depreciation` features cannot be enabled** — `api/Cargo.toml:72`, `domain/Cargo.toml:25`, `infrastructure/Cargo.toml:28` declare `mocks`; no crate references it. Thus `#[cfg_attr(feature = "mocks", automock)]` on 61 repository traits is ineffective — **not a single mock exists**, all handler tests need a real database. `depreciation` is switched off via `#![cfg(feature = "depreciation")]` in `domain/src/lib.rs:1-2`, `domain/src/depreciation.rs` (58 lines) is never compiled, and `run_monthly_amortization` (`:52`) has zero callers. Enable or remove the features.
- [ ] **J13 — Dead configuration code** — `shared/src/config.rs:65` `mqtt_broker` and `:68` `rust_log` have zero uses; the MQTT client instead reads hardcoded env vars. Wire up or remove. Plus `domain/src/repositories.rs:15` `VisibilityAwareEntity` — an empty marker trait with zero implementations.
- [ ] **J14 — `scheduler::stop()` and further stubs** — `scheduler/src/service.rs:79` logs "graceful shutdown not fully implemented" and does nothing else: no cancel token, running jobs are not aborted. `backup-service/src/manifest.rs:136` `get_schema_version()` always returns `Ok("unknown")`. `messaging/src/lib.rs:475` `try_deliver` is `Ok(())` without delivery confirmation (which is why J8 is not cleanly solvable). `notification/types.rs:132` `health_check()` → `Ok(())` and `:144` `is_available()` → always `true`: **the notification channel health checks are blind**.
- [ ] **J15 — Compliance filters load the whole table and filter in Rust** — `handlers/compliance.rs:212-234` and `:249-271` call `find_all(Pagination::default())` and filter in Rust, with their own comment "would require a find_by_site method". With a growing inventory this breaks performance **and** correctness, because only the first default page is taken into account. Add `find_by_site`/`find_by_type` to the repo.
- [ ] **J16 — Fertilizer costs are a fixed percentage** — `handlers/nutrition.rs:101` sets `cost_eur: total_amount * 0.5 // Placeholder cost`. Financially relevant for PAC and cost-center reports.
- [ ] **J17 — `iot.rs` bypasses the repo layer** — raw `sqlx::query` on `iot_devices` with a JSONB blob per device (`find_device:47-61`). Functional, but breaks the `Repository<T>` pattern and the `measure_sqlx_query!` instrumentation.

#### J-E Missing tests exactly where it broke (P1)

- [ ] **J18 — The auth round-trip is untested** — no test calls `find_by_refresh_token`, which is why the total failure went unnoticed. Add a `login → refresh → logout` test.
- [x] **J19 — No migration schema assertion test** — partially done (2026-10-01). Three tests in `crates/infrastructure/tests/database_setup_tests.rs` now check table existence, tenant reference and the readability of newly created rows; all three run green against the PostGIS test image. The part that compares the **column lists** of the repos against `information_schema` is still missing — it would find J4 and J5 automatically.

**Findings from the first successful fixture run (4 green, 5 red).** The fixture work made five pre-existing defects visible for the first time; the red tests are not caused by the migration:

- `test_repository_tables_exist`, `test_new_domain_rows_are_tenant_scoped`, `test_postgis_extension_enabled`, `test_uuid_ossp_extension_enabled` — green.
- `test_database_migrations_applied` — expects the table `spatial_properties`, which no migration creates; either add the table or remove the expectation.
- `test_site_crud_operations` and `test_updated_at_trigger` — both fail with `INSERT has more target columns than expressions`. Two tests, one cause: the shared site INSERT in the test environment has more target columns than bound values.
- `test_tenant_creation_and_isolation` — `common/mod.rs:71` uses a fixed `slug = 'test-tenant'` in `create_test_tenant`; as soon as a second test creates the same slug, `tenants_slug_key` fires. Suffix the slug with a uuid per call.
- `test_tenant_scoped_tables_have_tenant_id` — reports `kelter_deliveries`, `order_sites`, `spatial_ref_sys`, `tenants`, `user_sites`. `kelter_deliveries` independently confirms **B1**: the table has neither `tenant_id` in the schema nor a filter in the repo. `user_sites` and `order_sites` are pure mapping tables and belong in the exception list, as do the PostGIS system table `spatial_ref_sys` and `tenants` itself. Concretely to do: extend the exception list in the test by `order_sites`, `spatial_ref_sys`, `tenants`, `user_sites` and fix `kelter_deliveries` under B1. — no test compares repo INSERT column lists against `information_schema.columns`. A single such test would have found J4, J5 and J6 immediately.
- [ ] **J20a — The test fixture never ran successfully** — `crates/infrastructure/tests/common/mod.rs`. `testcontainers_modules::postgres` is hardwired to `postgres:11-alpine`; but migration `0000000000` needs PostGIS. All nine integration tests failed with `extension "postgis" is not available` and were therefore never green. Fixed: `GenericImage::new("postgis/postgis", "16-3.4")` plus connect retry. Four tests are still red afterwards (missing table `spatial_properties`, wrong number of INSERT columns, slug unique constraint, tables without `tenant_id`) — they are visible at all for the first time. Fixed in the course of J19.
- [x] **J20 — No route completeness test** — fixed. `crates/api/tests/route_inventory_tests.rs` builds the real Actix application and probes every candidate path against the running router, so the route list cannot drift from the handlers. The UI path list is read from the Admin UI sources. A negative check — renaming `/livestock` to `/livestock-renamed` — was run to confirm the test actually fails when a route disappears, which the old hand-written list could not do.
- [x] **J21 — No multi-tenant isolation test** — fixed. `crates/infrastructure/tests/tenant_isolation_rls_tests.rs` adds four tests that run under `SET ROLE agrocore_app` instead of the migration superuser, so RLS is actually enforced. It verifies that a pinned tenant sees only its own rows (with an unfiltered `SELECT`, not a filtered one), cannot write or delete another tenant's row, and can still write its own. The load-bearing test is `rls_is_actually_active_for_the_app_role`: if the app role is not subject to RLS, it fails there rather than letting the other three pass vacuously.

  Writing it surfaced a production bug: `DELETE /api/v1/system/tenant` failed on any tenant that had an audit entry. Three foreign keys on `tenants` lacked `ON DELETE CASCADE`, and fixing them exposed a second failure — the 41 cascade children carry audit triggers that each try to write an `audit_logs` row for a tenant that no longer exists. Migration 8 fixes both. Reproduced against PostgreSQL 17: without it the delete raises `audit_logs_tenant_id_fkey`, with it the tenant and its data are removed and a neighbouring tenant is untouched.

---

## P0 — Critical

### Complete the notification channels

From phase 8 section 3. The dispatcher is productive, but the channel list is incomplete.

- [ ] Push channels: Firebase (FCM), APNs, WebPush (`crates/messaging/src/notification/channel.rs`)
- [ ] Inbound webhooks for receiving (WhatsApp, Telegram, email reply)
- [ ] Additional email and SMS providers: Postmark, Vonage, Plivo, Sms77
- [ ] Tenant user preferences (which channel for whom, quiet hours, escalation)

### Finish the backup service

From phase 8 section 4. Affects `crates/backup-service/`.

- [ ] SFTP host key pinning against a `known_hosts` file — `check_server_key` currently accepts any key (`sftp_backend.rs`), that is an open MITM risk
- [ ] Make cloud downloads memory-efficient — `object_store` 0.11 provides no async byte stream, `GetResult::bytes()` loads the object completely. Affects S3, Azure and GCS; Local, SFTP and WebDAV already stream
- [ ] Integration tests against real cloud instances (S3/MinIO, Azure Blob, GCS) in CI
- [ ] Integration tests against real SFTP and WebDAV servers
- [ ] Secure the NATS progress events (0–100 %) with integration tests
- [ ] Age and KMS encryption — currently only AES-256-GCM (`encryption.rs`)
- [ ] Disaster recovery runbook: RTO < 15 min for a 50 GB database, single tenant

---

## P1 — MVP

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

From phase 4, not started so far.

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

### AI analytics (module 17)

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
- [ ] Blockchain traceability: supply chain transparencyW

## G2 — API endpoints with no UI caller (found by J20, completed 2026-10-03)

The route inventory test (`crates/api/tests/route_inventory_tests.rs`) probes the
running application and compares every registered path against what the Admin UI
actually calls. It started at 32 uncovered endpoints and reached zero.

### Five pages that were shells

`/trees`, `/groups`, `/buildings`, `/livestock` and `/plot/entities` were routed,
had complete CRUD handlers in the backend, and the pages themselves did nothing.
They collected form input and discarded it:

```rust
spawn_local(async move { let _ = (label.get(), count.get()); })
```

`buildings.rs` was worse — it showed a success toast for a write it never
performed. `plot_subentity.rs` was worst: it rendered four rows of invented data — a herd
called "Herde 1", two goats, four cork oaks — hard-coded in the markup, so it
looked like a working overview while showing nothing about the tenant. (That
sentence is still in the module's doc comment, describing what it used to do.)

All five now call the API and support list, create, update and delete.

The livestock page also sends `livestock_type` and `status`, which
`CreateAnimalDto` requires without a Rust default — the request would have been
rejected with a 422 the page could not have surfaced.

### 27 endpoints that had no page at all

Committed as 0.39.0 for the five pages above; 0.40.0 covers the 27 endpoints.

| Page | Endpoints |
|---|---|
| `/calculators` | 12 `calculate/*` plus the two duplicate registrations of fertiliser-amount and profitability |
| `/water` | 3 `water/*` — sources, usage, quotas |
| `/harvest` | 4 `harvest/*` — seasons, lots, deliveries, cold chain |
| `/agriculture` | 5 `specialized/*` — olive groves, olive oil records, vineyards, kelter deliveries |
| `/weather/warnings` | 3 `weather/*` — frost warnings, active frost warnings, pest risks |
| `/worker/logs`, `/worker/locations` | 2 `workforce/*` |
| `/compliance/licenses` | 1 `compliance/applicator-licenses` |

Three findings worth keeping:

- `/workforce/locations` is not a CRUD collection. `POST` takes the worker id from
  the authenticated user, not from the body, so a client cannot report a position
  on someone else's behalf. The page offers the current positions and a way to
  post your own — no edit-and-delete table, which would imply a capability the
  endpoint does not have.
- Two calculations are registered twice, by `calculation.rs` and by
  `nutrition.rs` / `specialized.rs`, with near-identical handler bodies. Both paths
  are live and both are now reachable.
- The enum wire values differ per enum and are not guessable: `CropType` and
  `RiskLevel` carry lowercase serde renames, `WaterSourceType` and `IrrigationMethod`
  carry `strum` snake_case but no serde rename and so serialise as the variant
  name, and `LotStatus` is lower-case. Each select was checked against its enum.

### C3 — the import path had no body limit (fixed, with a correction to the finding)

The audit recorded C3 as "no payload limit, DoS via the import". That was half
right, and the half that was wrong is the half that matters.

`actix_web::web::Json` has always defaulted to 2 MiB — `JsonConfig::DEFAULT_LIMIT`,
declared in `actix-web-4.15.0/src/types/json.rs`. So an ordinary endpoint was never
unbounded, and my first negative test confirmed it: removing the explicit
`JsonConfig` left all five tests green.

What *was* unbounded is the import path. `/sites/import/shapefile` carries a file as
Base64 — a third more than the binary — and a municipality's parcel shapefile is
genuinely 10–30 MB, so the default rejects legitimate work. A handler in that
position cannot simply raise the global limit, and re-registering the routes in a
scope with a second `JsonConfig` does not work: they are already mounted by
`handlers::configure`, and a second registration of the same path is shadowed by the
first rather than overriding it.

So the limit is now per-extractor. `LargeJson<T>` reads the body itself with
`MAX_IMPORT_PAYLOAD` (64 MiB) and is used by exactly the three import handlers;
everything else keeps 2 MiB, now written out explicitly rather than inherited, so
the value is greppable and pinned by a test rather than at the mercy of an Actix
upgrade.

The oversize check runs on every chunk, not once at the end, so an oversized body is
rejected without ever being fully buffered.

Two things about `LargeJson` that are worth stating because they are not obvious:
`Payload::take` in actix-http 3 takes no argument and applies no limit, so the
running total is compared by hand; and the payload stream is *moved* into the future
rather than borrowed, because `FromRequest::Future` is `'static` and cannot hold a
borrow of `&mut Payload`.

#### A finding the audit missed

`the_global_limit_applies_to_ordinary_endpoints` first failed with 405, not 413: the
settings group resource registers only GET and PUT, so a POST is rejected before the
payload is read and the test would have passed for the wrong reason. Worth recording
because it is the shape of a false pass — a test that looks like it covers the size
limit and actually covers the routing table.

## M5 — the `mocks` test build failed on unused imports (fixed 2026-10-03)

A direct consequence of M1, and a mistake rather than a pre-existing defect.

The M1 fix replaced thirty `AppState { ... }` literals with
`crate::common::state_with(mock_db)` calls, but left the imports each literal had
needed: `AppState`, `Database`, `EncodingKey`, `Header`, `encode`, `serde::Serialize`.
Ten files each had four or five of them.

This was invisible locally because I ran `cargo clippy --workspace --all-targets`
without `--features mocks`. Every one of the eleven targets has
`required-features = ["mocks"]`, so without the flag Clippy never compiled them.
GitHub sets `RUSTFLAGS: "-D warnings"` workflow-wide, so its test job rejects the
unused imports even though `cargo test` would otherwise have run.

**The gate was wrong, not just the code.** Every local report of "clippy clean"
should have included `--features mocks`; ten test files and 139 tests were outside
it.

Two more lints, both of them mine rather than inherited:

- `weather_fetch_tests.rs` had an `if status == OK { read_body } else { read_body }`
  with identical branches — a leftover from restructuring.
- `payload_limit_tests.rs` asserted a relationship between two `const`s at runtime.
  That can never fail, so it was moved into a `const _: () = { assert!(..) }` block
  where it is checked at compile time, and the values themselves are pinned in a
  separate test with a message, because the boundary tests generate bodies at them.

### `field_reassign_with_default` at thirty-one sites

`let mut mock_db = MockDatabase::default(); mock_db.repo = Some(...)` trips Clippy's
`field_reassign_with_default`, and the pattern appeared thirty-one times across ten
files — including two I had written that week.

`crate::common::db_with(|db| db.repo = Some(...))` replaces it. The builder exists in
`tests/common/mod.rs` rather than at each site because that is where the reasoning
belongs: the value is known before the binding exists, so the binding never should
have been mutable.

### Verified with the exact CI commands

```
RUSTFLAGS='-D warnings' cargo test --workspace --features=mocks --no-fail-fast
    → exit 0, 0 errors, 0 failed test results

RUSTFLAGS='-D warnings' cargo clippy --workspace -- -D warnings   → 0 errors
RUSTFLAGS='-D warnings' cargo build --target wasm32-unknown-unknown \
    --release --lib -p admin-ui                                   → exit 0
```

139 API tests with mocks pass.

## O1 — the Admin UI could not start without a network (fixed 2026-10-04)

Discovered by switching the network off and watching the page fail with
`net::ERR_INTERNET_DISCONNECTED` for the document itself.

### A correction

The 0.47.0 survey said the UI has no offline path: "the only localStorage use is
auth tokens, there is no `PendingChange` store, no offline state". That was wrong.
`public/sw.js` existed — 6,924 bytes with a full strategy: cache-first for static
files, network-first fallback for API calls, IndexedDB for queued task updates,
background sync, push handling. It was not dead in the sense of being unfinished; it
was dead in the sense that **nothing registered it**. `serviceWorker.register`
appeared nowhere in the repository.

The claim came from a partial search — I checked localStorage and did not check for a
service worker. That is the same error as the SIGPAC index measurement I inherited and
the Clippy gate I reported clean at 0.44.0: a statement about absence made without
looking.

### Why it would have failed anyway

Five defects, each of which alone prevents a working offline start:

1. `STATIC_ASSETS` named `/styles/tailwind.css` and `/styles/daisyui.css`. Neither
   exists. `cache.addAll` rejects the whole install if any URL fails, so one wrong
   path means no service worker at all.
2. It cached two API endpoints during install. An unauthenticated `GET /api/...`
   answers 401, `addAll` rejects non-2xx, install fails.
3. `manifest.json` declared `/icons/icon-*.png` and `/icons/task-icon.png`. That
   directory did not exist.
4. `cacheFirst` for every non-API request served the shell from cache forever — a
   deployed fix would not reach a client that has the application open.
5. Both offline pages loaded Tailwind from the CDN, so the pages that exist only for
   the no-network case could not render in it.

### What was done

**CDN references removed.** Twelve files under `public/vendor/`: DaisyUI, Leaflet,
Leaflet-Draw, Tailwind's JIT runtime, and Leaflet's six icon files. `index.html` now
has no external reference at all — verified, zero `src`/`href` pointing off-host.

Two of those were not in `index.html` and would have been missed by reading only
that file: `src/leaflet.js` fetched the red location marker from
`raw.githubusercontent.com` and its shadow from `cdnjs.cloudflare.com`.

Tailwind's CDN entry is the JIT *runtime* — 407 KB of JavaScript that compiles CSS in
the browser from the DOM. Vendoring it keeps the current behaviour offline, including
classes a view produces at runtime. Building Tailwind ahead of time would be smaller
and faster but needs a content scan of the Leptos views, and a wrong static build
silently drops every dynamically-named class. Recorded as O2 rather than done.

**The worker rewritten and registered.** `src/main.rs` now calls
`register_service_worker()` after `mount_to_body` — after, so a registration failure
cannot stop the application from starting, and the rejection is logged rather than
surfaced. The precache list names only servable paths, `precacheAll` skips what is
missing instead of failing the install, API responses are cached only when they
succeeded, navigations are network-first with the shell as fallback so a deep link
resolves and a deploy is picked up, and cache names are versioned so activation
deletes the previous build.

**Icons generated** from `logo_trans.png` with the maskable safe zone, since
`logo.png` has an opaque cream background and would render as a coloured square.
The two manifest shortcuts referenced icons that do not exist; they were removed
rather than pointed at fabricated images, because a shortcut with a missing icon is
unusable and a wrong one is worse.

**nginx** serves `sw.js` with `no-cache` — a worker cached forever can never be
updated, and the update is what applies a new cache version — and `/vendor/` with a
week, since those paths are unversioned.

### The finding that only the artefact showed

The first successful image build — 105 MB, exit 0 — contained the WASM bundle and
**none of the offline files**. `sw.js`, `manifest.json`, the icons and all twelve
vendored files were in the repository and absent from the image.

Trunk copies nothing from `public/` on its own. A file or directory only reaches
`dist/` when the HTML declares it with `rel="copy-dir"` or `rel="copy-file"`. Nine
tests were green, every reference resolved, every precache entry was servable — and
the offline capability was not in the thing you run. Every one of those tests reads
the repository, so every one of them would have passed with the capability completely
missing from the output.

This is the fourth time in this work that a check read the repository and reported
"fine" while the thing that mattered was somewhere else: the SIGPAC measurement
inherited at the start, the Clippy gate at 0.44.0, the "no offline path" claim at
0.47.0, and now this. The pattern is consistent enough to name: **a test that reads
the source cannot tell you whether the artefact is correct.**

So the assertions were extended in two directions. `public_assets_are_declared_for_
copying` checks the declaration exists, and the image itself was inspected with
`docker run … ls`. That second step is the one that matters and the one I would not
have done without looking inside.

### A second, unrelated defect the image run exposed

nginx refused to start in the standalone container:

```
[emerg] host not found in upstream "api" in /etc/nginx/conf.d/default.conf:58
```

nginx resolves an upstream host at startup and aborts if it cannot. `api` is a Docker
Compose service name that only resolves on a Compose network, so the image could be
built but never run on its own — which is exactly what is needed to check offline
behaviour against a real server.

`proxy_pass` now uses `${AGROCORE_API_UPSTREAM:api:8080}`. Compose behaviour is
unchanged; standalone the variable can point anywhere reachable.

This is worth recording separately because the symptom pointed at the offline work
and the cause had nothing to do with it. A container that will not start is a bad
place to discover an unrelated configuration problem.

### Verified

- `index.html`, `offline.html`, `worker-tasks-offline.html`: 0 external references,
  0 missing files.
- `SHELL_ASSETS`: every entry servable, no API endpoint among them.
- Docker build exits 0; the built image was inspected and `vendor/`, `icons/`,
  `sw.js`, `manifest.json` and the offline pages are present in `/usr/share/nginx/html`.
- 11 tests in `crates/admin-ui/tests/offline_capability_tests.rs`, each checked against
  a deliberately broken input: removing the registration, putting a CDN reference
  back, deleting a vendored file, removing the `data-wasm-opt` attribute and removing
  the copy declarations each fail exactly the corresponding test.

The test that matters is still yours: open the built image, switch the network off,
reload. It should start.

## O2 — Tailwind is compiled in the browser (open)

- [ ] Build Tailwind ahead of time from a content scan of the Leptos views, so the
      407 KB JIT runtime leaves the bundle. The obstacle is that `view!` macros
      construct class strings at runtime; the scan has to see through
      `format!("order_type_{}", value)` and every other dynamic class name, or a
      static build silently drops exactly the classes that vary.

## O3 — wasm-opt in the Docker build (worked around, root cause open)

`wasm-opt` exits 1 on the module Trunk produces. Three wrong turns before the actual
control was found, each of them a plausible reading of the same key:

1. `[tools].wasm_opt = "0"` read as "switch wasm-opt off". It is a *version* string, so
   it became Binaryen version `0` and Trunk fetched
   `binaryen-0-x86_64-linux.tar.gz` — a 404. Omitting the key lets Trunk use its pinned
   `version_123`, which fixed the download and exposed the real failure one stage later.
2. `[build].wasm_opt = false` read as the boolean form of the same thing. Trunk has no
   `build.wasm_opt` key at all — `grep` over `trunk/src/config/` finds `wasm_opt` only
   in `models/tools.rs`. The TOML was accepted and silently ignored, which is why the
   error message did not change.
3. The real control is the HTML attribute `data-wasm-opt` on the
   `<link data-trunk rel="rust">` element. `WasmOptLevel::from_str` maps `"0"` to
   `Off`, and without the attribute a release build uses `Default`.

`data-wasm-opt="0"` is now set in `crates/admin-ui/index.html`.

The step is a size optimisation, not a correctness one — wasm-bindgen and the linker
have already emitted a valid module before it runs — so an unoptimised build is
preferable to none. The root cause of the exit-1 is not investigated: plausibly
Binaryen 116 against Rust 1.97 output, but it could also be something in the module.

- [ ] Establish whether Binaryen `version_116` can process the module at all. If not,
        either pin a newer Binaryen through `[tools].wasm_opt` or move to a Trunk
        release that pins one which can. Both are dependency decisions, not code.

## M6 — a failed fetch rendered as an empty list (first page converted, open)

### The survey that prompted it

The Admin UI has no offline path, and the reason is not a missing cache but a
pattern: **43 `LocalResource` fetches across 21 page components call
`fetch_x().await.ok()`**. That turns a failed fetch into `None`, and `None` is
indistinguishable from "this tenant has nothing". A manager with a failing network
sees an empty table.

The writes are in better shape than the reads: 72 of 84 `spawn_local` blocks handle
their `Err` and show a toast, and no write result is discarded with `let _ =`. So the
problem is specifically on the read path.

Two further findings from the same survey, neither caused by this work:

- `error_boundary.rs` exists and is never mounted. Zero `<ErrorBoundary>` in the
  application, so a route that fails renders nothing at all.
- 45 hardcoded German strings in six files, 20 of them in `backup.rs`, in a UI with
  ten translated languages. `api.rs:46` is one: `"Sitzung abgelaufen. Bitte neu
  anmelden."`

### Why the order list was the right first page

`worker_tasks.rs` already separates loading, error and empty into three states and is
the model for the rest. `orders.rs` backs `/tasks` — the manager's order list — and
had `set_load_error` nowhere: its `error` signal existed but was only used for form
validation, so a failed load produced no error state at all.

It now has:

- `orders: Option<Vec<OrderDto>>`, `None` until a fetch succeeds and `None` again on
  failure, so the empty branch is only reachable with data that actually came from
  the server;
- a separate `load_error`, checked **before** the empty branch;
- a retry button that bumps a reload counter, because a worker in the field cannot
  navigate away to retry;
- three i18n keys in all ten languages.

The ordering is the whole fix and it is asserted as such. Checking emptiness first
defeats separate error storage, because the error never gets shown. Swapping the two
branches makes `error_before_empty` fail — verified by doing it.

Sites are fetched separately and a failure there does not blank the list, which is
the page's purpose.

### What remains

- [ ] **M6 — 42 fetches in 20 files still discard their error** — the pattern is
      mechanical and each conversion is ~40 lines. `orders.rs` is the template.
- [ ] **M7 — `ErrorBoundary` is never mounted** — mount it around the routed content
      so a failed route shows something.
- [ ] **M8 — 45 German strings bypass i18n** — start with `backup.rs`.

## M3 — the WASM build failed on feature-gated imports (fixed 2026-10-03)

The admin UI depends on `agrocore-logging` with `default-features = false`, so the
WASM build compiles that crate with neither `dev-console` nor `otlp`. Three
constructs in `crates/logging/src/layers.rs` were only valid with a tracing feature
enabled, and all three fail under `-D warnings` in that configuration:

- `use crate::config::RotationType` and `use crate::error::LoggingError` were
  unconditional, but every use of both sits behind `#[cfg(feature = "dev-console")]`
  or `#[cfg(feature = "otlp")]`.
- `init_logging`'s no-features branch ended in `return Ok(...)`, which is only
  necessary while the other `cfg` branches are also compiled. With neither feature
  on it is the sole block, and clippy's `needless_return` fires.

The imports are now gated to match their uses, and the branch returns directly.

`cargo check` passed in this configuration before the fix — only `clippy` and the
CI build caught it, which is the argument for keeping `-D warnings` in the WASM job
even though it duplicates the workspace gate.

Verified: `RUSTFLAGS='-D warnings' cargo build --target wasm32-unknown-unknown
--release --lib -p admin-ui` exits 0 and produces a 1,235,592 byte artefact.

### M4 — `agrocore-domain` does not build without its `sqlx` feature (open)

Found while checking whether M3 was the only feature-gated gap. It is not reachable
from the admin UI — `admin-ui` pulls `agrocore-shared` with `features = []` and does
not depend on `agrocore-domain` at all — but `cargo clippy -p agrocore-domain
--no-default-features` fails with `cannot find module or crate sqlx` in
`entities/site.rs` and `entities/spatial/types.rs`.

- [ ] Either gate the `sqlx::postgres` imports in `domain` on the feature that
      actually provides them, or make `sqlx` non-optional. The crate documents an
      optional `sqlx` feature for consumers that do not talk to a database, so the
      first is what the design asks for.

## M1 — eleven API test targets did not compile (fixed 2026-10-03)

`cargo test -p agrocore-api --features mocks` did not build. Eleven targets, six of
them already broken at 0.41.0. The cause was mechanical: thirty `AppState { ... }`
literals across ten files, each listing only the fields that file happened to need,
and a local copy of the `signed_token` helper in each of the eleven.

So the feature had no coverage at all, and every suite added in 0.40–0.43 that uses
a repository mock — metrics, error disclosure, impersonation, weather, payload
limits, LPIS cache — was invisible to a plain `cargo test --workspace`. CI runs
`cargo test --workspace --features=mocks`, and had it been pushed, this would have
been a red build for weeks.

### The fix

`tests/common/mod.rs` now holds `state_with(MockDatabase)` and `signed_token`. Thirty
literals became thirty one-line calls, and eleven token helpers became one.

The measure of the change: deleting `backup_service` from the helper now breaks
exactly one site. Before, it would have broken eleven. That is the whole argument
for the helper, and it is asserted rather than asserted-about.

Four fixtures had drifted from their entities and were corrected against the current
shapes, not papered over:

- `Animal` lost `weight_kg`, `last_weight_date`, `group_id`, `treatments` and
  `grazing_history`; `species` and `status` became strings, `tenant_id` a bare
  `Uuid`, `birth_date` a `NaiveDate`.
- `Site.plots` became a `serde_json::Value`; `tenant_id` a bare `Uuid`.
- `TreatmentRecord.medication` became `String`; `created_at` an `Option`.
- `Equipment` gained `fuel_capacity_liters` and `fuel_type`.

### Two things this fixed that were not on the list

`reporting_integration.rs` called `MessagingClient::new_mock()` and then
`set_mock_response`, which does not exist and never did — the test was building a
queue message the handler never reads. `new_mock()` itself is a placeholder that
panics on every call, so every one of the eleven files would have failed had it
compiled far enough to reach it. `state_with` sets `messaging: None`, which is what
the field became when the broker was made optional.

### M2 — the reporting exports cannot be tested end to end (open)

- [ ] `export_orders_excel`, `export_sites_geojson`, `export_pac_sip` and
      `export_veterinary` are NATS request-reply calls. `MessagingClient::request` is
      a real round trip with no seam, so with no broker connected they answer 500 —
      correctly. The two tests now assert that, plus that the endpoints require
      authentication, which is the behaviour that is reachable.
      The happy path needs either a NATS server in CI or a trait seam in
      `MessagingClient`. Worth doing, and it is a design change rather than a bug.

### Why this was not caught

`origin/main` was at 0.39.0 while the local branch was four commits ahead. The
broken targets were committed locally and CI never saw them. Whatever else changes,
the next step after this is a push.

## C1/D2/C2/J9/I5 — security and correctness batch (completed 2026-10-03)

### C2 — metrics endpoints had no authentication

`/metrics/db` and `/metrics/business` were registered directly on the `App` in
`main`, outside `handlers::configure`, with no extractor in the signature. Anyone
who could reach the port got the whole registry: per-table query counts and
durations, pool saturation, record counts.

Both now require an Admin. They also had their own Governor scope — 5 s per
request, burst 12 — because a scrape is a full `gather()` over every metric family,
and a dashboard polling at the API rate would spend the tenant-facing budget and do
real work on every request.

The registry carries no tenant label, so this was never a cross-tenant leak. It was
an unauthenticated operational map of the deployment, and the fix does not
oversell it.

The routes were moved into a public `metrics_routes` scope so the test can build
the same scope the server builds. They were previously invisible to the route
inventory test, which builds its app from `handlers::configure`.

### C1 — 5xx bodies carried database internals

`ApiError::error_response` rendered `self.0.to_string()` for every status. For a
500 that string is the `Display` of `sqlx::Error`, which names the table, the
column, the violated constraint and — for a failed decode — the Rust type expected.
It also changes whenever the schema does, so it was not reliably parseable.

A 5xx body now says only that an internal error occurred and points at the log.
4xx messages are unchanged, because they are authored and clients read them.
`NotImplemented` keeps a specific 501 message — and now actually returns 501; it
returned 500, which the audit had not flagged.

### D2 — impersonation was unreachable, and three defects sat behind it

`impersonate` compared roles against `"admin"` and `"superadmin"` while
`generate_jwt` emits `"Admin"`. No token the system produces matched, so nobody
could impersonate anyone. Fixing only the case would have made a dead endpoint live
with the rest intact:

- the admin's token was never revoked, so "stop impersonating" was a client promise;
- nothing was written to the audit log;
- the issued token carried no record of the original caller.

Now: `require_admin()`, self-impersonation refused, audit entry written before the
token is minted (a failed write means no token), the caller's `jti` revoked, and
`impersonator_id` in the response.

`stop_impersonation` was open to any authenticated caller, which made it a
token-minting endpoint with no audit trail. It is Admin-only now, and its response
says whose token it is — because after impersonation the caller's token is the
impersonated user's, and a client assuming otherwise would act as the wrong person.

`superadmin` is not a role the system has. `UserRole` is `Admin`, `Manager`,
`Worker`, `Viewer`, `Custom(String)`.

### J9 — the weather endpoint returned constants

`GET /api/v1/calculate/weather/fetch` parsed the requested provider into
`_service_type`, discarded it, and answered with a fixed 20.5 °C, 65 %, 0.0 mm,
12 km/h, 800 W/m², 1013.25 hPa, 18 °C, 45 %. The three providers in
`crates/weather-service` had no caller anywhere in the workspace, and a client
could not tell a real reading from a constant.

Now the provider is selected and called, an unknown name is a 400 rather than a
silent fallback, and a provider failure is reported as a failure rather than
papered over with plausible numbers.

This needed `crates/weather-service` to become a library: it was a binary crate
with no `lib.rs`, so the providers could not be called from the API at all. It is
now a `[lib]` plus `[[bin]]`.

### I5 — the LPIS cache existed and was never enabled

`LpisCache`, `LpisCache::new`, `CacheConfig` and `BaseClient::with_cache` all
existed and were correct. `BaseClient::new` and `BrpProvider::new` set
`cache: None`, and `create_default_registry` called only the constructors — so
nothing ever passed a cache and the `if let Some(cache)` branch in
`get_cached_or_fetch` was unreachable in every deployment. Every SIGPAC and BRP
listing went to the national WFS service on every call, on public rate-limited
endpoints, for data that does not change within the hour.

A synchronous `LpisCache::memory_if_enabled` was added because
`create_default_registry` is called from `AppState::new` and `LpisCache::new` is
async. It returns `None` for a disabled configuration and for a Redis backend,
rather than a cache that is written to but never read.

The fix was then widened: six further providers — RPG, iLPIS, SIAN, the German and
Polish LPIS services and INVEKOS — held a bare `reqwest::Client` and had no cache
field at all. They now share `cache::cached_get`, a narrow cached GET used at their
two fetch sites each. All eight registered providers receive the shared cache.

They were not routed through `BaseClient` on purpose: each carries its own error
type and status handling, and that would have meant rewriting six providers to
change one cache lookup. Cache keys are namespaced per provider, because a French
RPG parcel and a German LPIS parcel for the same bounding box would otherwise
collide on an identical key with different responses.

### G2i — a page can be unreachable without any test noticing (fixed)

The check for this was removed in 0.40.0 because it could not fail: it passed with
the `/water` route and its import deleted. It is back, and it does fail.

Two of the three causes are understood:

- **The embedding scan was a heuristic.** It collected every capitalised name that
  followed a `<` or a `(` anywhere under `admin-ui/src` and called it "rendered".
  That swept up 161 names — `Router`, `Routes`, `Icon`, `UserRole` — because
  `>Icon<` matches too. Every page looked embedded, so nothing was ever reported.
- **A component matched its own declaration.** The pattern `Name(` matched the
  `pub fn WaterManagement(` line in the very file that declares it. Same effect.

The third was the heuristic itself, so it is gone. Which components are embedded
rather than routed is now declared in `EMBEDDED_NOT_ROUTED`, each with a reason a
reviewer can check, and the two that are building blocks rather than pages are
recognised by `is_widget`. A hand-maintained list cannot rot the way a heuristic
does, because a new entry that is wrong reads as wrong.

Two assertions in the file were quietly weakened by the same bug and are fixed at
the same time:

- `sole_path` requires its input to be a trimmed, comma-terminated line, so passing
  it an offset substring silently returned `None` — and the route scan used it, so
  the first version of this test reported "no routes were parsed out of lib.rs".
  It now reads the literal with `literal_after`, as the rest of the file does.
- `#[test]` on a synchronous test in this file did not compile: `use actix_web::test`
  puts a module named `test` in scope, so the attribute resolved to Actix's
  async test macro and the error read "the async keyword is missing". The import is
  now `test as awtest`.

`the_page_reachability_check_can_still_fail` keeps the discrimination honest on
synthetic input, and the real check was verified against a broken tree by hand:
deleting the `/water` route reports `WaterManagement`; pointing a route at a
component that does not exist reports the route.

42 page components, 36 routes, 4 embedded, 2 widgets.
