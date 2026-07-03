use crate::repositories::base::MongoRepository;
use agrocore_domain::entities::tenant::{CreateTenantDto, Tenant};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use futures::Future;
use mongodb::Collection;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct TenantRepo {
    base: MongoRepository<Tenant>,
}

impl TenantRepo {
    pub fn new(c: Collection<Tenant>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }

    pub fn create(&self, dto: CreateTenantDto) -> Fut<Tenant> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let t = Tenant {
                id: Uuid::new_v4(),
                name: dto.name,
                slug: dto.slug,
                config: dto.config.unwrap_or_default(),
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&t)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(t)
        })
    }

    pub fn find_by_id(&self, id: Uuid) -> Fut<Option<Tenant>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            c.find_one(mongodb::bson::doc! { "id": id.to_string() })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    pub fn delete(&self, id: Uuid) -> Fut<bool> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let res = c
                .update_one(
                    mongodb::bson::doc! { "id": id.to_string() },
                    mongodb::bson::doc! { "$set": { "is_active": false, "updated_at": mongodb::bson::DateTime::now() } },
                )
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(res.modified_count > 0)
        })
    }
}
