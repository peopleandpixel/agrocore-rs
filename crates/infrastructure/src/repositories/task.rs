use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::task::{CreateTaskDataDto, TaskData};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::base::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct TaskDataRepo { base: MongoRepository<TaskData> }
impl TaskDataRepo {
    pub fn new(c: Collection<TaskData>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<TaskData>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_by_order(&self, tid: TenantId, oid: Uuid) -> Fut<Vec<TaskData>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"order_id":oid.to_string()};
            let opts=FindOptions::builder().sort(doc!{"started_at":-1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Database(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Database(e.to_string()))?);} Ok(d)
        })
    }
    pub fn find_by_worker(&self, tid: TenantId, wid: Uuid, p: Pagination) -> RepositoryFuture<PaginatedResponse<TaskData>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"worker_id":wid.to_string()};
            let sort=doc!{"started_at":-1};
            paginate(&c, f, p, Some(sort)).await
        })
    }
    pub fn create(&self, tid: TenantId, wid: Uuid, dto: CreateTaskDataDto) -> Fut<TaskData> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let td=TaskData{id:Uuid::new_v4(),tenant_id:tid,order_id:dto.order_id,worker_id:wid,site_id:dto.site_id,description:dto.description,started_at:dto.started_at.unwrap_or(now),ended_at:dto.ended_at,duration_minutes:dto.duration_minutes,machine_id:dto.machine_id,machine_hours:dto.machine_hours,cost_center_id:dto.cost_center_id,area_covered:dto.area_covered,materials_used:dto.materials_used,observations:dto.observations,gps_track:dto.gps_track,photo_urls:dto.photo_urls,created_at:now,updated_at:now};
            c.insert_one(&td).await.map_err(|e|SharedError::Database(e.to_string()))?; Ok(td)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: CreateTaskDataDto) -> Fut<Option<TaskData>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let d=doc!{"description":dto.description,"ended_at":dto.ended_at,"duration_minutes":dto.duration_minutes,"machine_id":dto.machine_id.map(|u|u.to_string()),"machine_hours":dto.machine_hours,"cost_center_id":dto.cost_center_id.map(|u|u.to_string()),"area_covered":dto.area_covered,"observations":dto.observations,"updated_at":Utc::now()};
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Repository::find_by_id(&MongoRepository::new(c),tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let c=self.base.collection.clone();
        Box::pin(async move { let r=c.delete_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Database(e.to_string()))?; Ok(r.deleted_count>0) })
    }
}
