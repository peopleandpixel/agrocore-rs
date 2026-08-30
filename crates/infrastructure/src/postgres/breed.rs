use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{Breed, CreateBreedDto, Species, UpdateBreedDto};
use agrocore_domain::repositories::{
    BreedRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

pg_repo!(PgBreedRepo);

impl BreedRepository for PgBreedRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Breed>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Breed>("SELECT * FROM breeds WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Breed>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM breeds WHERE tenant_id = $1")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Breed> = sqlx::query_as(
                "SELECT * FROM breeds WHERE tenant_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 {
                0
            } else {
                ((total as f64) / (per_page as f64)).ceil() as u64
            };

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page: page as u64,
                per_page: per_page as u64,
                total_pages,
            })
        })
    }

    fn find_by_species(&self, tid: TenantId, species: Species) -> RepositoryFuture<Vec<Breed>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Breed>(
                "SELECT * FROM breeds WHERE tenant_id = $1 AND species = $2 ORDER BY name",
            )
            .bind(tid)
            .bind(species.as_str())
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateBreedDto, _by: Uuid) -> RepositoryFuture<Breed> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let now = chrono::Utc::now();

            sqlx::query(
                r#"
                INSERT INTO breeds (id, species, name, origin, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(id)
            .bind(dto.species.as_str())
            .bind(&dto.name)
            .bind(&dto.origin)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(Breed {
                id,
                species: dto.species.as_str().to_string(),
                name: dto.name,
                origin: dto.origin,
            })
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateBreedDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Breed>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let existing =
                sqlx::query_as::<_, Breed>("SELECT * FROM breeds WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(mut breed) = existing {
                if let Some(species) = dto.species {
                    breed.species = species.as_str().to_string();
                }
                if let Some(name) = dto.name {
                    breed.name = name;
                }
                if let Some(origin) = dto.origin {
                    breed.origin = Some(origin);
                }

                sqlx::query(
                    r#"
                    UPDATE breeds SET species = $1, name = $2, origin = $3 WHERE id = $4 AND tenant_id = $5
                    "#,
                )
                .bind(&breed.species)
                .bind(&breed.name)
                .bind(&breed.origin)
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                Ok(Some(breed))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM breeds WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
