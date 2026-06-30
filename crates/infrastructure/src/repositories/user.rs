use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::user::{AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User, UserRole};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::base::{MongoRepository, paginate};
use crate::repositories::auth_utils::{hash_password, verify_password, generate_jwt};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct UserRepo { base: MongoRepository<User> }
impl UserRepo {
    pub fn new(c: Collection<User>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_by_email(&self, email: &str) -> Fut<Option<User>> {
        let c=self.base.collection.clone(); let email=email.to_string(); Box::pin(async move { c.find_one(doc!{"email":&email}).await.map_err(|e|SharedError::Database(e.to_string())) })
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            let sort=doc!{"lastname":1};
            paginate(&c, f, p, Some(sort)).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateUserDto) -> Fut<User> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now(); let pw=hash_password(&dto.password)?;
            let u=User{id:Uuid::new_v4(),tenant_id:tid,firstname:dto.firstname,lastname:dto.lastname,email:dto.email,password_hash:pw,roles:dto.roles.unwrap_or_else(||vec![UserRole::Worker]),is_active:true,internal_cost_per_hour:dto.internal_cost_per_hour,external_cost_per_hour:dto.external_cost_per_hour,color:None,language:dto.language,assigned_site_ids:None,last_login:None,created_at:now,updated_at:now};
            c.insert_one(&u).await.map_err(|e|if e.to_string().contains("duplicate key"){SharedError::Conflict("Email already exists".into())}else{SharedError::Database(e.to_string())})?; Ok(u)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateUserDto) -> Fut<Option<User>> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let mut d=Document::new();
            if let Some(v)=dto.firstname{d.insert("firstname",v);}
            if let Some(v)=dto.lastname{d.insert("lastname",v);}
            if let Some(v)=dto.is_active{d.insert("is_active",v);}
            if d.is_empty(){return Repository::find_by_id(&MongoRepository::new(c),tid,id).await;}
            d.insert("updated_at",Utc::now());
            c.update_one(doc!{"tenant_id":tid.to_string(),"id":id.to_string()},doc!{"$set":d}).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Repository::find_by_id(&MongoRepository::new(c),tid,id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }
    pub fn count_all(&self) -> Fut<u64> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            c.count_documents(doc! {}).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    pub fn authenticate(&self, dto: LoginDto) -> Fut<AuthResponse> {
        let c=self.base.collection.clone();
        Box::pin(async move {
            let u=c.find_one(doc!{"email":&dto.email}).await.map_err(|e|SharedError::Database(e.to_string()))?.ok_or_else(||SharedError::Unauthorized("Invalid credentials".into()))?;
            if !u.is_active{return Err(SharedError::Unauthorized("Account disabled".into()));}
            verify_password(&dto.password,&u.password_hash)?;
            let token=generate_jwt(&u)?;
            Ok(AuthResponse{token,user_id:u.id,tenant_id:u.tenant_id,firstname:u.firstname,lastname:u.lastname,roles:u.roles})
        })
    }
}
