use agrocore_domain::entities::type_system::{
    CreateTypeCategoryDto, TypeCategory, UpdateTypeCategoryDto,
};
use agrocore_domain::repositories::{RepositoryFuture, TypeCategoryRepository};
use agrocore_shared::SharedError;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgTypeCategoryRepo);

impl TypeCategoryRepository for PgTypeCategoryRepo {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<Option<TypeCategory>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeCategory>(
                "SELECT id, name, label, description, is_hierarchical FROM type_categories WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self) -> RepositoryFuture<Vec<TypeCategory>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeCategory>(
                "SELECT id, name, label, description, is_hierarchical FROM type_categories ORDER BY name",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_name(&self, name: &str) -> RepositoryFuture<Option<TypeCategory>> {
        let pool = self.pool.clone();
        let name = name.to_string();
        Box::pin(async move {
            sqlx::query_as::<_, TypeCategory>(
                "SELECT id, name, label, description, is_hierarchical FROM type_categories WHERE name = $1",
            )
            .bind(&name)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, dto: CreateTypeCategoryDto) -> RepositoryFuture<TypeCategory> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, TypeCategory>(
                "INSERT INTO type_categories (name, label, description, is_hierarchical)
                 VALUES ($1, $2, $3, $4)
                 RETURNING *",
            )
            .bind(&dto.name)
            .bind(&dto.label)
            .bind(&dto.description)
            .bind(dto.is_hierarchical)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        id: Uuid,
        dto: UpdateTypeCategoryDto,
    ) -> RepositoryFuture<Option<TypeCategory>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut parts = Vec::new();
            let mut idx = 1u32;

            if let Some(ref label) = dto.label {
                parts.push(format!("label = ${}", idx));
                idx += 1;
            }
            if let Some(ref description) = dto.description {
                parts.push(format!("description = ${}", idx));
                idx += 1;
            }
            if let Some(is_hierarchical) = dto.is_hierarchical {
                parts.push(format!("is_hierarchical = ${}", idx));
                idx += 1;
            }

            if parts.is_empty() {
                return sqlx::query_as::<_, TypeCategory>(
                    "SELECT id, name, label, description, is_hierarchical FROM type_categories WHERE id = $1",
                )
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()));
            }

            let id_param = idx;
            let query = format!(
                "UPDATE type_categories SET {} WHERE id = ${} RETURNING *",
                parts.join(", "),
                id_param
            );

            let mut builder = sqlx::query_as::<_, TypeCategory>(&query);
            if let Some(ref label) = dto.label {
                builder = builder.bind(label);
            }
            if let Some(ref description) = dto.description {
                builder = builder.bind(description);
            }
            if let Some(is_hierarchical) = dto.is_hierarchical {
                builder = builder.bind(is_hierarchical);
            }
            builder = builder.bind(id);

            builder
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
