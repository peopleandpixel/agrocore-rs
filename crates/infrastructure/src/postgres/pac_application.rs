use agrocore_domain::entities::finance::{PACApplication, CreatePACApplicationDto, UpdatePACApplicationDto};
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{PACApplicationRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPACApplicationRepo { pool: PgPool }
impl PgPACApplicationRepo { pub fn new(pool: PgPool) -> Self { Self { pool } } }

impl PACApplicationRepo for PgPACApplicationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PACApplication>("SELECT * FROM pac_applications WHERE tenant_id = $1::uuid AND id = $2")
                .bind(tid.to_string())
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<PACApplication>> {
        self.find_by_id(tid, id)
    }
    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pac_applications WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let data: Vec<PACApplication> = sqlx::query_as("SELECT * FROM pac_applications WHERE tenant_id = $1::uuid ORDER BY year DESC, created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
    fn create(&self, tid: TenantId, dto: CreatePACApplicationDto, _by: Uuid) -> RepositoryFuture<PACApplication> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PACApplication>(
                "INSERT INTO pac_applications (id, tenant_id, year, application_number, status, total_eligible_area, eco_schemes, documents_urls) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(Uuid::new_v4())
            .bind(tid.to_string())
            .bind(dto.year)
            .bind(dto.application_number)
            .bind(serde_json::to_value(agrocore_domain::entities::finance::PACStatus::Draft).unwrap())
            .bind(dto.total_eligible_area)
            .bind(serde_json::to_value(dto.eco_schemes).unwrap())
            .bind(serde_json::to_value(Vec::<String>::new()).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn update(&self, tid: TenantId, id: Uuid, dto: UpdatePACApplicationDto, _by: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        let pool = self.pool.clone();
        let self_clone = self.clone();
        Box::pin(async move {
            let mut query = String::from("UPDATE pac_applications SET updated_at = NOW()");
            let mut idx = 1;
            
            if dto.application_number.is_some() { query.push_str(&format!(", application_number = ${}", idx + 2)); idx += 1; }
            if dto.status.is_some() { query.push_str(&format!(", status = ${}", idx + 2)); idx += 1; }
            if dto.total_eligible_area.is_some() { query.push_str(&format!(", total_eligible_area = ${}", idx + 2)); idx += 1; }
            if dto.eco_schemes.is_some() { query.push_str(&format!(", eco_schemes = ${}", idx + 2)); idx += 1; }
            if dto.documents_urls.is_some() { query.push_str(&format!(", documents_urls = ${}", idx + 2)); }
            
            query.push_str(&format!(" WHERE tenant_id = $1::uuid AND id = $2 RETURNING *"));
            
            let mut q = sqlx::query_as::<_, PACApplication>(&query)
                .bind(tid.to_string())
                .bind(id);
            
            if let Some(v) = dto.application_number { q = q.bind(v); }
            if let Some(v) = dto.status { q = q.bind(serde_json::to_value(v).unwrap()); }
            if let Some(v) = dto.total_eligible_area { q = q.bind(v); }
            if let Some(v) = dto.eco_schemes { q = q.bind(serde_json::to_value(v).unwrap()); }
            if let Some(v) = dto.documents_urls { q = q.bind(serde_json::to_value(v).unwrap()); }
            
            q.fetch_optional(&pool).await.map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let res = sqlx::query("DELETE FROM pac_applications WHERE tenant_id = $1::uuid AND id = $2")
                .bind(tid.to_string())
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            Ok(res.rows_affected() > 0)
        })
    }
    fn find_by_year(&self, tid: TenantId, year: i32, p: Pagination) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pac_applications WHERE tenant_id = $1::uuid AND year = $2")
                .bind(tid.to_string())
                .bind(year)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let data: Vec<PACApplication> = sqlx::query_as("SELECT * FROM pac_applications WHERE tenant_id = $1::uuid AND year = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(year)
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            
            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            Ok(PaginatedResponse { data, total: total as u64, page, per_page, total_pages })
        })
    }
}
