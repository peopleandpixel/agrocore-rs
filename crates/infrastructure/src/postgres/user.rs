use agrocore_domain::entities::user::{AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User, UserRole};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, UserRepository};
use agrocore_shared::SharedError;
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

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use crate::postgres::error_mapper::map_db_error;
use crate::jwt::generate_jwt;

impl UserRepository for PgUserRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let user = sqlx::query_as::<_, User>(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.id = $1 AND u.tenant_id = $2 AND u.is_active = true"#)
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?;
            Ok(user)
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

    fn find_by_email(&self, email: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        let email = email.to_string();
        Box::pin(async move {
            sqlx::query_as::<_, User>(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.email = $1 AND u.is_active = true"#)
                .bind(email)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE tenant_id = $1::uuid AND is_active = true")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(map_db_error)?;

            let data: Vec<User> = sqlx::query_as(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.tenant_id = $1::uuid AND u.is_active = true 
                   LIMIT $2 OFFSET $3"#)
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(map_db_error)?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
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

    fn count_all(&self) -> RepositoryFuture<i64> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateUserDto, _by: Uuid) -> RepositoryFuture<User> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            let id = Uuid::new_v4();
            
            let password_hash = Argon2::default()
                .hash_password(dto.password.as_bytes())
                .map_err(|e| SharedError::Internal(format!("Hashing error: {}", e)))?
                .to_string();

            let user = sqlx::query_as::<_, User>(
                r#"INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(&dto.email)
            .bind(password_hash)
            .bind(serde_json::to_value(&dto.roles.unwrap_or_default()).unwrap())
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            
            // We need to return the user with assigned_site_ids (which is empty here)
            let mut user = user;
            user.assigned_site_ids = Some(vec![]);
            Ok(user)
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
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            
            sqlx::query(
                r#"UPDATE users SET 
                   firstname = COALESCE($1, firstname), 
                   lastname = COALESCE($2, lastname), 
                   is_active = COALESCE($3, is_active),
                   updated_at = NOW()
                   WHERE id = $4 AND tenant_id = $5"#)
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(dto.is_active)
            .bind(id)
            .bind(tid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

            if let Some(site_ids) = dto.assigned_site_ids {
                sqlx::query("DELETE FROM user_sites WHERE user_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_db_error)?;
                
                for site_id in site_ids {
                    sqlx::query("INSERT INTO user_sites (user_id, site_id) VALUES ($1, $2)")
                        .bind(id)
                        .bind(site_id)
                        .execute(&mut *tx)
                        .await
                        .map_err(map_db_error)?;
                }
            }

            let user = sqlx::query_as::<_, User>(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.id = $1 AND u.tenant_id = $2"#)
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            Ok(user)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE users SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn authenticate(&self, dto: LoginDto) -> RepositoryFuture<AuthResponse> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let user = sqlx::query_as::<_, User>(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.email = $1 AND u.is_active = true"#)
                .bind(&dto.email)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)?
                .ok_or_else(|| SharedError::Unauthorized("Invalid credentials".into()))?;

            Argon2::default()
                .verify_password(dto.password.as_bytes(), user.password_hash.as_str())
                .map_err(|_| SharedError::Unauthorized("Invalid credentials".into()))?;

            let token = generate_jwt(&user)
                .map_err(|e| SharedError::Internal(format!("Token generation failed: {}", e)))?;

            Ok(AuthResponse {
                token,
                user_id: user.id,
                tenant_id: user.tenant_id,
                firstname: user.firstname,
                lastname: user.lastname,
                roles: user.roles,
            })
        })
    }

    fn find_by_refresh_token(&self, refresh_token: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        let token = refresh_token.to_string();
        Box::pin(async move {
            sqlx::query_as::<_, User>(
                r#"SELECT u.*, 
                   COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json) as assigned_site_ids
                   FROM users u 
                   WHERE u.refresh_token = $1 AND u.is_active = true"#)
                .bind(token)
                .fetch_optional(&pool)
                .await
                .map_err(map_db_error)
        })
    }

    fn invalidate_refresh_token(&self, user_id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE users SET refresh_token = NULL, refresh_token_expires_at = NULL WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(map_db_error)
        })
    }

    fn update_refresh_token(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        let token = token.to_string();
        Box::pin(async move {
            sqlx::query("UPDATE users SET refresh_token = $1, refresh_token_expires_at = $2 WHERE id = $3")
                .bind(token)
                .bind(expires_at)
                .bind(user_id)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
