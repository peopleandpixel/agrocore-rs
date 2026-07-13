use agrocore_domain::entities::compliance::{CreateFertilizerRecordDto, FertilizerRecord};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{RepositoryFuture, FertilizerRecordRepo};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgFertilizerRecordRepo {
    pool: PgPool,
}

impl PgFertilizerRecordRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<FertilizerRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let row = sqlx::query_as::<_, (FertilizerRecord,)>("SELECT row_to_json(fertilizer_records) FROM fertilizer_records WHERE id = $1 AND tenant_id = $2")
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
    ) -> RepositoryFuture<Option<FertilizerRecord>> {
        self.find_by_id(tid, id)
    }

    pub fn find_all(
        &self,
        tid: TenantId,
        p: agrocore_shared::Pagination,
    ) -> RepositoryFuture<agrocore_shared::PaginatedResponse<FertilizerRecord>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fertilizer_records WHERE tenant_id = $1::uuid")
                .bind(tid.to_string())
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<FertilizerRecord> = sqlx::query_as("SELECT * FROM fertilizer_records WHERE tenant_id = $1::uuid ORDER BY application_date DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
                .bind(p.limit as i32)
                .bind(((p.page - 1) * p.limit) as i32)
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(agrocore_shared::PaginatedResponse { items, total, page: p.page, limit: p.limit })
        })
    }

    pub fn create(&self, tid: TenantId, dto: CreateFertilizerRecordDto) -> Fut<FertilizerRecord> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let record = FertilizerRecord {
                id: Uuid::new_v4(),
                tenant_id: tid,
                site_id: dto.site_id,
                order_id: dto.order_id,
                product_name: dto.product_name,
                nutrient_n: dto.nutrient_n,
                nutrient_p: dto.nutrient_p,
                nutrient_k: dto.nutrient_k,
                quantity_kg: dto.quantity_kg,
                area_ha: dto.area_ha,
                application_date: dto.application_date,
                created_at: now,
            };
            
            let rec = sqlx::query_as::<_, FertilizerRecord>(
                "INSERT INTO fertilizer_records (id, tenant_id, site_id, order_id, product_name, nutrient_n, nutrient_p, nutrient_k, quantity_kg, area_ha, application_date, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *"
            )
            .bind(record.id)
            .bind(tid.to_string())
            .bind(record.site_id.to_string())
            .bind(record.order_id.map(|u| u.to_string()))
            .bind(&record.product_name)
            .bind(record.nutrient_n)
            .bind(record.nutrient_p)
            .bind(record.nutrient_k)
            .bind(record.quantity_kg)
            .bind(record.area_ha)
            .bind(record.application_date)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;
            
            Ok(rec)
        })
    }
}