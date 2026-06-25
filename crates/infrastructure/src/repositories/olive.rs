use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::olive::{OliveGrove, OliveOilRecord, CreateOliveGroveDto, CreateOliveOilRecordDto};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct OliveGroveRepo { base: MongoRepository<OliveGrove> }
impl OliveGroveRepo {
    pub fn new(c: Collection<OliveGrove>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveGrove>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<OliveGrove>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateOliveGroveDto) -> Fut<OliveGrove> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let og=OliveGrove{id:Uuid::new_v4(),tenant_id:tid,site_id:dto.site_id,variety:dto.variety,tree_count:dto.tree_count,planting_year:dto.planting_year,organic_certified:dto.organic_certified,created_at:now,updated_at:now};
            c.insert_one(&og).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(og)
        })
    }
}

#[derive(Clone)]
pub struct OliveOilRecordRepo { base: MongoRepository<OliveOilRecord> }
impl OliveOilRecordRepo {
    pub fn new(c: Collection<OliveOilRecord>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<OliveOilRecord>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"created_at":-1})).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateOliveOilRecordDto) -> Fut<OliveOilRecord> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let rec=OliveOilRecord{id:Uuid::new_v4(),tenant_id:tid,grove_id:dto.grove_id,harvest_year:dto.harvest_year,oil_grade:dto.oil_grade,acidity_pct:dto.acidity_pct,peroxide_value:dto.peroxide_value,sensory_score:dto.sensory_score,liters_produced:dto.liters_produced,mill_name:dto.mill_name,lot_number:dto.lot_number,created_at:now};
            c.insert_one(&rec).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(rec)
        })
    }
}
