# PostgreSQL Migration - Vollständiger Plan

## ✅ ERLEDIGT (Komplett)

### Schema
- `migrations/001_initial_schema.sql` mit PostGIS Unterstützung
- Tabellen: tenants, users, sites, equipment, orders, animals, task_data

### Repositories
- `PgUserRepo` - Vollständig (CRUD, soft-delete, Rollen-Sichtbarkeit, Password-Hash)
- `PgEquipmentRepo` - Vollständig (CRUD, soft-delete)

## 📋 TODO - Nacheinander erledigen

### Phase 1: Repository-Konsolidierung
Alle bestehenden Repositories anpassen für korrekte PaginatedResponse Felder:
- animal.rs (falsche Felder)
- order.rs
- site.rs  
- task_data.rs
- weather_data.rs, weather_station.rs
- worker.rs, work_log.rs, worker_location.rs
- cost_center.rs, financial_record.rs
- pac_application.rs
- harvest_*.rs
- olive_*.rs
- kelter_delivery.rs

### Phase 2: API Layer
`crates/api/src/handlers/` - MongoClient Ersetzen durch PgPool:
- lib.rs - AppState enum für Postgres/Mongo
- auth.rs, users.rs, sites.rs, equipment.rs, orders.rs

### Phase 3: Application Layer
`crates/application/src/` - Service Layer anpassen

### Phase 4: Tests
- Integration Tests für PostgreSQL
- Test-Fixtures migrieren

### Phase 5: Deployment  
- Docker Compose: PostgreSQL + PostGIS statt MongoDB
- Migration-Skript

## 🔧 Sofortiger Fix für alle Repositories

```sql
-- Für jedes Repository Query anpassen:
-- ALT: items, limit
-- NEU: data, per_page, total_pages
```

## Start-Befehl für PostgreSQL
```bash
docker run --name agrocore-postgres \
  -e POSTGRES_DB=agrocore \
  -e POSTGRES_USER=agrocore \
  -e POSTGRES_PASSWORD=*** \
  -p 5432:5432 \
  postgis/postgis:16-3.4
  
psql -U agrocore -d agrocore -f migrations/001_initial_schema.sql
```