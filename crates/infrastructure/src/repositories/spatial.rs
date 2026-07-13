use crate::repositories::base::MongoRepository;
use agrocore_domain::entities::site::GeoPoint;
use agrocore_domain::entities::spatial::SpatialObject;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture, SpatialObjectRepository};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use futures::StreamExt;
use mongodb::Collection;
use mongodb::bson::doc;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct SpatialObjectRepo {
    base: MongoRepository<SpatialObject>,
}

impl SpatialObjectRepo {
    pub fn new(c: Collection<SpatialObject>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }

    pub fn query_containing_point(
        &self,
        tid: TenantId,
        point: GeoPoint,
        site_id: Option<Uuid>,
    ) -> Fut<Vec<SpatialObject>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut filter = doc! { "tenant_id": tid.to_string() };
            if let Some(site_id) = site_id {
                filter.insert("site_id", site_id.to_string());
            }

            let mut cursor = c
                .find(filter)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut matches = Vec::new();
            while let Some(res) = cursor.next().await {
                let object = res.map_err(|e| SharedError::Database(e.to_string()))?;
                if object.is_active && object.contains_point(&point) {
                    matches.push(object);
                }
            }

            Ok(matches)
        })
    }
}

impl SpatialObjectRepository for SpatialObjectRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SpatialObject>> {
        self.base.find_by_id(tid, id)
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SpatialObject>> {
        self.base.find_all(tid, p)
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }

    fn find_containing_point(
        &self,
        tid: TenantId,
        point: GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<SpatialObject>> {
        Self::query_containing_point(self, tid, point, site_id)
    }
}
