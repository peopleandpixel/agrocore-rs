use std::marker::PhantomData;
use futures::StreamExt;
use mongodb::{Collection, options::FindOptions};
use mongodb::bson::{doc, Document};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};

pub struct MongoRepository<T> 
where T: Send + Sync
{
    pub collection: Collection<T>,
    _marker: PhantomData<T>,
}

impl<T> Clone for MongoRepository<T>
where T: Send + Sync
{
    fn clone(&self) -> Self {
        Self {
            collection: self.collection.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> MongoRepository<T> 
where T: Serialize + DeserializeOwned + Send + Sync
{
    pub fn new(collection: Collection<T>) -> Self {
        Self {
            collection,
            _marker: PhantomData,
        }
    }
}

impl<T> Repository<T> for MongoRepository<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<T>> {
        let c = self.collection.clone();
        Box::pin(async move {
            c.find_one(doc! { "tenant_id": tid.to_string(), "id": id.to_string() })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<T>> {
        let c = self.collection.clone();
        let page = p.page.unwrap_or(0);
        let pp = p.per_page.unwrap_or(20);
        
        Box::pin(async move {
            let filter = doc! { "tenant_id": tid.to_string() };
            let total = c.count_documents(filter.clone())
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let opts = FindOptions::builder()
                .skip(page * pp)
                .limit(pp as i64)
                .sort(doc! { "updated_at": -1 })
                .build();
            
            let mut cursor = c.find(filter)
                .with_options(opts)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let mut data = Vec::new();
            while let Some(res) = cursor.next().await {
                data.push(res.map_err(|e| SharedError::Database(e.to_string()))?);
            }
            
            Ok(PaginatedResponse {
                data,
                total,
                page,
                per_page: pp,
                total_pages: (total as f64 / pp as f64).ceil() as u64,
            })
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let c = self.collection.clone();
        Box::pin(async move {
            let res = c.update_one(
                doc! { "tenant_id": tid.to_string(), "id": id.to_string() },
                doc! { "$set": { "is_active": false, "updated_at": mongodb::bson::DateTime::now() } }
            )
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(res.modified_count > 0)
        })
    }
}

pub async fn paginate<T>(
    collection: &Collection<T>,
    filter: Document,
    p: Pagination,
    sort: Option<Document>,
) -> agrocore_shared::Result<PaginatedResponse<T>>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    let page = p.page.unwrap_or(0);
    let pp = p.per_page.unwrap_or(20);
    
    let total = collection.count_documents(filter.clone())
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;
    
    let mut opts = FindOptions::builder()
        .skip(page * pp)
        .limit(pp as i64)
        .build();
    
    if let Some(s) = sort {
        opts.sort = Some(s);
    } else {
        opts.sort = Some(doc! { "updated_at": -1 });
    }
    
    let mut cursor = collection.find(filter)
        .with_options(opts)
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;
    
    let mut data = Vec::new();
    while let Some(res) = cursor.next().await {
        data.push(res.map_err(|e| SharedError::Database(e.to_string()))?);
    }
    
    Ok(PaginatedResponse {
        data,
        total,
        page,
        per_page: pp,
        total_pages: (total as f64 / pp as f64).ceil() as u64,
    })
}
