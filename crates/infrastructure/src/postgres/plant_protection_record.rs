use agrocore_domain::entities::plant_protection::{
    ApplicatorLicense, CreateApplicatorLicenseDto, CreatePlantProtectionDto, PlantProtectionRecord,
    UpdateApplicatorLicenseDto, UpdatePlantProtectionDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, PlantProtectionRecordRepo, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPlantProtectionRecordRepo {
    pool: PgPool,
}

impl PgPlantProtectionRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PlantProtectionRecordRepo for PgPlantProtectionRecordRepo {
    fn find_by_id(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PlantProtectionRecord>(
                "SELECT * FROM plant_protection_records WHERE id = $1 AND tenant_id = $2",
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
    ) -> RepositoryFuture<PaginatedResponse<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM plant_protection_records WHERE tenant_id = $1",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PlantProtectionRecord> = sqlx::query_as("SELECT * FROM plant_protection_records WHERE tenant_id = $1 ORDER BY application_date DESC LIMIT $2 OFFSET $3")
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

    fn create(
        &self,
        tid: TenantId,
        dto: CreatePlantProtectionDto,
        _by: Uuid,
    ) -> RepositoryFuture<PlantProtectionRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, PlantProtectionRecord>(
                r#"INSERT INTO plant_protection_records (id, tenant_id, site_id, order_id, product_name, active_substance, dosage_per_ha, total_quantity, area_ha, application_date, pre_harvest_days, re_entry_days, weather_conditions, applicator_license, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.site_id)
            .bind(dto.order_id)
            .bind(&dto.product_name)
            .bind(&dto.active_substance)
            .bind(dto.dosage_per_ha)
            .bind(dto.total_quantity)
            .bind(dto.area_ha)
            .bind(dto.application_date)
            .bind(dto.pre_harvest_days)
            .bind(dto.re_entry_days)
            .bind(&dto.weather_conditions)
            .bind(&dto.applicator_license)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePlantProtectionDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PlantProtectionRecord>(
                r#"UPDATE plant_protection_records SET product_name = COALESCE($1, product_name)
                   WHERE id = $2 AND tenant_id = $3 RETURNING *"#,
            )
            .bind(&dto.product_name)
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
            sqlx::query("DELETE FROM plant_protection_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_applicator_license_by_user(
        &self,
        _tid: TenantId,
        _user_id: Uuid,
    ) -> RepositoryFuture<Option<ApplicatorLicense>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }

    fn create_applicator_license(
        &self,
        _tid: TenantId,
        _dto: CreateApplicatorLicenseDto,
    ) -> RepositoryFuture<ApplicatorLicense> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }

    fn update_applicator_license(
        &self,
        _tid: TenantId,
        _id: Uuid,
        _dto: UpdateApplicatorLicenseDto,
    ) -> RepositoryFuture<Option<ApplicatorLicense>> {
        Box::pin(async move { Err(SharedError::Internal("Not implemented".into())) })
    }
}
