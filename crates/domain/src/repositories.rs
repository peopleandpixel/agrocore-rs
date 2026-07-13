use crate::entities::tenant::TenantId;
use agrocore_shared::{PaginatedResponse, Pagination, Result};
use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

pub type RepositoryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

pub trait VisibilityAwareEntity {
    #[cfg(feature = "mongodb")]
    fn visibility_filter(
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> mongodb::bson::Document;
}

#[cfg_attr(test, automock)]
pub trait Repository<T>: Send + Sync
where
    T: Serialize + Send + Sync + 'static,
{
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<T>>;
    fn find_all(
        &self,
        tid: TenantId,
        pagination: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<T>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

use crate::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
use crate::entities::site::{CreateSiteDto, Site, UpdateSiteDto};
use crate::entities::spatial::SpatialObject;

#[cfg_attr(test, automock)]
pub trait EquipmentRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Equipment>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Equipment>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateEquipmentDto,
        by: Uuid,
    ) -> RepositoryFuture<Equipment>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateEquipmentDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Equipment>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait SiteRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Site>>;
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Site>>;
    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateSiteDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Site>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
}

#[cfg_attr(test, automock)]
pub trait SpatialObjectRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SpatialObject>>;
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<SpatialObject>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn find_containing_point(
        &self,
        tid: TenantId,
        point: crate::entities::site::GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<SpatialObject>>;
}

use crate::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto};

#[cfg_attr(test, automock)]
pub trait AnimalRepository: Send + Sync {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Animal>>;
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[crate::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Animal>>;
    fn find_all(&self, tid: TenantId, p: Pagination)
    -> RepositoryFuture<PaginatedResponse<Animal>>;
    fn create(&self, tid: TenantId, dto: CreateAnimalDto, by: Uuid) -> RepositoryFuture<Animal>;
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateAnimalDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Animal>>;
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool>;
    fn add_treatment(
        &self,
        tid: TenantId,
        id: Uuid,
        treatment: crate::entities::livestock::TreatmentRecord,
    ) -> RepositoryFuture<bool>;
    fn add_grazing_record(
        &self,
        tid: TenantId,
        id: Uuid,
        record: crate::entities::livestock::GrazingRecord,
    ) -> RepositoryFuture<bool>;
}

use crate::entities::worker_task_status::{
    CreateWorkerTaskStatusDto, WorkerTaskStatus, WorkerTaskStatusType,
};

#[cfg_attr(test, automock)]
pub trait WorkerTaskStatusRepository: Send + Sync {
    fn find_by_task_and_worker(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>>;
    fn find_all_for_task(
        &self,
        tid: TenantId,
        task_id: Uuid,
    ) -> RepositoryFuture<Vec<WorkerTaskStatus>>;
    fn create(
        &self,
        tid: TenantId,
        dto: CreateWorkerTaskStatusDto,
    ) -> RepositoryFuture<WorkerTaskStatus>;
    fn update_status(
        &self,
        tid: TenantId,
        task_id: Uuid,
        worker_id: Uuid,
        status: WorkerTaskStatusType,
    ) -> RepositoryFuture<Option<WorkerTaskStatus>>;
}
