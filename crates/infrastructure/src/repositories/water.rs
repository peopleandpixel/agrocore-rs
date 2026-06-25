use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::water::{
    WaterSource, WaterUsage, WaterQuota,
    CreateWaterSourceDto, CreateWaterUsageDto,
};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct WaterSourceRepo { base: MongoRepository<WaterSource> }
impl WaterSourceRepo {
    pub fn new(c: Collection<WaterSource>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterSource>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterSource>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateWaterSourceDto) -> Fut<WaterSource> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let source = WaterSource {
                id: Uuid::new_v4(),
                tenant_id: tid,
                site_id: dto.site_id,
                source_type: dto.source_type,
                name: dto.name,
                capacity_m3: dto.capacity_m3,
                current_level_m3: None,
                license_number: dto.license_number,
                license_expiry: dto.license_expiry,
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&source).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(source)
        })
    }
}

#[derive(Clone)]
pub struct WaterUsageRepo { base: MongoRepository<WaterUsage> }
impl WaterUsageRepo {
    pub fn new(c: Collection<WaterUsage>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterUsage>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterUsage>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string() };
            paginate(&c, f, p, Some(doc! { "usage_date": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateWaterUsageDto) -> Fut<WaterUsage> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let usage = WaterUsage {
                id: Uuid::new_v4(),
                tenant_id: tid,
                site_id: dto.site_id,
                source_id: dto.source_id,
                usage_date: dto.usage_date,
                volume_m3: dto.volume_m3,
                irrigation_method: dto.irrigation_method,
                efficiency_pct: dto.efficiency_pct,
                created_at: now,
            };
            c.insert_one(&usage).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(usage)
        })
    }
}

#[derive(Clone)]
pub struct WaterQuotaRepo { base: MongoRepository<WaterQuota> }
impl WaterQuotaRepo {
    pub fn new(c: Collection<WaterQuota>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WaterQuota>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WaterQuota>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string() };
            paginate(&c, f, p, Some(doc! { "year": -1 })).await
        })
    }
}
