use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::{CreateUserDto, User, UpdateUserDto, UserRole};
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, UserRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use serde_json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserRepository for PgUserRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (User,)>("SELECT row_to_json(users) FROM users WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            row.map(|(u,)| u).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<User>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<User> = sqlx::query_as("SELECT * FROM users WHERE tenant_id = $1::uuid ORDER BY lastname, firstname LIMIT $2 OFFSET $3")
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

    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<User>> {
        self.find_all(tid, p)
    }

    fn create(&self, tid: TenantId, dto: CreateUserDto, _by: Uuid) -> RepositoryFuture<User> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();
            
            let user = sqlx::query_as::<_, User>(
                "INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, roles, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                 RETURNING *"
            )
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(&dto.email)
            .bind(&dto.password)
            .bind(serde_json::to_string(&dto.roles).unwrap_or_else(|_| "[]".to_string()))
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(user)
        })
    }

    fn find_by_email(&self, tid: TenantId, email: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (User,)>("SELECT row_to_json(users) FROM users WHERE tenant_id = $1 AND email = $2")
                .bind(tid.to_string())
                .bind(email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            row.map(|(u,)| u).ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateUserDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let user = sqlx::query_as::<_, User>(
                r#"UPDATE users SET 
                    firstname = COALESCE($1, firstname),
                    lastname = COALESCE($2, lastname),
                    email = COALESCE($3, email),
                    roles = COALESCE($4, roles),
                    updated_at = $5
                   WHERE id = $6 AND tenant_id = $7
                   RETURNING *"#
            )
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(&dto.email)
            .bind(dto.roles.as_ref().map(|r| serde_json::to_string(r).unwrap_or_else(|_| "[]".to_string())))
            .bind(now)
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(user)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM users WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(result.rows_affected() > 0)
        })
    }
}