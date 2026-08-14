use agrocore_domain::entities::harvest::{
    CreateHarvestDeliveryDto, HarvestDelivery, UpdateHarvestDeliveryDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{HarvestDeliveryRepo, RepositoryFuture};
use agrocore_shared::{PaginatedResponse, Pagination, SharedError};
use sqlx::PgPool;
use uuid::Uuid;

type Fut<T> = RepositoryFuture<T>;

agrocore_shared::pg_repo!(PgHarvestDeliveryRepo);

impl HarvestDeliveryRepo for PgHarvestDeliveryRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> Fut<Option<HarvestDelivery>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, HarvestDelivery>(
                "SELECT * FROM harvest_deliveries WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> Fut<PaginatedResponse<HarvestDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM harvest_deliveries WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<HarvestDelivery> = sqlx::query_as("SELECT * FROM harvest_deliveries WHERE tenant_id = $1 ORDER BY delivery_date DESC LIMIT $2 OFFSET $3")
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

    fn find_by_lot(
        &self,
        tid: TenantId,
        lot_id: Uuid,
        p: Pagination,
    ) -> Fut<PaginatedResponse<HarvestDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM harvest_deliveries WHERE tenant_id = $1 AND lot_id = $2",
            )
            .bind(tid)
            .bind(lot_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<HarvestDelivery> = sqlx::query_as("SELECT * FROM harvest_deliveries WHERE tenant_id = $1 AND lot_id = $2 ORDER BY delivery_date DESC LIMIT $3 OFFSET $4")
                .bind(tid)
                .bind(lot_id)
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
        dto: CreateHarvestDeliveryDto,
        _by: Uuid,
    ) -> Fut<HarvestDelivery> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let net_weight = dto.gross_weight_kg - dto.tare_weight_kg;
            sqlx::query_as::<_, HarvestDelivery>(
                r#"INSERT INTO harvest_deliveries (id, tenant_id, lot_id, delivery_date, gross_weight_kg, tare_weight_kg, net_weight_kg, carrier_name, vehicle_id, quality_notes, temperature_at_delivery, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
                   RETURNING *"#)
            .bind(id)
            .bind(tid)
            .bind(dto.lot_id)
            .bind(dto.delivery_date)
            .bind(dto.gross_weight_kg)
            .bind(dto.tare_weight_kg)
            .bind(net_weight)
            .bind(&dto.carrier_name)
            .bind(&dto.vehicle_id)
            .bind(&dto.quality_notes)
            .bind(dto.temperature_at_delivery)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateHarvestDeliveryDto,
        _by: Uuid,
    ) -> Fut<Option<HarvestDelivery>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // Need to handle net_weight calculation if gross or tare changes
            let current = sqlx::query_as::<_, HarvestDelivery>(
                "SELECT * FROM harvest_deliveries WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(c) = current {
                let gross = dto.gross_weight_kg.unwrap_or(c.gross_weight_kg);
                let tare = dto.tare_weight_kg.unwrap_or(c.tare_weight_kg);
                let net = gross - tare;

                sqlx::query_as::<_, HarvestDelivery>(
                    r#"UPDATE harvest_deliveries SET 
                        lot_id = COALESCE($1, lot_id),
                        delivery_date = COALESCE($2, delivery_date),
                        gross_weight_kg = $3,
                        tare_weight_kg = $4,
                        net_weight_kg = $5,
                        carrier_name = COALESCE($6, carrier_name),
                        vehicle_id = COALESCE($7, vehicle_id),
                        quality_notes = COALESCE($8, quality_notes),
                        temperature_at_delivery = COALESCE($9, temperature_at_delivery),
                        updated_at = NOW()
                    WHERE id = $10 AND tenant_id = $11 RETURNING *"#,
                )
                .bind(dto.lot_id)
                .bind(dto.delivery_date)
                .bind(gross)
                .bind(tare)
                .bind(net)
                .bind(&dto.carrier_name)
                .bind(&dto.vehicle_id)
                .bind(&dto.quality_notes)
                .bind(dto.temperature_at_delivery)
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                Ok(None)
            }
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> Fut<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM harvest_deliveries WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
