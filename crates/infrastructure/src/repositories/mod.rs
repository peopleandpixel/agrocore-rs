use std::future::Future;
use std::pin::Pin;
use bson::{doc, Document};
use chrono::Utc;
use futures::StreamExt;
use mongodb::options::FindOptions;
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::order::{CreateOrderDto, MyTask, Order, UpdateOrderDto};
use agrocore_domain::entities::site::{CreateSiteDto, Site, UpdateSiteDto};
use agrocore_domain::entities::task::{CreateTaskDataDto, TaskData};
use agrocore_domain::entities::user::{AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User, UserRole};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct SiteRepo { c: Collection<Site> }
impl SiteRepo {
    pub fn new(c: Collection<Site>) -> Self { Self { c } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<Site>> {
        let c = self.c.clone();
        Box::pin(async move { c.find_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
    pub fn find_all(&self, tid: TenantId, p: &Pagination) -> Fut<PaginatedResponse<Site>> {
        let c = self.c.clone(); let page=p.page.unwrap_or(0); let pp=p.per_page.unwrap_or(20);
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()}; let t=c.count_documents(f.clone()).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let opts=FindOptions::builder().skip(page*pp).limit(pp as i64).sort(doc!{"updated_at":-1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Internal(e.to_string()))?);}
            Ok(PaginatedResponse{data:d,total:t,page,per_page:pp,total_pages:(t as f64/pp as f64).ceil() as u64})
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> Fut<Site> {
        let c=self.c.clone();
        Box::pin(async move {
            let now=Utc::now();
            let s=Site{id:Uuid::new_v4(),tenant_id:tid,business_id:None,label:dto.label,site_type:dto.site_type,crop_type:dto.crop_type,variety:dto.variety,area:dto.area,plots:dto.plots.unwrap_or_default(),row_config:dto.row_config,bbch_stage:dto.bbch_stage,planted_date:dto.planted_date,cleared_date:None,soil_type:dto.soil_type,slope:dto.slope,slope_facing:dto.slope_facing,altitude:dto.altitude,organic:dto.organic,organic_eligible:None,center:dto.center,boundary:dto.boundary,custom_fields:dto.custom_fields,note1:dto.note1,note2:dto.note2,is_active:true,is_temporary:false,created_at:now,updated_at:now,created_by:Some(by),updated_by:Some(by)};
            c.insert_one(&s).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(s)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateSiteDto, by: Uuid) -> Fut<Option<Site>> {
        let c=self.c.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.label{d.insert("label",v);}
            if let Some(v)=dto.area{d.insert("area",v);}
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Self::find_by_id(&Self{c},tid,id).await;}
            d.insert("updated_at",Utc::now()); d.insert("updated_by",by.to_string());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            Self::find_by_id(&Self{c},tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let c=self.c.clone();
        Box::pin(async move { let r=c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":{"is_active":false,"updated_at":Utc::now()}}).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(r.modified_count>0) })
    }
    pub fn count(&self, tid: TenantId) -> Fut<u64> {
        let c=self.c.clone();
        Box::pin(async move { c.count_documents(doc!{"tenant_id":tid.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
}

#[derive(Clone)]
pub struct OrderRepo { c: Collection<Order> }
impl OrderRepo {
    pub fn new(c: Collection<Order>) -> Self { Self { c } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<Order>> {
        let c=self.c.clone(); Box::pin(async move { c.find_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
    pub fn find_all(&self, tid: TenantId, p: &Pagination) -> Fut<PaginatedResponse<Order>> {
        let c=self.c.clone(); let page=p.page.unwrap_or(0); let pp=p.per_page.unwrap_or(20);
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()}; let t=c.count_documents(f.clone()).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let opts=FindOptions::builder().skip(page*pp).limit(pp as i64).sort(doc!{"deadline_date":1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Internal(e.to_string()))?);}
            Ok(PaginatedResponse{data:d,total:t,page,per_page:pp,total_pages:(t as f64/pp as f64).ceil() as u64})
        })
    }
    pub fn find_my_tasks(&self, tid: TenantId, wid: Uuid) -> Fut<Vec<MyTask>> {
        let c=self.c.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"assigned_worker_ids":wid.to_string(),"status":{"$ne":"completed"}};
            let opts=FindOptions::builder().sort(doc!{"deadline_date":1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut tasks=Vec::new();
            while let Some(r)=cur.next().await{let o=r.map_err(|e|SharedError::Internal(e.to_string()))?;
                tasks.push(MyTask{order_id:o.id,label:o.label,order_type:o.order_type,status:o.status,site_count:o.site_ids.len() as u32,total_area:0.0,deadline_date:o.deadline_date,planned_date:o.planned_date});}
            Ok(tasks)
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateOrderDto, by: Uuid) -> Fut<Order> {
        let c=self.c.clone();
        Box::pin(async move {
            let now=Utc::now();
            let o=Order{id:Uuid::new_v4(),tenant_id:tid,label:dto.label,order_type:dto.order_type,status:agrocore_domain::entities::OrderStatus::Draft,site_ids:dto.site_ids,assigned_worker_ids:dto.assigned_worker_ids.unwrap_or_default(),planned_date:dto.planned_date,deadline_date:dto.deadline_date,started_at:None,completed_at:None,articles:dto.articles,quantities:dto.quantities,results:None,weather:None,custom_fields:dto.custom_fields,is_active:true,created_at:now,updated_at:now,created_by:Some(by),updated_by:Some(by)};
            c.insert_one(&o).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(o)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateOrderDto, by: Uuid) -> Fut<Option<Order>> {
        let c=self.c.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.label{d.insert("label",v);}
            if let Some(v)=dto.status{d.insert("status",bson::to_bson(&v).unwrap());}
            if let Some(v)=dto.results{d.insert("results",v);}
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Self::find_by_id(&Self{c},tid,id).await;}
            d.insert("updated_at",Utc::now()); d.insert("updated_by",by.to_string());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            Self::find_by_id(&Self{c},tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let c=self.c.clone();
        Box::pin(async move { let r=c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":{"is_active":false,"updated_at":Utc::now()}}).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(r.modified_count>0) })
    }
}

#[derive(Clone)]
pub struct UserRepo { c: Collection<User> }
impl UserRepo {
    pub fn new(c: Collection<User>) -> Self { Self { c } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<User>> {
        let c=self.c.clone(); Box::pin(async move { c.find_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
    pub fn find_by_email(&self, email: &str) -> Fut<Option<User>> {
        let c=self.c.clone(); let email=email.to_string(); Box::pin(async move { c.find_one(doc!{"email":&email}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
    pub fn find_all(&self, tid: TenantId, p: &Pagination) -> Fut<PaginatedResponse<User>> {
        let c=self.c.clone(); let page=p.page.unwrap_or(0); let pp=p.per_page.unwrap_or(20);
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()}; let t=c.count_documents(f.clone()).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let opts=FindOptions::builder().skip(page*pp).limit(pp as i64).sort(doc!{"lastname":1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Internal(e.to_string()))?);}
            Ok(PaginatedResponse{data:d,total:t,page,per_page:pp,total_pages:(t as f64/pp as f64).ceil() as u64})
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateUserDto) -> Fut<User> {
        let c=self.c.clone();
        Box::pin(async move {
            let now=Utc::now(); let pw=hash_password(&dto.password)?;
            let u=User{id:Uuid::new_v4(),tenant_id:tid,firstname:dto.firstname,lastname:dto.lastname,email:dto.email,password_hash:pw,roles:dto.roles.unwrap_or_else(||vec![UserRole::Worker]),is_active:true,internal_cost_per_hour:dto.internal_cost_per_hour,external_cost_per_hour:dto.external_cost_per_hour,color:None,language:dto.language,assigned_site_ids:None,last_login:None,created_at:now,updated_at:now};
            c.insert_one(&u).await.map_err(|e|if e.to_string().contains("duplicate key"){SharedError::Conflict("Email already exists".into())}else{SharedError::Internal(e.to_string())})?; Ok(u)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateUserDto) -> Fut<Option<User>> {
        let c=self.c.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.firstname{d.insert("firstname",v);}
            if let Some(v)=dto.lastname{d.insert("lastname",v);}
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Self::find_by_id(&Self{c},tid,id).await;}
            d.insert("updated_at",Utc::now());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            Self::find_by_id(&Self{c},tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let c=self.c.clone();
        Box::pin(async move { let r=c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":{"is_active":false,"updated_at":Utc::now()}}).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(r.modified_count>0) })
    }
    pub fn authenticate(&self, dto: LoginDto) -> Fut<AuthResponse> {
        let c=self.c.clone();
        Box::pin(async move {
            let u=c.find_one(doc!{"email":&dto.email}).await.map_err(|e|SharedError::Internal(e.to_string()))?.ok_or_else(||SharedError::Unauthorized("Invalid credentials".into()))?;
            if !u.is_active{return Err(SharedError::Unauthorized("Account disabled".into()));}
            verify_password(&dto.password,&u.password_hash)?;
            let token=generate_jwt(&u)?;
            Ok(AuthResponse{token,user_id:u.id,tenant_id:u.tenant_id,firstname:u.firstname,lastname:u.lastname,roles:u.roles})
        })
    }
}

#[derive(Clone)]
pub struct TaskDataRepo { c: Collection<TaskData> }
impl TaskDataRepo {
    pub fn new(c: Collection<TaskData>) -> Self { Self { c } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<TaskData>> {
        let c=self.c.clone(); Box::pin(async move { c.find_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string())) })
    }
    pub fn find_by_order(&self, tid: TenantId, oid: Uuid) -> Fut<Vec<TaskData>> {
        let c=self.c.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"order_id":oid.to_string()};
            let opts=FindOptions::builder().sort(doc!{"started_at":-1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Internal(e.to_string()))?);} Ok(d)
        })
    }
    pub fn find_by_worker(&self, tid: TenantId, wid: Uuid, p: &Pagination) -> Fut<PaginatedResponse<TaskData>> {
        let c=self.c.clone(); let page=p.page.unwrap_or(0); let pp=p.per_page.unwrap_or(20);
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string(),"worker_id":wid.to_string()};
            let t=c.count_documents(f.clone()).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let opts=FindOptions::builder().skip(page*pp).limit(pp as i64).sort(doc!{"started_at":-1}).build();
            let mut cur=c.find(f).with_options(opts).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            let mut d=Vec::new(); while let Some(r)=cur.next().await{d.push(r.map_err(|e|SharedError::Internal(e.to_string()))?);}
            Ok(PaginatedResponse{data:d,total:t,page,per_page:pp,total_pages:(t as f64/pp as f64).ceil() as u64})
        })
    }
    pub fn create(&self, tid: TenantId, wid: Uuid, dto: CreateTaskDataDto) -> Fut<TaskData> {
        let c=self.c.clone();
        Box::pin(async move {
            let now=Utc::now();
            let td=TaskData{id:Uuid::new_v4(),tenant_id:tid,order_id:dto.order_id,worker_id:wid,site_id:dto.site_id,description:dto.description,started_at:dto.started_at.unwrap_or(now),ended_at:dto.ended_at,duration_minutes:dto.duration_minutes,area_covered:dto.area_covered,materials_used:dto.materials_used,observations:dto.observations,gps_track:dto.gps_track,photo_urls:dto.photo_urls,created_at:now,updated_at:now};
            c.insert_one(&td).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(td)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: CreateTaskDataDto) -> Fut<Option<TaskData>> {
        let c=self.c.clone();
        Box::pin(async move {
            let d=doc!{"description":dto.description,"ended_at":dto.ended_at,"duration_minutes":dto.duration_minutes,"area_covered":dto.area_covered,"observations":dto.observations,"updated_at":Utc::now()};
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Internal(e.to_string()))?;
            Self::find_by_id(&Self{c},tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let c=self.c.clone();
        Box::pin(async move { let r=c.delete_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()}).await.map_err(|e|SharedError::Internal(e.to_string()))?; Ok(r.deleted_count>0) })
    }
}

fn hash_password(pw: &str) -> Result<String> {
    use argon2::PasswordHasher;
    argon2::Argon2::default().hash_password(pw.as_bytes()).map(|h|h.to_string()).map_err(|e|SharedError::Internal(e.to_string()))
}
fn verify_password(pw: &str, hash: &str) -> Result<()> {
    use argon2::PasswordHash;
    use argon2::PasswordVerifier;
    let parsed=PasswordHash::new(hash).map_err(|e|SharedError::Internal(e.to_string()))?;
    argon2::Argon2::default().verify_password(pw.as_bytes(),&parsed).map_err(|_|SharedError::Unauthorized("Invalid credentials".into()))
}
fn generate_jwt(u: &User) -> Result<String> {
    use jsonwebtoken::{encode,EncodingKey,Header}; use serde::Serialize;
    #[derive(Serialize)] struct Claims{sub:String,tenant_id:String,exp:u64}
    let exp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()+86400;
    let claims=Claims{sub:u.id.to_string(),tenant_id:u.tenant_id.to_string(),exp};
    let secret=std::env::var("JWT_SECRET").unwrap_or_else(|_|"dev-secret".into());
    encode(&Header::default(),&claims,&EncodingKey::from_secret(secret.as_bytes())).map_err(|e|SharedError::Internal(e.to_string()))
}
