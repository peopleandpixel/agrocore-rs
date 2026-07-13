#!/usr/bin/env bash
# Generate remaining PostgreSQL repositories from templates
# Run from project root

cat > crates/infrastructure/src/postgres/animal.rs << 'EOF'
use agrocore_domain::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{AnimalRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgAnimalRepo { pool: PgPool }

impl PgAnimalRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl AnimalRepository for PgAnimalRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Animal>> { /* TODO */ Box::pin(async { Err(SharedError::NotFound.to_error()) }) }
    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, _uid: Uuid, _roles: &[UserRole]) -> RepositoryFuture<Option<Animal>> { self.find_by_id(tid, id) }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Animal>> { /* TODO */ Box::pin(async { Err(SharedError::Internal("TODO".into()).to_error()) }) }
    fn create(&self, tid: TenantId, dto: CreateAnimalDto, _by: Uuid) -> RepositoryFuture<Animal> { /* TODO */ Box::pin(async { Err(SharedError::Internal("TODO".into()).to_error()) }) }
    fn update(&self, _tid: TenantId, _id: Uuid, _dto: UpdateAnimalDto, _by: Uuid) -> RepositoryFuture<Option<Animal>> { Box::pin(async { Ok(None) }) }
    fn delete(&self, _tid: TenantId, _id: Uuid) -> RepositoryFuture<bool> { Box::pin(async { Ok(false) }) }
    fn add_treatment(&self, _tid: TenantId, _id: Uuid, _record: TreatmentRecord) -> RepositoryFuture<bool> { Box::pin(async { Ok(false) }) }
    fn add_grazing_record(&self, _tid: TenantId, _id: Uuid, _record: GrazingRecord) -> RepositoryFuture<bool> { Box::pin(async { Ok(false) }) }
}
EOF

echo "Generated: animal.rs"