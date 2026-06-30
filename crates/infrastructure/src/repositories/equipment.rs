use chrono::Utc;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use uuid::Uuid;
use agrocore_domain::entities::equipment::{CreateEquipmentDto, Equipment, UpdateEquipmentDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{Repository, RepositoryFuture, EquipmentRepository};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use crate::repositories::base::MongoRepository;

#[derive(Clone)]
pub struct EquipmentRepo {
    base: MongoRepository<Equipment>,
}

impl EquipmentRepo {
    pub fn new(c: Collection<Equipment>) -> Self {
        Self {
            base: MongoRepository::new(c),
        }
    }
}

impl EquipmentRepository for EquipmentRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Equipment>> {
        self.base.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        self.base.find_all(tid, p)
    }

    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<Option<Equipment>> {
        self.base.find_by_id_visible(tid, id, user_id, roles)
    }

    fn find_all_visible(&self, tid: TenantId, p: Pagination, user_id: Uuid, roles: &[agrocore_domain::entities::user::UserRole]) -> RepositoryFuture<PaginatedResponse<Equipment>> {
        self.base.find_all_visible(tid, p, user_id, roles)
    }

    fn create(&self, tid: TenantId, dto: CreateEquipmentDto, _by: Uuid) -> RepositoryFuture<Equipment> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let now = Utc::now();
            let e = Equipment {
                id: Uuid::new_v4(),
                tenant_id: tid,
                label: dto.label,
                code: dto.code,
                equipment_type: dto.equipment_type,
                in_usage: false,
                maintenance_intervals: dto.maintenance_intervals,
                next_maintenance_date: None,
                last_maintenance_hours: None,
                created_at: now,
                updated_at: now,
            };
            c.insert_one(&e)
                .await
                .map_err(|err| SharedError::Database(err.to_string()))?;
            Ok(e)
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateEquipmentDto, _by: Uuid) -> RepositoryFuture<Option<Equipment>> {
        let c = self.base.collection.clone();
        Box::pin(async move {
            let mut d = Document::new();
            if let Some(v) = dto.label {
                d.insert("label", v);
            }
            if let Some(v) = dto.code {
                d.insert("code", v);
            }
            if let Some(v) = dto.equipment_type {
                let bson_type = mongodb::bson::to_bson(&v).map_err(|e| SharedError::Database(e.to_string()))?;
                d.insert("equipment_type", bson_type);
            }
            if let Some(v) = dto.in_usage {
                d.insert("in_usage", v);
            }
            if let Some(v) = dto.maintenance_intervals {
                d.insert("maintenance_intervals", mongodb::bson::to_bson(&v).unwrap());
            }
            if let Some(v) = dto.next_maintenance_date {
                d.insert("next_maintenance_date", mongodb::bson::to_bson(&v).unwrap());
            }
            if let Some(v) = dto.last_maintenance_hours {
                d.insert("last_maintenance_hours", v);
            }
            
            if d.is_empty() {
                return Repository::find_by_id(&MongoRepository::new(c), tid, id).await;
            }
            
            d.insert("updated_at", Utc::now());
            
            c.update_one(
                doc! {"tenant_id": tid.to_string(), "id": id.to_string()},
                doc! {"$set": d},
            )
            .await
            .map_err(|err| SharedError::Database(err.to_string()))?;
            
            Repository::find_by_id(&MongoRepository::new(c), tid, id).await
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        self.base.delete(tid, id)
    }
}
