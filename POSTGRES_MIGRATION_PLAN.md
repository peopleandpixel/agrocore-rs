# PostgreSQL Migration Plan für agrocore-rs

## Ziel
Vollständiger Wechsel von MongoDB zu PostgreSQL + PostGIS für bessere Geospatial Performance.

## Phase 1: Schema Definition (Heute)
- [ ] PostGIS Tabellen für alle Entitäten
- [ ] Migrations-Skript erstellen
- [ ] Docker Compose anpassen

## Phase 2: Infrastructure Layer (Tag 1-2)
- [ ] Neue `postgres.rs` Module
- [ ] Connection Pool Setup
- [ ] Repository Traits implementieren

## Phase 3: Geospatial Refactor (Tag 1)
- [ ] `PolygonGeometry` ↔ `GEOMETRY(POLYGON, 4326)` Mapping
- [ ] Query-Logik anpassen

## Phase 4: Tests (Tag 2)
- [ ] Test-Fixtures migrieren
- [ ] Integration Tests anpassen

## Phase 5: Deployment (Tag 3)
- [ ] CI/CD anpassen
- [ ] Rollback-Strategie

## Datenbank URL: 
`postgresql://agrocore:password@localhost:5432/agrocore`

## Starte mit PostgreSQL Schema Creation