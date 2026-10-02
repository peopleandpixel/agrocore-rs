use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::{
    AuthResponse, CreateUserDto, LoginDto, UpdateUserDto, User, UserRole,
};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, UserRepository,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgUserRepo);

use crate::jwt::generate_jwt;
use crate::postgres::error_mapper::map_db_error;
use crate::postgres::tenant_pool::TenantPool;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::phc::SaltString;
use rand::rng;

/// Query result struct for user queries with ::float8 casts
/// sqlx validates this against the SQL query output, not the database schema
#[derive(sqlx::FromRow, Debug)]
struct UserRow {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password_hash: String,
    #[sqlx(json)]
    pub roles: Vec<UserRole>,
    pub is_active: bool,
    pub internal_cost_per_hour: Option<f64>,
    pub external_cost_per_hour: Option<f64>,
    pub color: Option<String>,
    pub language: Option<String>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
    pub refresh_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[sqlx(json(nullable))]
    pub assigned_site_ids: Option<Vec<Uuid>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            firstname: row.firstname,
            lastname: row.lastname,
            email: row.email,
            password_hash: row.password_hash,
            roles: row.roles,
            is_active: row.is_active,
            internal_cost_per_hour: row.internal_cost_per_hour,
            external_cost_per_hour: row.external_cost_per_hour,
            color: row.color,
            language: row.language,
            last_login: row.last_login,
            refresh_token: row.refresh_token,
            refresh_token_expires_at: row.refresh_token_expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            assigned_site_ids: row.assigned_site_ids,
        }
    }
}

/// Shared SQL query fragment for selecting a user with assigned site IDs via LEFT JOIN.
/// Replaces the correlated subquery `COALESCE((SELECT json_agg(site_id) FROM user_sites WHERE user_id = u.id), '[]'::json)`
/// with a LEFT JOIN + GROUP BY approach to eliminate per-row subquery execution (N+1 avoidance at the SQL level).
/// Uses ::float8 cast to convert NUMERIC(10,2) to float8 for sqlx compatibility with f64.
const USER_SELECT_FIELDS: &str = r#"SELECT u.id, u.tenant_id, u.firstname, u.lastname, u.email, u.password_hash, u.roles, u.is_active, 
                   u.internal_cost_per_hour::float8 as internal_cost_per_hour, u.external_cost_per_hour::float8 as external_cost_per_hour,
                   u.color, u.language, u.last_login, u.refresh_token, u.refresh_token_expires_at, u.created_at, u.updated_at,
                   COALESCE(json_agg(us.site_id) FILTER (WHERE us.site_id IS NOT NULL), '[]'::json) as assigned_site_ids
                   FROM users u
                   LEFT JOIN user_sites us ON u.id = us.user_id"#;

