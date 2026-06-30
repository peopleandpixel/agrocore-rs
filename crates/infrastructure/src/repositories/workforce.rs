use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use futures::StreamExt;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::workforce::{
    Worker, WorkLog, WorkerLocation,
    CreateWorkerDto, CreateWorkLogDto, ReportLocationDto,
};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct WorkerRepo { base: MongoRepository<Worker> }
impl WorkerRepo {
    pub fn new(c: Collection<Worker>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Worker>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Worker>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateWorkerDto) -> Fut<Worker> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let worker = Worker {
                id: Uuid::new_v4(),
                tenant_id: tid,
                user_id: dto.user_id,
                contract_type: dto.contract_type,
                language: dto.language,
                skills: dto.skills.unwrap_or_default(),
                certifications: Vec::new(),
                emergency_contact: dto.emergency_contact,
                nationality: dto.nationality,
                is_active: true,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&worker).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(worker)
        })
    }
}

#[derive(Clone)]
pub struct WorkLogRepo { base: MongoRepository<WorkLog> }
impl WorkLogRepo {
    pub fn new(c: Collection<WorkLog>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<WorkLog>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<WorkLog>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f = doc! { "tenant_id": tid.to_string() };
            paginate(&c, f, p, Some(doc! { "date": -1 })).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateWorkLogDto) -> Fut<WorkLog> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let log = WorkLog {
                id: Uuid::new_v4(),
                tenant_id: tid,
                worker_id: dto.worker_id,
                date: dto.date,
                hours_worked: dto.hours_worked,
                overtime_hours: dto.overtime_hours,
                rest_period_hours: dto.rest_period_hours,
                task_description: dto.task_description,
                site_id: dto.site_id,
                is_night_shift: dto.is_night_shift,
                breaks_taken: dto.breaks_taken,
                created_at: now,
            };
            c.insert_one(&log).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(log)
        })
    }
}

#[derive(Clone)]
pub struct WorkerLocationRepo { base: MongoRepository<WorkerLocation> }
impl WorkerLocationRepo {
    pub fn new(c: Collection<WorkerLocation>) -> Self { Self { base: MongoRepository::new(c) } }

    pub fn report_location(&self, tid: TenantId, worker_id: Uuid, dto: ReportLocationDto) -> Fut<WorkerLocation> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let loc = WorkerLocation {
                id: Uuid::new_v4(),
                tenant_id: tid,
                worker_id,
                lat: dto.lat,
                lng: dto.lng,
                current_task_id: dto.current_task_id,
                timestamp: now,
            };
            c.insert_one(&loc).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(loc)
        })
    }

    pub fn get_latest_locations(&self, tid: TenantId) -> Fut<Vec<WorkerLocation>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let pipeline = vec![
                doc! { "$match": { "tenant_id": tid.to_string() } },
                doc! { "$sort": { "timestamp": -1 } },
                doc! { "$group": {
                    "_id": "$worker_id",
                    "latest": { "$first": "$$ROOT" }
                }},
                doc! { "$replaceRoot": { "newRoot": "$latest" } }
            ];
            let mut cursor = c.aggregate(pipeline).await.map_err(|e| SharedError::Database(e.to_string()))?;
            let mut result = Vec::new();
            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| SharedError::Database(e.to_string()))?;
                let loc: WorkerLocation = mongodb::bson::from_document(doc).map_err(|e| SharedError::Database(e.to_string()))?;
                result.push(loc);
            }
            Ok(result)
        })
    }
}
