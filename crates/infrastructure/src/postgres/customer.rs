use agrocore_domain::entities::customer::{CreateCustomerDto, Customer, UpdateCustomerDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    CustomerRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgCustomerRepo);

impl CustomerRepository for PgCustomerRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Customer>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Customer>(
                "SELECT * FROM customers WHERE id = $1 AND tenant_id = $2 AND is_active = true",
            )
            .bind(id)
            .bind(tid.0)
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
    ) -> RepositoryFuture<Option<Customer>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        let _ = user_id;
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);
            if can_see_all {
                sqlx::query_as::<_, Customer>(
                    "SELECT * FROM customers WHERE id = $1 AND tenant_id = $2 AND is_active = true",
                )
                .bind(id)
                .bind(tid.0)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, Customer>(
                    "SELECT * FROM customers WHERE id = $1 AND tenant_id = $2 AND is_active = true",
                )
                .bind(id)
                .bind(tid.0)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Customer>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM customers WHERE tenant_id = $1 AND is_active = true",
            )
            .bind(tid.0)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Customer> = sqlx::query_as(
                "SELECT * FROM customers WHERE tenant_id = $1 AND is_active = true ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(tid.0)
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

    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Customer>> {
        self.find_all(tid, p)
    }

    fn find_by_customer_number(
        &self,
        tid: TenantId,
        number: &str,
    ) -> RepositoryFuture<Option<Customer>> {
        let pool = self.pool.clone();
        let number_owned = number.to_owned();
        Box::pin(async move {
            sqlx::query_as::<_, Customer>(
                "SELECT * FROM customers WHERE customer_number = $1 AND tenant_id = $2 AND is_active = true",
            )
            .bind(&number_owned)
            .bind(tid.0)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateCustomerDto,
        by: Uuid,
    ) -> RepositoryFuture<Customer> {
        let pool = self.pool.clone();
        let _ = by; // by is used for audit in the future
        Box::pin(async move {
            sqlx::query_as::<_, Customer>(
                r#"INSERT INTO customers (
                    id, tenant_id, name, email, phone, address, company,
                    customer_number, vat_rate, payment_terms,
                    preferred_delivery_location, preferences,
                    created_at, updated_at, is_active
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW(), true
                ) RETURNING *"#,
            )
            .bind(Uuid::new_v4())
            .bind(tid.0)
            .bind(dto.name)
            .bind(dto.email)
            .bind(dto.phone)
            .bind(dto.address)
            .bind(dto.company)
            .bind(dto.customer_number)
            .bind(dto.vat_rate)
            .bind(dto.payment_terms)
            .bind(dto.preferred_delivery_location)
            .bind(dto.preferences)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateCustomerDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Customer>> {
        let _ = by; // by is used for audit in the future
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Customer>(
                r#"UPDATE customers SET
                    name = COALESCE($1, name),
                    email = COALESCE($2, email),
                    phone = COALESCE($3, phone),
                    address = COALESCE($4, address),
                    company = COALESCE($5, company),
                    customer_number = COALESCE($6, customer_number),
                    vat_rate = COALESCE($7, vat_rate),
                    payment_terms = COALESCE($8, payment_terms),
                    preferred_delivery_location = COALESCE($9, preferred_delivery_location),
                    preferences = COALESCE($10, preferences),
                    is_active = COALESCE($11, is_active),
                    updated_at = NOW()
                WHERE id = $12 AND tenant_id = $13
                RETURNING *"#,
            )
            .bind(dto.name)
            .bind(dto.email)
            .bind(dto.phone)
            .bind(dto.address)
            .bind(dto.company)
            .bind(dto.customer_number)
            .bind(dto.vat_rate)
            .bind(dto.payment_terms)
            .bind(dto.preferred_delivery_location)
            .bind(dto.preferences)
            .bind(dto.is_active)
            .bind(id)
            .bind(tid.0)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn search(
        &self,
        tid: TenantId,
        query: &str,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<Customer>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        let like_pattern = format!("%{}%", query);

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM customers WHERE tenant_id = $1 AND is_active = true \
                 AND (name ILIKE $2 OR email ILIKE $2 OR company ILIKE $2 OR customer_number ILIKE $2)",
            )
            .bind(tid.0)
            .bind(&like_pattern)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Customer> = sqlx::query_as(
                "SELECT * FROM customers WHERE tenant_id = $1 AND is_active = true \
                 AND (name ILIKE $2 OR email ILIKE $2 OR company ILIKE $2 OR customer_number ILIKE $2) \
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(tid.0)
            .bind(&like_pattern)
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

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE customers SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.0)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
