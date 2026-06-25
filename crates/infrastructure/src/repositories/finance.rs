use std::future::Future;
use std::pin::Pin;
use chrono::Utc;
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::finance::{CostCenter, FinancialRecord, PACApplication, CreateCostCenterDto, CreateFinancialRecordDto, CreatePACApplicationDto, PACStatus};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
use crate::repositories::base::MongoRepository;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct PACApplicationRepo { base: MongoRepository<PACApplication> }
impl PACApplicationRepo {
    pub fn new(c: Collection<PACApplication>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreatePACApplicationDto) -> Fut<PACApplication> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let app = PACApplication {
                id: Uuid::new_v4(),
                tenant_id: tid,
                year: dto.year,
                application_number: dto.application_number,
                status: PACStatus::Draft,
                total_eligible_area: dto.total_eligible_area,
                submitted_at: None,
                eco_schemes: dto.eco_schemes,
                documents_urls: Vec::new(),
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&app).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(app)
        })
    }
}

#[derive(Clone)]
pub struct CostCenterRepo { base: MongoRepository<CostCenter> }
impl CostCenterRepo {
    pub fn new(c: Collection<CostCenter>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<CostCenter>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<CostCenter>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateCostCenterDto) -> Fut<CostCenter> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let cc = CostCenter {
                id: Uuid::new_v4(),
                tenant_id: tid,
                label: dto.label,
                code: dto.code,
                cost_center_type: dto.cost_center_type,
                reference_id: dto.reference_id,
                is_active: true,
            };
            c.insert_one(&cc).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(cc)
        })
    }
}

#[derive(Clone)]
pub struct FinancialRecordRepo { base: MongoRepository<FinancialRecord> }
impl FinancialRecordRepo {
    pub fn new(c: Collection<FinancialRecord>) -> Self { Self { base: MongoRepository::new(c) } }
    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FinancialRecord>> {
        self.base.find_by_id(tid, id)
    }
    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<FinancialRecord>> {
        self.base.find_all(tid, p)
    }
    pub fn create(&self, tid: TenantId, dto: CreateFinancialRecordDto) -> Fut<FinancialRecord> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let rec = FinancialRecord {
                id: Uuid::new_v4(),
                tenant_id: tid,
                cost_center_id: dto.cost_center_id,
                date: dto.date,
                amount: dto.amount,
                currency: dto.currency,
                record_type: dto.record_type,
                category: dto.category,
                description: dto.description,
                reference_id: dto.reference_id,
                created_at: now,
            };
            c.insert_one(&rec).await.map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(rec)
        })
    }
}
