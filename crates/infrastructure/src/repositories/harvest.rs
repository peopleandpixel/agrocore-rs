use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::harvest::*;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct HarvestSeasonRepo { base: MongoRepository<HarvestSeason> }
impl HarvestSeasonRepo {
    pub fn new(c: Collection<HarvestSeason>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestSeason>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestSeason>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateHarvestSeasonDto) -> Fut<HarvestSeason> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let s = HarvestSeason {
                id: Uuid::new_v4(),
                tenant_id: tid,
                year: dto.year,
                label: dto.label,
                start_date: dto.start_date,
                end_date: dto.end_date,
                is_active: true,
                created_at: Utc::now(),
            };
            c.insert_one(&s).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(s)
        })
    }
}

#[derive(Clone)]
pub struct HarvestLotRepo { base: MongoRepository<HarvestLot> }
impl HarvestLotRepo {
    pub fn new(c: Collection<HarvestLot>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestLot>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<HarvestLot>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateHarvestLotDto) -> Fut<HarvestLot> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let lot = HarvestLot {
                id: Uuid::new_v4(),
                tenant_id: tid,
                season_id: dto.season_id,
                lot_number: dto.lot_number,
                site_ids: dto.site_ids,
                crop_type: dto.crop_type,
                variety: dto.variety,
                quality_target: dto.quality_target,
                total_weight_kg: 0.0,
                status: LotStatus::Collecting,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&lot).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(lot)
        })
    }
}

#[derive(Clone)]
pub struct HarvestDeliveryRepo { base: MongoRepository<HarvestDelivery> }
impl HarvestDeliveryRepo {
    pub fn new(c: Collection<HarvestDelivery>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<HarvestDelivery>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all_for_lot(&self, tid: TenantId, lot_id: Uuid, p: Pagination) -> Fut<PaginatedResponse<HarvestDelivery>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let filter = doc! { "tenant_id": tid.to_string(), "lot_id": lot_id.to_string() };
            paginate(&c, filter, p, Some(doc! { "delivery_date": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateHarvestDeliveryDto) -> Fut<HarvestDelivery> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let net = dto.gross_weight_kg - dto.tare_weight_kg;
            let d = HarvestDelivery {
                id: Uuid::new_v4(),
                tenant_id: tid,
                lot_id: dto.lot_id,
                delivery_date: dto.delivery_date,
                gross_weight_kg: dto.gross_weight_kg,
                net_weight_kg: net,
                tare_weight_kg: dto.tare_weight_kg,
                carrier_name: dto.carrier_name,
                vehicle_id: dto.vehicle_id,
                quality_notes: dto.quality_notes,
                temperature_at_delivery: dto.temperature_at_delivery,
            };
            c.insert_one(&d).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(d)
        })
    }
}

#[derive(Clone)]
pub struct ColdChainLogRepo { base: MongoRepository<ColdChainLog> }
impl ColdChainLogRepo {
    pub fn new(c: Collection<ColdChainLog>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_all_for_lot(&self, tid: TenantId, lot_id: Uuid, p: Pagination) -> Fut<PaginatedResponse<ColdChainLog>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let filter = doc! { "tenant_id": tid.to_string(), "lot_id": lot_id.to_string() };
            paginate(&c, filter, p, Some(doc! { "recorded_at": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateColdChainLogDto) -> Fut<ColdChainLog> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let log = ColdChainLog {
                id: Uuid::new_v4(),
                tenant_id: tid,
                lot_id: dto.lot_id,
                sensor_id: dto.sensor_id,
                recorded_at: dto.recorded_at,
                temperature_c: dto.temperature_c,
                humidity_pct: dto.humidity_pct,
                location: dto.location,
            };
            c.insert_one(&log).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(log)
        })
    }
}
