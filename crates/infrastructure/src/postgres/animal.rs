use agrocore_domain::entities::livestock::{
    Animal, CreateAnimalDto, GrazingRecord, TreatmentRecord, UpdateAnimalDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    AnimalRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

agrocore_shared::pg_repo!(PgAnimalRepo);

impl AnimalRepository for PgAnimalRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Animal>("SELECT * FROM animals WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
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
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);

            if can_see_all {
                sqlx::query_as::<_, Animal>(
                    "SELECT * FROM animals WHERE id = $1 AND tenant_id = $2",
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, Animal>(
                    "SELECT * FROM animals WHERE id = $1 AND tenant_id = $2 AND id = $3",
                )
                .bind(id)
                .bind(tid)
                .bind(user_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<Animal>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM animals WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Animal> = sqlx::query_as("SELECT * FROM animals WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid)
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 {
                0
            } else {
                (total as f64 / per_page as f64).ceil() as u64
            };

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
                r#"INSERT INTO animals (id, tenant_id, identifier, species, breed, birth_date, gender, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9)
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(&dto.identifier)
            .bind(serde_json::to_value(&dto.species).unwrap())
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

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateAnimalDto,
        _by: Uuid,
    ) -> Fut<Option<Animal>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, Animal>(
                r#"UPDATE animals SET
                    identifier = COALESCE($1, identifier),
                    breed = COALESCE($2, breed),
                    status = COALESCE($3, status),
                    current_site_id = COALESCE($4, current_site_id),
                    updated_at = $5
                   WHERE id = $6 AND tenant_id = $7
                   RETURNING *"#,
            )
            .bind(&dto.identifier)
            .bind(&dto.breed)
            .bind(
                dto.status
                    .as_ref()
                    .map(|s| serde_json::to_value(s).unwrap()),
            )
            .bind(dto.current_site_id)
            .bind(now)
            .bind(id)
            .bind(tid)
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
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn add_treatment(&self, tid: TenantId, id: Uuid, record: TreatmentRecord) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query(
                r#"INSERT INTO animal_treatments (id, animal_id, tenant_id, treatment_date, treatment_type, medication, dosage, veterinarian, withdrawal_days, notes, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"#)
            .bind(record.id)
            .bind(id)
            .bind(tid)
            .bind(record.date)
            .bind(record.treatment_type)
            .bind(record.medication)
            .bind(record.dosage)
            .bind(record.veterinarian)
            .bind(record.withdrawal_days.map(|d| d as i32))
            .bind(record.notes)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_treatments_by_animal(
        &self,
        tid: TenantId,
        animal_id: Uuid,
    ) -> Fut<Option<Vec<TreatmentRecord>>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TreatmentRecord>(
                r#"SELECT id, animal_id, treatment_type, date, medication, dosage, veterinarian, withdrawal_days, notes, created_at
                   FROM animal_treatments
                   WHERE animal_id = $1 AND tenant_id = $2
                   ORDER BY date DESC"#,
            )
            .bind(animal_id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map(|v| if v.is_empty() { None } else { Some(v) })
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn add_grazing_record(&self, tid: TenantId, id: Uuid, record: GrazingRecord) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query(
                r#"INSERT INTO animal_grazing_records (id, animal_id, tenant_id, site_id, start_date, end_date, notes, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"#)
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(tid)
            .bind(record.site_id)
            .bind(record.start_date)
            .bind(record.end_date)
            .bind(record.notes)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
