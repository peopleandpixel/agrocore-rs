# Changelog

Alle Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

## [Unreleased]

### Added
- Native PostGIS-Unterstützung für Geometriedaten (ersetzt manuelle WKT-Konvertierung).
- Neue Geometrie-Typen `SpatialGeometry` und `GeoPoint` mit direkter `sqlx`-Anbindung.
- `geozero` Crate zur effizienten Verarbeitung von räumlichen Daten.
- Umfassende Indizierung für Performance-Optimierung (z.B. auf `tenant_id`, `is_active`).
- Dynamische `updated_at` Trigger für alle Datenbanktabellen.
- `error_mapper` zur Umwandlung von SQL-Fehlern in Domänen-Fehler.
- Paginierung nach einheitlichem Standard über alle Repositories hinweg.

### Changed
- **Migration von MongoDB zu PostgreSQL abgeschlossen.**
- Sämtliche Services (`api`, `reporting`, `weather`, `geometry`, `asset-registry`) nutzen nun PostgreSQL als primäre Datenbank.
- Domänen-Modelle von MongoDB-spezifischen Attributen und Logiken bereinigt.
- `VisibilityAwareEntity` Trait vereinfacht (keine MongoDB-Filter mehr).
- Docker-Compose und Kubernetes Manifeste auf PostgreSQL/PostGIS umgestellt.
- Entwicklungs-Skripte (`dev.sh`, `dev-start.sh`) für PostgreSQL angepasst.

### Removed
- MongoDB-Container und zugehörige Abhängigkeiten aus dem gesamten Projekt entfernt.
- `mongodb` und `bson` Crates aus den Abhängigkeiten entfernt.
- Veraltete WKT-Konvertierungslogik in den Repositories.
