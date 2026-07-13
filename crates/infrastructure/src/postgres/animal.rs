use agrocore_domain::entities::livestock::{Animal, CreateAnimalDto, UpdateAnimalDto, TreatmentRecord, GrazingRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{AnimalRepository, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
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
            sqlx::query_as::<_, (Animal,)>("SELECT row_to_json(animals) FROM animals WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(a,)| a)
                .ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let can_see_all = roles.contains(&UserRole::Admin) || roles.contains(&UserRole::Manager);
            
            if can_see_all {
                sqlx::query_as::<_, (Animal,)>("SELECT row_to_json(animals) FROM animals WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid.to_string())
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?
                    .map(|(a,)| a)
                    .ok_or_else(|| SharedError::NotFound.to_error())
            } else {
                sqlx::query_as::<_, (Animal,)>("SELECT row_to_json(animals) FROM animals WHERE id = $1 AND tenant_id = $2 AND id = $3")
                    .bind(id)
                    .bind(tid.to_string())
                    .bind(user_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?
                    .map(|(a,)| a)
                    .ok_or_else(|| SharedError::NotFound.to_error())
            }
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<Animal>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true)")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Animal> = sqlx::query_as("SELECT * FROM animals WHERE tenant_id = $1::uuid AND (is_active IS NULL OR is_active = true) ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn create(&self, tid: TenantId, dto: CreateAnimalDto, _by: Uuid) -> Fut<Animal> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            sqlx::query_as::<_, Animal>(
                r#"INSERT INTO animals (id, tenant_id, tag_number, species, breed, birth_date, gender, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9)
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.tag_number)
            .bind(&dto.species)
            .bind(&dto.breed)
            .bind(dto.birth_date)
            .bind(&dto.gender)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateAnimalDto, _by: Uuid) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, Animal>(
                r#"UPDATE animals SET
                    tag_number = COALESCE($1, tag_number),
                    species = COALESCE($2, species),
                    breed = COALESCE($3, breed),
                    birth_date = COALESCE($4, birth_date),
                    gender = COALESCE($5, gender),
                    is_active = COALESCE($6, is_active),
                    updated_at = $7
                   WHERE id = $8 AND tenant_id = $9
                   RETURNING *"#)
            .bind(&dto.tag_number)
            .bind(&dto.species)
            .bind(&dto.breed)
            .bind(dto.birth_date)
            .bind(&dto.gender)
            .bind(dto.is_active)
            .bind(now)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("UPDATE animals SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
                .bind(Utc::now())
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }

    fn add_treatment(&self, _tid: TenantId, _id: Uuid, _record: TreatmentRecord) -> Fut<bool> {
        Box::pin(async move { Ok(false) })
    }

    fn add_grazing_record(&self, _tid: TenantId, _id: Uuid, _record: GrazingRecord) -> Fut<bool> {
        Box::pin(async move { Ok(false) })
    }
}