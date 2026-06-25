use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::vineyard::{Vineyard, KelterDelivery, CreateVineyardDto, CreateKelterDeliveryDto};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct VineyardRepo { base: MongoRepository<Vineyard> }
impl VineyardRepo {
    pub fn new(c: Collection<Vineyard>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Vineyard>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Vineyard>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateVineyardDto) -> Fut<Vineyard> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let v=Vineyard{id:Uuid::new_v4(),tenant_id:tid,site_id:dto.site_id,doc_area:dto.doc_area,vintage:dto.vintage,grape_variety:dto.grape_variety,brix_at_harvest:dto.brix_at_harvest,ph_at_harvest:dto.ph_at_harvest,acidity:dto.acidity,yield_tons:dto.yield_tons,quality_grade:dto.quality_grade,kelter_delivery:None,created_at:now,updated_at:now};
            c.insert_one(&v).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(v)
        })
    }
}

#[derive(Clone)]
pub struct KelterDeliveryRepo { base: MongoRepository<KelterDelivery> }
impl KelterDeliveryRepo {
    pub fn new(c: Collection<KelterDelivery>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<KelterDelivery>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<KelterDelivery>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"delivery_date":-1})).await
        })
    }
    pub fn create(&self, _tid: TenantId, dto: CreateKelterDeliveryDto) -> Fut<KelterDelivery> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let _now=Utc::now();
            let kd=KelterDelivery{id:Uuid::new_v4(),vineyard_id:dto.vineyard_id,delivery_date:dto.delivery_date,gross_weight_kg:dto.gross_weight_kg,net_weight_kg:dto.net_weight_kg,lot_number:dto.lot_number,kelter_name:dto.kelter_name,transport_company:dto.transport_company,temperature_c:dto.temperature_c,notes:dto.notes};
            c.insert_one(&kd).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(kd)
        })
    }
}
