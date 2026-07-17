use agrocore_domain::entities::olive::{
    CreateOliveOilRecordDto, OliveOilRecord, UpdateOliveOilRecordDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    OliveOilRecordRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgOliveOilRecordRepo {
    pool: PgPool,
}

impl PgOliveOilRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl OliveOilRecordRepo for PgOliveOilRecordRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<OliveOilRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveOilRecord>(
                "SELECT * FROM olive_oil_records WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveOilRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM olive_oil_records WHERE tenant_id = $1",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<OliveOilRecord> = sqlx::query_as(
                "SELECT * FROM olive_oil_records WHERE tenant_id = $1 LIMIT $2 OFFSET $3",
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

    fn find_by_grove(
        &self,
        tid: TenantId,
        grove_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<OliveOilRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM olive_oil_records WHERE tenant_id = $1 AND grove_id = $2")
                .bind(tid)
                .bind(grove_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<OliveOilRecord> = sqlx::query_as("SELECT * FROM olive_oil_records WHERE tenant_id = $1 AND grove_id = $2 LIMIT $3 OFFSET $4")
                .bind(tid)
                .bind(grove_id)
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

    fn create(
        &self,
        tid: TenantId,
        dto: CreateOliveOilRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<OliveOilRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, OliveOilRecord>(
                r#"INSERT INTO olive_oil_records (id, tenant_id, grove_id, harvest_year, oil_grade, acidity_pct, peroxide_value, sensory_score, liters_produced, mill_name, lot_number, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.grove_id)
            .bind(dto.harvest_year)
            .bind(serde_json::to_value(&dto.oil_grade).unwrap())
            .bind(dto.acidity_pct)
            .bind(dto.peroxide_value)
            .bind(dto.sensory_score)
            .bind(dto.liters_produced)
            .bind(&dto.mill_name)
            .bind(&dto.lot_number)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateOliveOilRecordDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<OliveOilRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, OliveOilRecord>(
                r#"UPDATE olive_oil_records SET harvest_year = COALESCE($1, harvest_year), acidity_pct = COALESCE($2, acidity_pct)
                   WHERE id = $3 AND tenant_id = $4 RETURNING *"#)
            .bind(dto.harvest_year)
            .bind(dto.acidity_pct)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM olive_oil_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