impl UserRepository for PgUserRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<User>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let user: Option<UserRow> = sqlx::query_as(&format!(
                "{} WHERE u.id = $1 AND u.tenant_id = $2 AND u.is_active = true GROUP BY u.id",
                USER_SELECT_FIELDS
            ))
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(map_db_error)?;
            Ok(user.map(Into::into))
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

    /// Look a user up by email for the login path.
    ///
    /// This runs *before* the tenant is known: authentication has not happened
    /// yet, so there is no `app.current_tenant_id` to pin. RLS would therefore
    /// hide every row. The lookup is instead scoped to what a login may see -
    /// active users only - and the caller pins the returned user's tenant
    /// before touching anything else.
    ///
    /// This is the one place where a tenant-agnostic read is required by
    /// design. It returns the tenant id with the user, so the caller never has
    /// to search again.
    fn find_by_email(&self, email: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        let email = email.to_string();
        Box::pin(async move {
            let mut conn = pool.acquire().await.map_err(map_db_error)?;
            // `SET LOCAL ROLE` is only effective inside a transaction block, so
            // BEGIN has to come first. Ordering these the other way round would
            // silently leave the privileged role active for the whole
            // connection.
            sqlx::query("BEGIN")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
            sqlx::query("SET LOCAL ROLE agrocore_auth")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
            let user: Option<UserRow> = sqlx::query_as(&format!(
                "{} WHERE u.email = $1 AND u.is_active = true GROUP BY u.id",
                USER_SELECT_FIELDS
            ))
            .bind(email)
            .fetch_optional(&mut *conn)
            .await
            .map_err(map_db_error)?;
            // End the transaction so the connection is clean when it returns to
            // the pool; `SET LOCAL ROLE` is scoped to it.
            sqlx::query("COMMIT").execute(&mut *conn).await.ok();
            Ok(user.map(Into::into))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<User>> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND is_active = true",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)?;

            let data: Vec<User> = sqlx::query_as::<_, UserRow>(&format!(
                "{} WHERE u.tenant_id = $1 AND u.is_active = true GROUP BY u.id LIMIT $2 OFFSET $3",
                USER_SELECT_FIELDS
            ))
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(map_db_error)?
            .into_iter()
            .map(Into::into)
            .collect();

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

    fn create(&self, tid: TenantId, dto: CreateUserDto, _by: Uuid) -> RepositoryFuture<User> {
        let pool = TenantPool::new(&self.pool, tid.0);
        let password_hash = dto.password;
        let roles = dto.roles.unwrap_or_default();
        let internal_cost_per_hour = dto.internal_cost_per_hour;
        let external_cost_per_hour = dto.external_cost_per_hour;
        let language = dto.language;

        Box::pin(async move {
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(password_hash.as_bytes())
                .map_err(|e| SharedError::Internal(format!("Password hashing failed: {}", e)))?
                .to_string();

            let user = sqlx::query_as::<_, User>(
                r#"INSERT INTO users (tenant_id, firstname, lastname, email, password_hash, roles, is_active, internal_cost_per_hour, external_cost_per_hour, color, language, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8, $9, $10, NOW(), NOW())
                   RETURNING id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, 
                           internal_cost_per_hour::float8 as internal_cost_per_hour, external_cost_per_hour::float8 as external_cost_per_hour,
                           color, language, last_login, refresh_token, refresh_token_expires_at, created_at, updated_at,
                           '[]'::json as assigned_site_ids"#
            )
            .bind(tid)
            .bind(dto.firstname)
            .bind(dto.lastname)
            .bind(dto.email)
            .bind(password_hash)
            .bind(serde_json::to_value(&roles).unwrap())
            .bind(internal_cost_per_hour)
            .bind(external_cost_per_hour)
            .bind(None::<String>) // color
            .bind(language)
            .fetch_one(&pool)
            .await
            .map_err(map_db_error)?;

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
        // Extract all owned data from dto and self to avoid lifetime issues
        let pool = TenantPool::new(&self.pool, tid.0);
        let firstname = dto.firstname;
        let lastname = dto.lastname;
        let email = dto.email;
        let is_active = dto.is_active;
        let internal_cost_per_hour = dto.internal_cost_per_hour;
        let external_cost_per_hour = dto.external_cost_per_hour;
        let color = dto.color;
        let language = dto.language;
        let roles = dto.roles;

        Box::pin(async move {
            // Build SET clause dynamically
            let mut set_clauses = Vec::new();
            let mut param_idx = 1;

            if firstname.is_some() {
                set_clauses.push(format!("firstname = ${}", param_idx));
                param_idx += 1;
            }
            if lastname.is_some() {
                set_clauses.push(format!("lastname = ${}", param_idx));
                param_idx += 1;
            }
            if email.is_some() {
                set_clauses.push(format!("email = ${}", param_idx));
                param_idx += 1;
            }
            if is_active.is_some() {
                set_clauses.push(format!("is_active = ${}", param_idx));
                param_idx += 1;
            }
            if internal_cost_per_hour.is_some() {
                set_clauses.push(format!("internal_cost_per_hour = ${}", param_idx));
                param_idx += 1;
            }
            if external_cost_per_hour.is_some() {
                set_clauses.push(format!("external_cost_per_hour = ${}", param_idx));
                param_idx += 1;
            }
            if color.is_some() {
                set_clauses.push(format!("color = ${}", param_idx));
                param_idx += 1;
            }
            if language.is_some() {
                set_clauses.push(format!("language = ${}", param_idx));
                param_idx += 1;
            }
            if roles.is_some() {
                set_clauses.push(format!("roles = ${}", param_idx));
                param_idx += 1;
            }

            set_clauses.push("updated_at = NOW()".to_string());

            // Build query with all possible parameters (use Option for optional fields)
            let query = format!(
                r#"UPDATE users SET {} WHERE id = ${} AND tenant_id = ${}
                   RETURNING id, tenant_id, firstname, lastname, email, password_hash, roles, is_active, 
                           internal_cost_per_hour::float8 as internal_cost_per_hour, external_cost_per_hour::float8 as external_cost_per_hour,
                           color, language, last_login, refresh_token, refresh_token_expires_at, created_at, updated_at,
                           COALESCE(json_agg(us.site_id) FILTER (WHERE us.site_id IS NOT NULL), '[]'::json) as assigned_site_ids
                   FROM users u LEFT JOIN user_sites us ON u.id = us.user_id GROUP BY u.id"#,
                set_clauses.join(", "),
                param_idx,
                param_idx + 1
            );

            let mut q = sqlx::query_as::<_, User>(&query);

            // Bind parameters in order
            if let Some(v) = firstname {
                q = q.bind(v);
            }
            if let Some(v) = lastname {
                q = q.bind(v);
            }
            if let Some(v) = email {
                q = q.bind(v);
            }
            if let Some(v) = is_active {
                q = q.bind(v);
            }
            if let Some(v) = internal_cost_per_hour {
                q = q.bind(v);
            }
            if let Some(v) = external_cost_per_hour {
                q = q.bind(v);
            }
            if let Some(v) = color {
                q = q.bind(v);
            }
            if let Some(v) = language {
                q = q.bind(v);
            }
            if let Some(v) = roles {
                q = q.bind(serde_json::to_string(&v).unwrap());
            }

            q = q.bind(id).bind(tid);

            let user = q.fetch_optional(&pool).await.map_err(map_db_error)?;

            Ok(user)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = TenantPool::new(&self.pool, tid.0);
        Box::pin(async move {
            let result = sqlx::query("DELETE FROM users WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map_err(map_db_error)?;
            Ok(result.rows_affected() > 0)
        })
    }

    fn authenticate(&self, dto: LoginDto) -> RepositoryFuture<AuthResponse> {
        let pool = self.pool.clone();
        let email = dto.email;
        let password = dto.password;

        Box::pin(async move {
            // The user lookup runs before the tenant is known - the pin is read
            // *from* this row - so `users_select` (`tenant_id =
            // get_current_tenant_id()`) cannot be satisfied and would hide every
            // user. `agrocore_auth` has a SELECT-only policy on `users` that
            // admits exactly this step and nothing else.
            //
            // BEGIN comes first: `SET LOCAL ROLE` is a no-op outside a
            // transaction block, and getting that wrong would leave the
            // privileged role active on the whole pooled connection.
            let mut conn = pool.acquire().await.map_err(map_db_error)?;
            sqlx::query("BEGIN")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
            sqlx::query("SET LOCAL ROLE agrocore_auth")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;

            let user: User = sqlx::query_as(&format!(
                "{} WHERE u.email = $1 AND u.is_active = true GROUP BY u.id",
                USER_SELECT_FIELDS
            ))
            .bind(&email)
            .fetch_one(&mut *conn)
            .await
            .map_err(map_db_error)?;

            // Hand the connection back in its normal (RLS-enforcing) role
            // before the tenant update below.
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;

            let argon2 = Argon2::default();
            let parsed_hash = PasswordHash::new(&user.password_hash)
                .map_err(|e| SharedError::Internal(format!("Invalid password hash: {}", e)))?;
            argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| SharedError::Unauthorized("Invalid credentials".to_string()))?;

            let token = generate_jwt(user.id, user.tenant_id.0, &user.roles)
                .map_err(|e| SharedError::Internal(format!("JWT generation failed: {}", e)))?;
            let refresh_token = generate_jwt(user.id, user.tenant_id.0, &user.roles)
                .map_err(|e| SharedError::Internal(format!("JWT generation failed: {}", e)))?;

            // Update refresh token in DB. Pinned to the user's own tenant, so
            // RLS restricts the write to that row.
            let pinned = TenantPool::new(&pool, user.tenant_id.0);
            sqlx::query(
                "UPDATE users SET refresh_token = $1, refresh_token_expires_at = $2, last_login = NOW() WHERE id = $3"
            )
            .bind(&refresh_token)
            .bind(chrono::Utc::now() + chrono::Duration::days(30))
            .bind(user.id)
            .execute(&pinned)
            .await
            .map_err(map_db_error)?;

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

    fn count_all(&self) -> RepositoryFuture<i64> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = true")
                .fetch_one(&pool)
                .await
                .map_err(map_db_error)
        })
    }

    fn find_by_refresh_token(&self, refresh_token: &str) -> RepositoryFuture<Option<User>> {
        let pool = self.pool.clone();
        let refresh_token = refresh_token.to_string();
        Box::pin(async move {
            // The refresh token is itself the credential, so the tenant is not
            // known before this lookup - the same position as `authenticate`.
            // `agrocore_auth` is the restricted role that admits it.
            let mut conn = pool.acquire().await.map_err(map_db_error)?;
            sqlx::query("BEGIN")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
            sqlx::query("SET LOCAL ROLE agrocore_auth")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;

            // Expiry and the active flag are filtered in SQL rather than in
            // Rust, so an expired token is indistinguishable from an unknown one
            // at this layer and cannot be mistaken for a valid session.
            let user: Option<UserRow> = sqlx::query_as(&format!(
                "{} WHERE u.refresh_token = $1 AND u.refresh_token_expires_at > NOW() \
                 AND u.is_active = true GROUP BY u.id",
                USER_SELECT_FIELDS
            ))
            .bind(&refresh_token)
            .fetch_optional(&mut *conn)
            .await
            .map_err(map_db_error)?;

            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;

            Ok(user.map(User::from))
        })
    }

    fn invalidate_refresh_token(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
    ) -> RepositoryFuture<bool> {
        // Pinned for the same reason as `update_refresh_token`: `users_update`
        // is tenant-scoped, so an unpinned write would silently match no rows
        // and report `false` - a revocation that appears to have happened.
        let pool = TenantPool::new(&self.pool, tenant_id.0);
        Box::pin(async move {
            // `refresh_token IS NOT NULL` keeps this idempotent: invalidating
            // an already-cleared token reports "nothing changed" instead of
            // pretending to have revoked something.
            let result = sqlx::query(
                "UPDATE users SET refresh_token = NULL, refresh_token_expires_at = NULL \
                 WHERE id = $1 AND refresh_token IS NOT NULL",
            )
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(result.rows_affected() > 0)
        })
    }

    fn update_refresh_token(
        &self,
        tenant_id: TenantId,
        user_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> RepositoryFuture<bool> {
        // Pinned: `users_update` requires `tenant_id =
        // get_current_tenant_id()`, so without the pin the UPDATE would match no
        // rows and report `false` - which the login handler correctly treats as
        // "refresh token could not be persisted".
        let pool = TenantPool::new(&self.pool, tenant_id.0);
        let token = token.to_string();
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE users SET refresh_token = $1, refresh_token_expires_at = $2, \
                 updated_at = NOW() WHERE id = $3",
            )
            .bind(&token)
            .bind(expires_at)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(map_db_error)?;

            Ok(result.rows_affected() > 0)
        })
    }
}
