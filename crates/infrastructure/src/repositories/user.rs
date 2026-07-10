use crate::repositories::auth_utils::{generate_jwt, hash_password, verify_password};
use crate::repositories::base::{paginate, MongoRepository};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::{
    AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User, UserRole,
};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct UserRepo {
    base: MongoRepository<User>,
}
impl UserRepo {
    pub fn new(c: Collection<User>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_by_email(&self, email: &str) -> Fut<Option<User>> {
        let c = self.base.collection.clone();
        let email = email.to_string();
        Box::pin(async move {
            c.find_one(doc! {"email":&email})
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    pub fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<User>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! {"tenant_id":tid.to_string()};
            let sort = doc! {"lastname":1};
            paginate(&c, f, p, Some(sort)).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateUserDto) -> Fut<User> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let pw = hash_password(&dto.password)?;
            let u = User {
                id: Uuid::new_v4(),
                tenant_id: tid,
                firstname: dto.firstname,
                lastname: dto.lastname,
                email: dto.email,
                password_hash: pw,
                roles: dto.roles.unwrap_or_else(|| vec![UserRole::Worker]),
                is_active: true,
                internal_cost_per_hour: dto.internal_cost_per_hour,
                external_cost_per_hour: dto.external_cost_per_hour,
                color: None,
                language: dto.language,
                assigned_site_ids: None,
                last_login: None,
                refresh_token: None,
                refresh_token_expires_at: None,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&u).await.map_err(|e| {
                if e.to_string().contains("duplicate key") {
                    SharedError::Conflict("Email already exists".into())
                } else {
                    SharedError::Database(e.to_string())
                }
            })?;
            Ok(u)
        })
    }
    pub fn update(&self, tid: TenantId, id: Uuid, dto: UpdateUserDto) -> Fut<Option<User>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut d = Document::new();
            if let Some(v) = dto.firstname {
                d.insert("firstname", v);
            }
            if let Some(v) = dto.lastname {
                d.insert("lastname", v);
            }
            if let Some(v) = dto.is_active {
                d.insert("is_active", v);
            }
            if d.is_empty() {
                return Repository::find_by_id(&MongoRepository::new(c), tid, id).await;
            }
            d.insert("updated_at", Utc::now());
            c.update_one(
                doc! {"tenant_id":tid.to_string(),"id":id.to_string()},
                doc! {"$set":d},
            )
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            Repository::find_by_id(&MongoRepository::new(c), tid, id).await
        })
    }
    pub fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }
    pub fn count_all(&self) -> Fut<u64> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            c.count_documents(doc! {})
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    pub fn authenticate(&self, dto: LoginDto) -> Fut<AuthResponse> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut u = c
                .find_one(doc! {"email":&dto.email})
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .ok_or_else(|| SharedError::Unauthorized("Invalid credentials".into()))?;
            if !u.is_active {
                return Err(SharedError::Unauthorized("Account disabled".into()));
            }
            verify_password(&dto.password, &u.password_hash)?;

            // Generate tokens
            let token = generate_jwt(&u)?;
            let refresh_token = Uuid::new_v4().to_string();
            let refresh_token_expires_at = Utc::now() + chrono::Duration::days(7);
            u.refresh_token = Some(refresh_token.clone());
            u.refresh_token_expires_at = Some(refresh_token_expires_at);

            // Update user with refresh token
            c.update_one(
                doc! { "_id": u.id.to_string(), "tenant_id": u.tenant_id.to_string() },
                doc! { "$set": { "refresh_token": refresh_token.clone(), "refresh_token_expires_at": refresh_token_expires_at } }
            )
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(AuthResponse {
                token,
                user_id: u.id,
                tenant_id: u.tenant_id,
                firstname: u.firstname,
                lastname: u.lastname,
                roles: u.roles,
            })
        })
    }

    pub fn find_by_refresh_token(&self, refresh_token: &str) -> Fut<Option<User>> {
        let c = self.base.collection.clone();
        let rt = refresh_token.to_string();
        Box::pin(async move {
            c.find_one(doc! {"refresh_token": &rt})
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    pub fn update_refresh_token(&self, user_id: Uuid, refresh_token: &str, expires_at: chrono::DateTime<Utc>) -> Fut<bool> {
        let c = self.base.collection.clone();
        let rt = refresh_token.to_string();
        let expires = expires_at;
        Box::pin(async move {
            let result = c
                .update_one(
                    doc! { "_id": user_id.to_string() },
                    doc! { "$set": { "refresh_token": rt, "refresh_token_expires_at": expires } }
                )
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(result.modified_count > 0)
        })
    }

    pub fn invalidate_refresh_token(&self, user_id: Uuid) -> Fut<bool> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let result = c
                .update_one(
                    doc! { "_id": user_id.to_string() },
                    doc! { "$set": { "refresh_token": None::<String>, "refresh_token_expires_at": None::<chrono::DateTime<Utc>> } }
                )
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(result.modified_count > 0)
        })
    }
}
