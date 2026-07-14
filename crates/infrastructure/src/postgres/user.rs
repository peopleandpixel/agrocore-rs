use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::{CreateUserDto, User, UpdateUserDto, UserRole, LoginDto, AuthResponse};
use agrocore_domain::repositories::{UserRepository, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, Result, SharedError};
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
            let row = sqlx::query_as::<_, (User,)>("SELECT row_to_json(users) FROM users WHERE id = $1 AND tenant_id = $2 AND is_active = true")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(row.map(|(u,)| u))
        })
    }

    fn find_by_id_visible(&self, tid: TenantId, id: Uuid, _user_id: Uuid, _roles: &[UserRole]) -> RepositoryFuture<Option<User>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM users WHERE tenant_id = $1::uuid AND is_active = true")
            .bind(tid.to_string())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<User> = sqlx::query_as(
                "SELECT * FROM users WHERE tenant_id = $1::uuid AND is_active = true ORDER BY lastname, firstname LIMIT $2 OFFSET $3")
            .bind(tid.to_string())
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            
            Ok(PaginatedResponse {
                data: items,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn find_all_visible(&self, tid: TenantId, p: Pagination, _user_id: Uuid, _roles: &[UserRole]) -> RepositoryFuture<PaginatedResponse<User>> {
        self.find_all(tid, p)
    }

    fn find_by_email(&self, email: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (User,)>("SELECT row_to_json(users) FROM users WHERE email = $1 AND is_active = true")
                .bind(email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(row.map(|(u,)| u))
        })
    }

    fn create(&self, tid: TenantId, dto: CreateUserDto, _by: Uuid) -> RepositoryFuture<User> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();
            let roles_json = serde_json::to_string(&dto.roles).unwrap_or_else(|_| "[]".to_string());
            
            let user = sqlx::query_as::<_, User>(
                r#"INSERT INTO users (id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, created_at, updated_at) 
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9) 
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(&dto.email)
            .bind(&dto.password)
            .bind(roles_json)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(user)
        })
    }

    fn update(&self, tid: TenantId, id: Uuid, dto: UpdateUserDto, _by: Uuid) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            
            let user = sqlx::query_as::<_, User>(
                r#"UPDATE users SET 
                    firstname = COALESCE($1, firstname),
                    lastname = COALESCE($2, lastname),
                    email = COALESCE($3, email),
                    roles = COALESCE($4, roles),
                    language = COALESCE($5, language),
                    color = COALESCE($6, color),
                    assigned_site_ids = COALESCE($7, assigned_site_ids),
                    is_active = COALESCE($8, is_active),
                    updated_at = $9
                   WHERE id = $10 AND tenant_id = $11
                   RETURNING *"#)
            .bind(&dto.firstname)
            .bind(&dto.lastname)
            .bind(&dto.email)
            .bind(dto.roles.as_ref().map(|r| serde_json::to_string(r).unwrap_or_else(|_| "[]".to_string())) as Option<String>)
            .bind(&dto.language)
            .bind(&dto.color)
            .bind(dto.assigned_site_ids.as_ref().map(|r| serde_json::to_string(r).unwrap_or_else(|_| "[]".to_string())))
            .bind(dto.is_active)
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
            let result = sqlx::query("UPDATE users SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3")
                .bind(Utc::now())
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn authenticate(&self, dto: LoginDto) -> RepositoryFuture<AuthResponse> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (User,)>("SELECT row_to_json(users) FROM users WHERE email = $1 AND is_active = true")
                .bind(&dto.email)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(row.map(|(u,)| u).map(|_u| AuthResponse {
                token: String::new(),
                refresh_token: String::new(),
            }).unwrap_or(AuthResponse {
                token: String::new(),
                refresh_token: String::new(),
            }))
        })
    }

    fn find_by_refresh_token(&self, _refresh_token: &str) -> RepositoryFuture<Option<User>> {
        Box::pin(async move { Ok(None) })
    }

    fn invalidate_refresh_token(&self, _user_id: Uuid) -> RepositoryFuture<bool> {
        Box::pin(async move { Ok(false) })
    }
}