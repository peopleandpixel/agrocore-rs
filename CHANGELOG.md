# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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