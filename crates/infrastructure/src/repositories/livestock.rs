use uuid::Uuid;
use chrono::Utc;
use mongodb::bson::{doc, to_document};
use mongodb::Collection;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use agrocore_domain::repositories::{AnimalRepository, Repository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use crate::repositories::base::MongoRepository;

pub struct AnimalRepo {
    base: MongoRepository<Animal>,
}

impl AnimalRepo {
    pub fn new(c: Collection<Animal>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }
}

impl AnimalRepository for AnimalRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Animal>> {
        self.base.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Animal>> {
        self.base.find_all(tid, p)
    }

    fn create(&self, tid: TenantId, dto: CreateAnimalDto, _by: Uuid) -> RepositoryFuture<Animal> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let animal = Animal {
                id: Uuid::new_v4(),
                tenant_id: tid,
                species: dto.species,
                breed: dto.breed,
                identifier: dto.identifier,
                birth_date: dto.birth_date,
                gender: dto.gender,
                status: agrocore_domain::entities::livestock::AnimalStatus::Active,
                current_site_id: dto.current_site_id,
                group_id: None,
                weight_kg: None,
                last_weight_date: None,
                treatments: Vec::new(),
                grazing_history: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            c.insert_one(animal.clone())
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(animal)
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateAnimalDto, _by: Uuid) -> RepositoryFuture<Option<Animal>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut update_doc = doc! {};
            if let Some(b) = dto.breed { update_doc.insert("breed", b); }
            if let Some(i) = dto.identifier { update_doc.insert("identifier", i); }
            if let Some(s) = dto.status { 
                let s_val = mongodb::bson::to_bson(&s).unwrap_or(mongodb::bson::Bson::Null);
                update_doc.insert("status", s_val); 
            }
            if let Some(sid) = dto.current_site_id { update_doc.insert("current_site_id", sid.to_string()); }
            if let Some(gid) = dto.group_id { update_doc.insert("group_id", gid.to_string()); }
            if let Some(w) = dto.weight_kg { 
                update_doc.insert("weight_kg", w);
                update_doc.insert("last_weight_date", Utc::now().to_rfc3339());
            }

            if update_doc.is_empty() {
                return Ok(None);
            }

            update_doc.insert("updated_at", Utc::now().to_rfc3339());

            let filter = doc! { "tenant_id": tid.to_string(), "id": id.to_string() };
            let res = c.find_one_and_update(filter, doc! { "$set": update_doc })
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(res)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }

    fn add_treatment(&self, tid: TenantId, id: Uuid, treatment: TreatmentRecord) -> RepositoryFuture<bool> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let filter = doc! { "tenant_id": tid.to_string(), "id": id.to_string() };
            let t_doc = to_document(&treatment).map_err(|e| SharedError::Database(e.to_string()))?;
            let update = doc! { 
                "$push": { "treatments": t_doc },
                "$set": { "updated_at": Utc::now().to_rfc3339() }
            };
            
            let res = c.update_one(filter, update)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(res.modified_count > 0)
        })
    }

    fn add_grazing_record(&self, tid: TenantId, id: Uuid, record: GrazingRecord) -> RepositoryFuture<bool> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let filter = doc! { "tenant_id": tid.to_string(), "id": id.to_string() };
            let r_doc = to_document(&record).map_err(|e| SharedError::Database(e.to_string()))?;
            let update = doc! { 
                "$push": { "grazing_history": r_doc },
                "$set": { 
                    "current_site_id": record.site_id.to_string(),
                    "updated_at": Utc::now().to_rfc3339() 
                }
            };
            
            let res = c.update_one(filter, update)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(res.modified_count > 0)
        })
    }
}
