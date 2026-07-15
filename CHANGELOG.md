# Changelog

Alle Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

## [Unreleased]

## [0.2.0] - 2026-07-15

### Added
- Umfassendes Auditing-System: Jede Erstellung, Änderung und Löschung von Kern-Entitäten (Orders, PAC-Applications, Worker) wird nun automatisch protokolliert.
- Historien-Funktion mit Diff: Audit-Logs speichern den alten und neuen Zustand der Daten als JSON, was einen detaillierten Vergleich ermöglicht.
- `PgAuditLogRepo`: Vollständige Implementierung für persistente Speicherung und Abfrage von Audit-Einträgen.

### Changed
- Strikte Tenant-Isolation: Alle Datenbankabfragen validieren nun konsequent die `tenant_id`.
- RBAC-Erweiterung: Sichtbarkeitsprüfungen (`find_by_id_visible`) in Repositories berücksichtigen nun Benutzerrollen (Admin, Manager, Worker).
- Repository-Updates: `PgOrderRepo`, `PgPACApplicationRepo` und `PgWorkerRepo` nutzen nun das neue Auditing-System.
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
