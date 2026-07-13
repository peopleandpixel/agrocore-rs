# PostgreSQL Migration Checklist

## Repositories zu migrieren (nach Core-Repos)

### Domain Repositories (aus lib.rs exports)
- [x] SiteRepo → PgSiteRepo
- [x] UserRepo → PgUserRepo  
- [x] OrderRepo → PgOrderRepo
- [x] TenantRepo → PgTenantRepo
- [x] EquipmentRepo → PgEquipmentRepo

### Remaining:
- [ ] AnimalRepo
- [ ] FertilizerRecordRepo
- [ ] PlantProtectionRecordRepo
- [ ] ComplianceChecklistRepo
- [ ] WeatherStationRepo
- [ ] WeatherDataRepo
- [ ] PhenologyRecordRepo
- [ ] HarvestSeasonRepo
- [ ] HarvestLotRepo
- [ ] HarvestDeliveryRepo
- [ ] WaterSourceRepo
- [ ] WaterUsageRepo
- [ ] WaterQuotaRepo
- [ ] WorkLogRepo
- [ ] WorkerRepo
- [ ] WorkerLocationRepo
- [ ] WorkerTaskStatusRepo
- [ ] SpatialObjectRepo
- [ ] TaskDataRepo
- [ ] AuditLogRepo
- [ ] CostCenterRepo
- [ ] FinancialRecordRepo
- [ ] PACApplicationRepo
- [ ] KelterDeliveryRepo
- [ ] VineyardRepo
- [ ] OliveGroveRepo
- [ ] OliveOilRecordRepo
- [ ] ColdChainLogRepo

## Schritte für jede Repository
1. `crates/infrastructure/src/postgres/{name}.rs` erstellen
2. Trait aus `domain/src/repositories.rs` implementieren
3. In `postgres.rs` mod hinzufügen
4. In `postgres/database.rs` export hinzufügen
5. In `lib.rs` `#[cfg(feature = "postgres")]` export hinzufügen

## Query Patterns

### Standard CRUD
```rust
sqlx::query_as("SELECT * FROM table WHERE tenant_id = $1 AND id = $2")
    .bind(tid.to_string())
    .bind(id)
    .fetch_optional(&pool)
    .await
```

### JSONB Spalten
```rust
.bind(serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string()))
```

### PostGIS Spatial
```rust
// GeoJSON zu Geometry
CASE WHEN $1 IS NOT NULL THEN ST_GeomFromGeoJSON($1::text) ELSE NULL END

// Point in Polygon Check
ST_Contains(polygon_col, ST_Point($lng, $lat))
```