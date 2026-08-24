# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
