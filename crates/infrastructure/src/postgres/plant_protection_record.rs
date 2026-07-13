use agrocore_domain::entities::plant_protection::{CreatePlantProtectionDto, PlantProtectionRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{PlantProtectionRecordRepo, PaginatedResponse, Pagination, RepositoryFuture};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
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

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, (PlantProtectionRecord,)>("SELECT row_to_json(plant_protection_records) FROM plant_protection_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?
                .map(|(r,)| r)
                .ok_or_else(|| SharedError::NotFound.to_error())
        })
    }

    pub fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<Option<PlantProtectionRecord>> {
        self.find_by_id(tid, id)
    }

    pub fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plant_protection_records WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PlantProtectionRecord> = sqlx::query_as("SELECT * FROM plant_protection_records WHERE tenant_id = $1::uuid ORDER BY application_date DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(per_page as i32)
                .bind(offset as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let total_pages = if total == 0 { 0 } else { (total as f64 / per_page as f64).ceil() as u64 };
            
            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreatePlantProtectionDto) -> RepositoryFuture<PlantProtectionRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            sqlx::query_as::<_, PlantProtectionRecord>(
                r#"INSERT INTO plant_protection_records (tenant_id, site_id, order_id, product_name, active_substance, dosage_per_ha, total_quantity, area_ha, application_date, pre_harvest_days, re_entry_days, weather_conditions, applicator_license, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                   RETURNING *"#)
            .bind(tid.to_string())
            .bind(dto.site_id.to_string())
            .bind(dto.order_id.map(|u| u.to_string()))
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
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}