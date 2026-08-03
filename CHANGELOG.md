# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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