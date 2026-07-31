# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[... earlier versions omitted ...]