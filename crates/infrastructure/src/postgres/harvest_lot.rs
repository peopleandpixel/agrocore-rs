use agrocore_domain::entities::harvest::{CreateHarvestLotDto, HarvestLot, UpdateHarvestLotDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    HarvestLotRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

#[derive(Clone)]
pub struct PgHarvestLotRepo {
    pool: PgPool,
}

impl PgHarvestLotRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl HarvestLotRepo for PgHarvestLotRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<HarvestLot>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestLot>(
                "SELECT * FROM harvest_lots WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<HarvestLot>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM harvest_lots WHERE tenant_id = $1::uuid")
                    .bind(tid.to_string())
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<HarvestLot> = sqlx::query_as("SELECT * FROM harvest_lots WHERE tenant_id = $1::uuid ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(tid.to_string())
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

    fn find_by_season(
        &self,
        tid: TenantId,
        season_id: Uuid,
        p: Pagination,
    ) -> Fut<PaginatedResponse<HarvestLot>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM harvest_lots WHERE tenant_id = $1::uuid AND season_id = $2",
            )
            .bind(tid.to_string())
            .bind(season_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<HarvestLot> = sqlx::query_as("SELECT * FROM harvest_lots WHERE tenant_id = $1::uuid AND season_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid.to_string())
                .bind(season_id)
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

    fn create(&self, tid: TenantId, dto: CreateHarvestLotDto, _by: Uuid) -> Fut<HarvestLot> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, HarvestLot>(
                r#"INSERT INTO harvest_lots (id, tenant_id, season_id, lot_number, site_ids, crop_type, variety, quality_target, status, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid.to_string())
            .bind(dto.season_id)
            .bind(&dto.lot_number)
            .bind(&dto.site_ids)
            .bind(&dto.crop_type)
            .bind(&dto.variety)
            .bind(&dto.quality_target)
            .bind(serde_json::to_value(agrocore_domain::entities::harvest::LotStatus::Collecting).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestLotDto,
        _by: Uuid,
    ) -> Fut<Option<HarvestLot>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestLot>(
                r#"UPDATE harvest_lots SET 
                    lot_number = COALESCE($1, lot_number),
                    site_ids = COALESCE($2, site_ids),
                    crop_type = COALESCE($3, crop_type),
                    variety = COALESCE($4, variety),
                    quality_target = COALESCE($5, quality_target),
                    total_weight_kg = COALESCE($6, total_weight_kg),
                    status = COALESCE($7, status),
                    updated_at = NOW()
                   WHERE id = $8 AND tenant_id = $9 RETURNING *"#,
            )
            .bind(&dto.lot_number)
            .bind(&dto.site_ids)
            .bind(&dto.crop_type)
            .bind(&dto.variety)
            .bind(&dto.quality_target)
            .bind(dto.total_weight_kg)
            .bind(
                dto.status
                    .as_ref()
                    .map(|s| serde_json::to_value(s).unwrap()),
            )
            .bind(id)
            .bind(tid.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM harvest_lots WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid.to_string())
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
