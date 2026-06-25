use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;
use serde::Serialize;
use crate::entities::tenant::TenantId;
use agrocore_shared::{PaginatedResponse, Pagination, Result};

#[cfg(test)]
use mockall::automock;

pub type RepositoryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[cfg_attr(test, automock)]
pub trait Repository<T>: Send + Sync 
where T: Serialize + Send + Sync + 'static
{
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<T>>;
    fn find_all(&self, tid: TenantId, pagination: Pagination) -> RepositoryFuture<PaginatedResponse<T>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

use crate::entities::site::{Site, CreateSiteDto, UpdateSiteDto};

#[cfg_attr(test, automock)]
pub trait SiteRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site>;
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateSiteDto, by: Uuid) -> RepositoryFuture<Option<Site>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}
