### Zusammenfassung der Optimierungsschritte

Um die PostgreSQL-Migration zu professionalisieren und die Performance sowie Wartbarkeit des Projekts zu steigern, wurden folgende Schritte durchgeführt:

#### 1. Verfeinerung des Daten-Schemas (Migrations)
- [x] **Normalisierung von Relationen**: Ersetzen Sie `JSONB`-Spalten für Entitäts-Verknüpfungen (z. B. `assigned_site_ids` in `users`) durch klassische n:m Verknüpfungstabellen, um die Abfrageperformance zu verbessern.
    - *Erledigt*: Neue Migration `20240715_optimization.sql` erstellt. Die Junction-Table `user_sites` wurde eingeführt, um `assigned_site_ids` aus der `users`-Tabelle zu normalisieren.
- [x] **Vervollständigung der Audit-Trigger**: Wenden Sie den `set_updated_at`-Trigger auf **alle** Tabellen an, um eine konsistente Nachverfolgung von Änderungen sicherzustellen.
    - *Erledigt*: Ein dynamisches PL/pgSQL-Skript in der Migration fügt den `set_updated_at`-Trigger automatisch zu allen Tabellen hinzu, die eine `updated_at`-Spalte besitzen.
- [x] **Constraint-Optimierung**: Fügen Sie explizite `UNIQUE`-Constraints und `NOT NULL`-Checks dort hinzu, wo sie im aktuellen Entwurf noch fehlen (z. B. `tag_number` bei Tieren, falls diese eindeutig sein muss).
    - *Erledigt*: `UNIQUE`-Constraints (z. B. für `animals.tag_number` pro Tenant) wurden in der Migrationsdatei ergänzt.

#### 2. Modernisierung des Geometrie-Handlings
- [x] **Native PostGIS-Anbindung**: Ersetzen Sie die manuelle WKT-String-Konvertierung (`boundary_to_wkt`) durch die Integration von Crates wie `geozero` oder nutzen Sie den direkten Support von `sqlx` für binäre PostGIS-Typen.
    - *Erledigt*: `geozero` integriert und `sqlx` Traits (`Type`, `Encode`, `Decode`) für `SpatialGeometry` und `GeoPoint` implementiert. Repositories nutzen nun binäres PostGIS-Mapping.
- [x] **Typsicherheit**: Definieren Sie spezialisierte Geo-Typen im Rust-Code, die direkt auf die Datenbank-Spalten gemappt werden können, um Parse-Fehler zur Laufzeit zu vermeiden.
    - *Erledigt*: Domänen-Typen sind nun direkt kompatibel mit PostGIS `geometry` Spalten. `sqlx::FromRow` nutzt nativ das binäre Format.

#### 3. Verbesserung der Repository-Implementierung
- [x] **Differenzierte Fehlerbehandlung**: Mappen Sie SQL-Fehler (wie `UniqueViolation` oder `ForeignKeyViolation`) auf spezifische Domain-Fehler (`AlreadyExists`, `ValidationFailed`), anstatt alles als `Internal Server Error` auszugeben.
    - *Erledigt*: `error_mapper.rs` in der Infrastruktur implementiert. SQL-Fehlercodes (wie 23505 für Unique Violations) werden nun auf `SharedError::AlreadyExists` gemappt und in der API mit korrektem Statuscode (409 Conflict) ausgegeben.
- [x] **Batching & JOINs**: Optimieren Sie Abfragen von verschachtelten Objekten (z. B. Sites mit ihren Plots) durch SQL-Joins oder `JSON_AGG`, um die Anzahl der Datenbank-Roundtrips (N+1 Problem) zu minimieren.
    - *Erledigt*: In `PgUserRepo` werden die verknüpften Sites nun mittels `json_agg` in einem einzigen Query geladen, was das N+1 Problem für diese Relation löst.

#### 4. Infrastruktur & Tooling
- [x] **Versions-Management**: Führen Sie ein Tool für versionierte Migrationen ein (z. B. `sqlx-cli`), um Schema-Änderungen nachvollziehbar und reproduzierbar zu machen.
    - *Erledigt*: Das Projekt nutzt nun die standardisierte `sqlx`-Migrationsstruktur mit Zeitstempeln im Dateinamen.
- [x] **Indizierung**: Überprüfen Sie die Abfrageprofile der API und ergänzen Sie Indizes für häufig gefilterte Spalten (z. B. `is_active` in Kombination mit `tenant_id`).
    - *Erledigt*: Performance-Indizes für `users`, `sites` und `orders` (jeweils auf `is_active`/`status` und `tenant_id`) wurden in der Migration hinzugefügt.

#### 5. Konsolidierung der Paginierung
- [x] **Einheitliche API-Struktur**: Schließen Sie die Umstellung aller Repositories auf das Format `{ data, total, page, per_page, total_pages }` ab, um ein konsistentes Verhalten der API für das Frontend sicherzustellen.
    - *Erledigt*: Das `PaginatedResponse`-DTO wird nun konsistent in den Repositories (z. B. `PgUserRepo`) verwendet.