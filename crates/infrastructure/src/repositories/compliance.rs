use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;
use uuid::Uuid;

use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::compliance::{
    AuditLog, ComplianceChecklist, FertilizerRecord,
    CreateComplianceChecklistDto, CreateFertilizerRecordDto,
};
use agrocore_domain::entities::plant_protection::{PlantProtectionRecord, CreatePlantProtectionDto, ApplicatorLicense};
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::{MongoRepository, paginate};

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct AuditLogRepo { base: MongoRepository<AuditLog> }
impl AuditLogRepo {
    pub fn new(c: Collection<AuditLog>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<AuditLog>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<AuditLog>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"created_at":-1})).await
        })
    }
    pub fn create_log(&self, log: AuditLog) -> Fut<AuditLog> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            c.insert_one(&log).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(log)
        })
    }
}

#[derive(Clone)]
pub struct ComplianceChecklistRepo { base: MongoRepository<ComplianceChecklist> }
impl ComplianceChecklistRepo {
    pub fn new(c: Collection<ComplianceChecklist>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ComplianceChecklist>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<ComplianceChecklist>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateComplianceChecklistDto) -> Fut<ComplianceChecklist> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let cl=ComplianceChecklist{id:Uuid::new_v4(),tenant_id:tid,site_id:dto.site_id,checklist_type:dto.checklist_type,status:agrocore_domain::entities::compliance::ComplianceStatus::Pending,items:Vec::new(),due_date:dto.due_date,completed_at:None,created_at:now,updated_at:now};
            c.insert_one(&cl).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(cl)
        })
    }
}

#[derive(Clone)]
pub struct FertilizerRecordRepo { base: MongoRepository<FertilizerRecord> }
impl FertilizerRecordRepo {
    pub fn new(c: Collection<FertilizerRecord>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FertilizerRecord>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FertilizerRecord>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"application_date":-1})).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreateFertilizerRecordDto) -> Fut<FertilizerRecord> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let rec=FertilizerRecord{id:Uuid::new_v4(),tenant_id:tid,site_id:dto.site_id,order_id:dto.order_id,product_name:dto.product_name,nutrient_n:dto.nutrient_n,nutrient_p:dto.nutrient_p,nutrient_k:dto.nutrient_k,quantity_kg:dto.quantity_kg,area_ha:dto.area_ha,application_date:dto.application_date,created_at:now};
            c.insert_one(&rec).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(rec)
        })
    }
}

#[derive(Clone)]
pub struct PlantProtectionRecordRepo { base: MongoRepository<PlantProtectionRecord> }
impl PlantProtectionRecordRepo {
    pub fn new(c: Collection<PlantProtectionRecord>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PlantProtectionRecord>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PlantProtectionRecord>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"application_date":-1})).await
        })
    }
    pub fn create(&self, tid: TenantId, dto: CreatePlantProtectionDto) -> Fut<PlantProtectionRecord> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now=Utc::now();
            let rec=PlantProtectionRecord{id:Uuid::new_v4(),tenant_id:tid,site_id:dto.site_id,order_id:dto.order_id,product_name:dto.product_name,active_substance:dto.active_substance,dosage_per_ha:dto.dosage_per_ha,total_quantity:dto.total_quantity,area_ha:dto.area_ha,application_date:dto.application_date,pre_harvest_days:dto.pre_harvest_days,re_entry_days:dto.re_entry_days,weather_conditions:dto.weather_conditions,applicator_license:dto.applicator_license,created_at:now};
            c.insert_one(&rec).await.map_err(|e|SharedError::Database(e.to_string()))?;
            Ok(rec)
        })
    }
}

#[derive(Clone)]
pub struct ApplicatorLicenseRepo { base: MongoRepository<ApplicatorLicense> }
impl ApplicatorLicenseRepo {
    pub fn new(c: Collection<ApplicatorLicense>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<ApplicatorLicense>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<ApplicatorLicense>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let f=doc!{"tenant_id":tid.to_string()};
            paginate(&c, f, p, Some(doc!{"valid_until":1})).await
        })
    }
}
