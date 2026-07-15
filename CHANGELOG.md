# Changelog

Alle Änderungen an diesem Projekt werden in dieser Datei dokumentiert.

## [Unreleased]

## [0.3.0] - 2026-07-15

### Added
- **Benutzerverwaltung:** Vollständige CRUD-Funktionalität für Benutzer inklusive Rollenübersicht in der Admin-UI.
- **Audit-Log UI:** Neuer Bereich in der Admin-UI zur Anzeige der System-Audit-Logs (nur für Administratoren).
- **Navigation:** Konsistente Sidebar-Navigation mit Zugriff auf alle Systembereiche (Flächen, Equipment, Aufträge, Livestock, Finanzen, Compliance).

### Changed
- **Modus-Umschalter:** Der Wechsel zwischen "Einfachem" und "Normalem" Modus wurde als zentraler Switch in die Sidebar verschoben.
- **Flächen-Editor:** Verbesserte Kartenansicht beim Anlegen von Flächen mit größerem Dialogfenster (90% Viewport-Höhe) und wiederhergestellten Zeichenwerkzeugen.
- **RBAC:** Menüpunkte und Aktionen in der Admin-UI werden nun basierend auf den Benutzerrollen (Admin, Manager, Worker) gefiltert.

### Fixed
- **Infrastruktur:** Fehlende Imports in der PostgreSQL-Implementierung (`UpdateWorkLogDto`) behoben.
- **Admin-UI:** Diverse JavaScript-Fixes in der Leaflet-Integration zur Vermeidung von Initialisierungsfehlern.

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
