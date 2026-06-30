use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::site::{CreateSiteDto, Site, UpdateSiteDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture, SiteRepository};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::base::{MongoRepository};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct SiteRepo { base: MongoRepository<Site> }
impl SiteRepo {
    pub fn new(c: Collection<Site>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn count(&self, tid: TenantId) -> Fut<u64> {
        let c=self.base.collection.clone();
        Box::pin(async move { c.count_documents(doc!{"tenant_id":tid.to_string()}).await.map_err(|e|SharedError::Database(e.to_string())) })
    }
}

impl SiteRepository for SiteRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>> {
        self.base.find_by_id(tid, id)
    }
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<Option<Site>> {
        self.base.find_by_id_visible(tid, id, user_id, roles)
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>> {
        self.base.find_all(tid, p)
    }
    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Site>> {
        self.base.find_all_visible(tid, p, user_id, roles)
    }
    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let s=Site{id:Uuid::new_v4(),tenant_id:tid,business_id:None,label:dto.label,site_type:dto.site_type,crop_type:dto.crop_type,variety:dto.variety,area:dto.area,gross_area:dto.gross_area,plots:dto.plots.unwrap_or_default(),row_config:dto.row_config,bbch_stage:dto.bbch_stage,planted_date:dto.planted_date,cleared_date:None,soil_type:dto.soil_type,slope:dto.slope,slope_facing:dto.slope_facing,altitude:dto.altitude,organic:dto.organic,organic_eligible:None,center:dto.center,sigpac_data:dto.sigpac_data,regepac_id:dto.regepac_id,boundary:dto.boundary,properties:dto.properties,custom_fields:dto.custom_fields,note1:dto.note1,note2:dto.note2,is_active:true,is_temporary:false,created_at:now,updated_at:now,created_by:Some(by),updated_by:Some(by)};
            c.insert_one(&s).await.map_err(|e|SharedError::Database(e.to_string()))?; Ok(s)
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateSiteDto, by: Uuid) -> RepositoryFuture<Option<Site>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.label{d.insert("label",v);}
            if let Some(v)=dto.area{d.insert("area",v);}
            if let Some(v)=dto.gross_area{d.insert("gross_area",v);}
            if let Some(v)=dto.properties{
                if let Ok(bson_v) = mongodb::bson::to_bson(&v) {
                    d.insert("properties", bson_v);
                }
            }
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Repository::find_by_id(&MongoRepository::new(c),tid,id).await;}
            d.insert("updated_at",Utc::now()); d.insert("updated_by",by.to_string());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Repository::find_by_id(&MongoRepository::new(c),tid,id).await
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }
}
