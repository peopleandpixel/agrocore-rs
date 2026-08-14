use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::vineyard::{
    CreateKelterDeliveryDto, KelterDelivery, UpdateKelterDeliveryDto,
};
use agrocore_domain::repositories::{
    KelterDeliveryRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgKelterDeliveryRepo);

impl KelterDeliveryRepo for PgKelterDeliveryRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<KelterDelivery>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, KelterDelivery>("SELECT * FROM kelter_deliveries WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<KelterDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kelter_deliveries")
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<KelterDelivery> =
                sqlx::query_as("SELECT * FROM kelter_deliveries LIMIT $1 OFFSET $2")
                    .bind(per_page as i32)
                    .bind(offset as i32)
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn find_by_vineyard(
        &self,
        tid: TenantId,
        vineyard_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<KelterDelivery>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM kelter_deliveries WHERE vineyard_id = $1")
                    .bind(vineyard_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<KelterDelivery> = sqlx::query_as(
                "SELECT * FROM kelter_deliveries WHERE vineyard_id = $1 LIMIT $2 OFFSET $3",
            )
            .bind(vineyard_id)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateKelterDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<KelterDelivery> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, KelterDelivery>(
                r#"INSERT INTO kelter_deliveries (id, vineyard_id, delivery_date, gross_weight_kg, net_weight_kg, lot_number, kelter_name, transport_company, temperature_c, notes)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   RETURNING *"#)
            .bind(id)
            .bind(dto.vineyard_id)
            .bind(dto.delivery_date)
            .bind(dto.gross_weight_kg)
            .bind(dto.net_weight_kg)
            .bind(&dto.lot_number)
            .bind(&dto.kelter_name)
            .bind(&dto.transport_company)
            .bind(dto.temperature_c)
            .bind(&dto.notes)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateKelterDeliveryDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<KelterDelivery>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, KelterDelivery>(
                r#"UPDATE kelter_deliveries SET 
                    vineyard_id = COALESCE($1, vineyard_id),
                    delivery_date = COALESCE($2, delivery_date),
                    gross_weight_kg = COALESCE($3, gross_weight_kg),
                    net_weight_kg = COALESCE($4, net_weight_kg),
                    lot_number = COALESCE($5, lot_number),
                    kelter_name = COALESCE($6, kelter_name),
                    transport_company = COALESCE($7, transport_company),
                    temperature_c = COALESCE($8, temperature_c),
                    notes = COALESCE($9, notes)
                   WHERE id = $10 RETURNING *"#,
            )
            .bind(dto.vineyard_id)
            .bind(dto.delivery_date)
            .bind(dto.gross_weight_kg)
            .bind(dto.net_weight_kg)
            .bind(&dto.lot_number)
            .bind(&dto.kelter_name)
            .bind(&dto.transport_company)
            .bind(dto.temperature_c)
            .bind(&dto.notes)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("DELETE FROM kelter_deliveries WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
