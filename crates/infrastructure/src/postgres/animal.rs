use agrocore_domain::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{AnimalRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

#[derive(Clone)]
pub struct PgAnimalRepo {
    pool: PgPool,
}

impl PgAnimalRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AnimalRepository for PgAnimalRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (Animal,)>("SELECT row_to_json(animals) FROM animals WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            row.map(|(a,)| a).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> Fut<Option<Animal>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<Animal> = sqlx::query_as("SELECT * FROM animals WHERE tenant_id = $1::uuid LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                items,
                total,
                page: p.page,
                limit: p.limit,
            })
        })
    }

    fn create(&self, tid: TenantId, dto: CreateAnimalDto, _by: Uuid) -> Fut<Animal> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as("INSERT INTO animals (id, tenant_id, ...) VALUES (...) RETURNING *")
                .bind(id)
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateAnimalDto, _by: Uuid) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // TBD
            Ok(None)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM animals WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(result.rows_affected() > 0)
        })
    }

    fn add_treatment(&self, tid: TenantId, id: Uuid, record: TreatmentRecord) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // TBD: implement treatment records
            Ok(false)
        })
    }

    fn add_grazing_record(&self, tid: TenantId, id: Uuid, record: GrazingRecord) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // TBD
            Ok(false)
        })
    }
}