use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::order::{CreateOrderDto, MyTask, Order, UpdateOrderDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::base::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct OrderRepo { base: MongoRepository<Order> }
impl OrderRepo {
    pub fn new(c: Collection<Order>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Order>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<Option<Order>> {
        self.base.find_by_id_visible(tid, id, user_id, roles)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Order>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            let sort=doc!{"deadline_date":1};
            paginate(&c, f, p, Some(sort)).await
        })
    }
    pub fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Order>> {
        self.base.find_all_visible(tid, p, user_id, roles)
    }
    pub fn find_my_tasks(&self, tid: TenantId, wid: Uuid) -> Fut<Vec<MyTask>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"assigned_worker_ids":wid.to_string(),"status":{"$ne":"completed"}};
            let opts=FindOptions::builder().sort(doc!{"deadline_date":1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Database(e.to_string()))?;
            let mut tasks=Vec::new();
            while let Some(r)=cur.next().await{let o=r.map_err(|e|SharedError::Database(e.to_string()))?;
                tasks.push(MyTask{order_id:o.id,label:o.label,order_type:o.order_type,status:o.status,site_count:o.site_ids.len() as u32,total_area:0.0,deadline_date:o.deadline_date,planned_date:o.planned_date});}
            Ok(tasks)
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateOrderDto, by: Uuid) -> Fut<Order> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let o=Order{id:Uuid::new_v4(),tenant_id:tid,label:dto.label,order_type:dto.order_type,status:agrocore_domain::entities::OrderStatus::Draft,site_ids:dto.site_ids,assigned_worker_ids:dto.assigned_worker_ids.unwrap_or_default(),planned_date:dto.planned_date,deadline_date:dto.deadline_date,started_at:None,completed_at:None,articles:dto.articles,quantities:dto.quantities,results:None,weather:None,custom_fields:dto.custom_fields,parent_order_id:dto.parent_order_id,workflow_config:dto.workflow_config,cost_center_id:dto.cost_center_id,is_active:true,created_at:now,updated_at:now,created_by:Some(by),updated_by:Some(by)};
            c.insert_one(&o).await.map_err(|e|SharedError::Database(e.to_string()))?; Ok(o)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOrderDto, by: Uuid) -> Fut<Option<Order>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.label{d.insert("label",v);}
            if let Some(v)=dto.status{d.insert("status",mongodb::bson::to_bson(&v).unwrap());}
            if let Some(v)=dto.results{d.insert("results",v);}
            if let Some(v)=dto.cost_center_id{d.insert("cost_center_id",v.to_string());}
            if let Some(v)=dto.workflow_config{d.insert("workflow_config",mongodb::bson::to_bson(&v).unwrap());}
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Repository::find_by_id(&MongoRepository::new(c),tid,id).await;}
            d.insert("updated_at",Utc::now()); d.insert("updated_by",by.to_string());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Repository::find_by_id(&MongoRepository::new(c),tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }
}
