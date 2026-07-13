use agrocore_domain::entities::plant_protection::{CreatePlantProtectionDto, PlantProtectionRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{RepositoryFuture, PlantProtectionRecordRepo};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

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
            let row = sqlx::query_as::<_, (PlantProtectionRecord,)>("SELECT row_to_json(plant_protection_records) FROM plant_protection_records WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;
            row.map(|(r,)| r).ok_or_else(|| SharedError::NotFound.to_error())
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

    pub fn find_all(
        &self,
        tid: TenantId,
        p: agrocore_shared::Pagination,
    ) -> RepositoryFuture<agrocore_shared::PaginatedResponse<PlantProtectionRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plant_protection_records WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<PlantProtectionRecord> = sqlx::query_as("SELECT * FROM plant_protection_records WHERE tenant_id = $1::uuid ORDER BY application_date DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(agrocore_shared::PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreatePlantProtectionDto) -> Fut<PlantProtectionRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let record = PlantProtectionRecord {
                id: Uuid::new_v4(),
                tenant_id: tid,
                site_id: dto.site_id,
                order_id: dto.order_id,
                product_name: dto.product_name,
                active_substance: dto.active_substance,
                dosage_per_ha: dto.dosage_per_ha,
                total_quantity: dto.total_quantity,
                area_ha: dto.area_ha,
                application_date: dto.application_date,
                pre_harvest_days: dto.pre_harvest_days,
                re_entry_days: dto.re_entry_days,
                weather_conditions: dto.weather_conditions,
                applicator_license: dto.applicator_license,
                created_at: now,
            };
            
            let rec = sqlx::query_as::<_, PlantProtectionRecord>(
                "INSERT INTO plant_protection_records (id, tenant_id, site_id, order_id, product_name, active_substance, dosage_per_ha, total_quantity, area_ha, application_date, pre_harvest_days, re_entry_days, weather_conditions, applicator_license, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING *"
            )
            .bind(record.id)
            .bind(tid.to_string())
            .bind(record.site_id.to_string())
            .bind(record.order_id.map(|u| u.to_string()))
            .bind(&record.product_name)
            .bind(&record.active_substance)
            .bind(record.dosage_per_ha)
            .bind(record.total_quantity)
            .bind(record.area_ha)
            .bind(record.application_date)
            .bind(record.pre_harvest_days)
            .bind(record.re_entry_days)
            .bind(&record.weather_conditions)
            .bind(&record.applicator_license)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(rec)
        })
    }
}