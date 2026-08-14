# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  - Country dropdown (ES, NL, FR, PT, IT, DE, PL, AT) in Import component
  - `lpis_country` field added to `GeoJsonImportRequest` and `ShapefileImportRequest` DTOs
  - Updated import validation text to "Validate against LPIS (LPIS data)"
- **LPIS Settings API & UI**
  - `GET /api/v1/settings/lpis` - Load LPIS provider settings
  - `PUT /api/v1/settings/lpis` - Save LPIS provider settings (with restart_required flag)
  - `GET /api/v1/settings/lpis/providers` - List available providers with defaults
  - `LpisProviderConfig` DTO: base_url, timeout, cache_ttl, rate limits, enabled flag
  - Admin UI Settings page: Grid with 8 provider cards (Base URL, Timeout, Cache TTL, Rate Limit, Enabled)
- **Admin UI WASM Build Fixed**
  - `sqlx` made optional in `shared` crate with `sqlx` feature flag
  - `agrocore-shared` used with `features = []` in `admin-ui` → eliminates `mio`/`tokio` WASM incompatibility
  - Workspace-wide `sqlx` feature flags for consistent dependency management
- **Feature-flag architecture for sqlx** (workspace-consistent)
  - `shared`, `domain`, `infrastructure`, `api`: `sqlx` optional + `sqlx` feature
  - `admin-ui`: uses `shared` **without** `sqlx` feature → WASM-compatible
  - `sqlx = ["dep:sqlx", "agrocore-shared/sqlx"]` pattern across crates

### Fixed
- `lpis-providers` clippy warnings: collapsible_if, redundant closures
- `shared` clippy: single-component path imports with allow attribute
- `admin-ui` clippy: useless_vec, unused variables/imports
- `api` error handling: `From<sqlx::Error>` gated behind `sqlx` feature
- `settings` handler: correct `ServiceConfig` signature, removed unused imports

### Changed
- Updated workspace version to 0.5.9
- `agrocore-shared`: `sqlx` now optional, gated behind `sqlx` feature
- `agrocore-admin-ui`: uses `shared` without default features (no sqlx)

## [0.5.8] - 2026-07-31

### Added
- GeoJSON and Shapefile import functionality with LPIS validation against SIGPAC reference data
- Import options: skip duplicates, update existing, validate against SIGPAC
- Import result display with statistics (total, created, updated, skipped) and error/warning reporting
- Base64 encoding for Shapefile ZIP uploads using `js_sys::Uint8Array`
- DataImport component with file selection, preview, and validation
### Added (Multi-country LPIS support)
- LPIS abstraction layer with `LpisProvider` trait and `LpisRegistry`
- Support for 8 European countries:
  - NL: BRP (Netherlands) - PDOK WFS
  - ES: SIGPAC (Spain) - FEGA/regional WFS
  - FR: RPG (France) - IGN Geoservices WFS
  - PT: iLPIS (Portugal) - IFAP WFS
  - IT: SIAN (Italy) - AGEA/regional WFS
  - DE: LPIS (Germany) - State-level WFS
  - PL: LPIS (Poland) - ARiMR WFS
  - AT: INVEKOS (Austria) - AMA/data.gv.at WFS
- Generic `LpisData` structure in domain for all LPIS implementations
- Deprecated `regepac_id` in favor of `lpis_data.reference`

### Fixed
- Fixed admin-ui edition upgraded to 2024 for async move blocks and let chains
- Fixed clippy warnings: collapsed nested if statements using let chains
- Fixed type inference in conditional view rendering using `.into_any()` for branch type erasure
- Fixed reqwest version conflict (0.12 -> 0.13 with rustls features)
- Added missing API types: GeoJsonImportRequest, ShapefileImportRequest, ImportResult
- Fixed missing web-sys features: FileList, HtmlInputElement, EventTarget
- Fixed clippy warnings in LPIS providers: `manual_is_multiple_of`, `collapsible_if`

### Changed
- Updated workspace version to 0.5.8

## [0.5.7] - 2026-07-31

### Added
- SIGPAC parcel detail modal with full field display (province, municipality, aggregate, zone, polygon, parcel, enclosure, usage code, area hectares, official area, source dataset/year, geometry)
- Spatial search by coordinates with configurable radius
- Near point search endpoint (`search_parcels_near_point`) in API handlers
- Complete SIGPAC CRUD operations: `list_sigpac_parcels`, `get_sigpac_parcel`, list all handler endpoints
- Admin UI SIGPAC module (`SigpacParcels` component) with filters, pagination, detail view, and spatial search

### Changed
- Updated workspace version to 0.5.7
- Fixed admin-ui compilation errors (restored api.rs, fixed sigpac.rs structural issues, added Missing SIGPAC type references)

## [0.5.6] - 2026-07-28

### Added
- SIGPAC reference data import from official Spanish fiboa GeoParquet files (source.coop/fiboa)
- Support for 15 Spanish autonomous regions (~25M parcels total)
- Python import script with batch processing, upsert, and progress tracking
- Fiboa field mapping: admin_province_code, admin_municipality_code, crop:code, crop:name, geometry
- SIGPAC reference generation (20-digit: PPMMMAAAZZZPPPPEEE)
- SIGPAC data sources: Andalusia, Aragon, Catalonia, Castile & León, Navarre, Basque Country, Castile-La Mancha, Valencia, Galicia, Extremadura, Madrid, Murcia, Balearic Islands, Canary Islands, Cantabria, La Rioja

### Changed
- Updated workspace version to 0.5.6

## [0.5.5] - 2026-07-XX

### Added
- Phase 3.6: Mobile/PWA Admin UI completion
- Task management and API endpoint verification

## [0.5.4] - 2026-07-XX

### Added
- Phase 3.5: Farm operations API
- Weather service integration
