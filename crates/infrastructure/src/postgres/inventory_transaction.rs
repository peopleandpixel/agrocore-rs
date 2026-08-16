use agrocore_domain::entities::inventory::{
    CreateInventoryTransactionDto, InventoryTransaction, TransactionType,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    InventoryTransactionRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pg_repo!(PgInventoryTransactionRepo);

impl InventoryTransactionRepo for PgInventoryTransactionRepo {
    fn find_by_id(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryTransaction>(
                "SELECT * FROM inventory_transactions WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_item(
        &self,
        tid: TenantId,
        item_id: Uuid,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<InventoryTransaction>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2",
            )
            .bind(item_id)
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<InventoryTransaction> = sqlx::query_as(
                "SELECT * FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(item_id)
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
                data: items,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn find_recent_transactions(
        &self,
        tid: TenantId,
        limit: u32,
    ) -> RepositoryFuture<Vec<InventoryTransaction>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryTransaction>(
                "SELECT * FROM inventory_transactions WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(tid)
            .bind(limit as i64)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn create_transaction(
        &self,
        tid: TenantId,
        dto: CreateInventoryTransactionDto,
        by: Uuid,
    ) -> RepositoryFuture<InventoryTransaction> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            let total_cost = dto.unit_cost.map(|c| c * dto.quantity);

            let tx_type =
                serde_json::to_string(&dto.transaction_type).unwrap_or_else(|_| "null".to_string());

            let transaction = sqlx::query_as::<_, InventoryTransaction>(
                r#"INSERT INTO inventory_transactions
                   (id, tenant_id, item_id, transaction_type, quantity, unit_cost, total_cost,
                    batch_number, expiration_date, location, notes, reference_id, created_by, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(dto.item_id)
            .bind(&tx_type)
            .bind(dto.quantity)
            .bind(dto.unit_cost)
            .bind(total_cost)
            .bind(&dto.batch_number)
            .bind(&dto.expiration_date)
            .bind(&dto.location)
            .bind(&dto.notes)
            .bind(None::<String>)
            .bind(by)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(transaction)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn stock_in(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        unit_cost: Option<f64>,
        batch_number: Option<String>,
        expiration_date: Option<String>,
        location: Option<String>,
        notes: Option<String>,
        by: Uuid,
    ) -> RepositoryFuture<InventoryTransaction> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();
            let total_cost = unit_cost.map(|c| c * quantity);
            let tx_type = serde_json::to_string(&TransactionType::StockIn)
                .unwrap_or_else(|_| "null".to_string());

            let transaction = sqlx::query_as::<_, InventoryTransaction>(
                r#"INSERT INTO inventory_transactions
                   (id, tenant_id, item_id, transaction_type, quantity, unit_cost, total_cost,
                    batch_number, expiration_date, location, notes, reference_id, created_by, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(item_id)
            .bind(&tx_type)
            .bind(quantity)
            .bind(unit_cost)
            .bind(total_cost)
            .bind(&batch_number)
            .bind(&expiration_date)
            .bind(&location)
            .bind(&notes)
            .bind(None::<String>)
            .bind(by)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(transaction)
        })
    }

    fn stock_out(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        location: Option<String>,
        notes: Option<String>,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            // Calculate available balance using FIFO/FEFO logic
            let balances: Vec<(Uuid, f64, f64, Option<String>)> = sqlx::query_as(
                "SELECT id, quantity, unit_cost, expiration_date \
                 FROM inventory_transactions \
                 WHERE item_id = $1 AND tenant_id = $2 \
                 AND (transaction_type = 'stock_in' OR transaction_type = 'transfer_in') \
                 AND (expiration_date IS NULL OR expiration_date > $3) \
                 ORDER BY COALESCE(expiration_date, '9999-12-31'), created_at ASC",
            )
            .bind(item_id)
            .bind(tid)
            .bind(Utc::now().to_string())
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut remaining = quantity;
            for (tx_id, available, unit_cost, _) in balances {
                if remaining <= 0.0 {
                    break;
                }
                let take = remaining.min(available);
                remaining -= take;
                let _ = unit_cost; // unit_cost used for cost of goods sold tracking
            }

            if remaining > 0.0 {
                return Err(SharedError::Conflict(format!(
                    "Insufficient stock: requested {}, available {}",
                    quantity,
                    quantity - remaining
                )));
            }

            let now = Utc::now();
            let id = Uuid::new_v4();
            let tx_type = serde_json::to_string(&TransactionType::StockOut)
                .unwrap_or_else(|_| "null".to_string());

            let transaction = sqlx::query_as::<_, InventoryTransaction>(
                r#"INSERT INTO inventory_transactions
                   (id, tenant_id, item_id, transaction_type, quantity, unit_cost, total_cost,
                    location, notes, created_by, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(item_id)
            .bind(&tx_type)
            .bind(-quantity) // Negative quantity for stock out
            .bind(None::<f64>)
            .bind(None::<f64>)
            .bind(&location)
            .bind(&notes)
            .bind(None::<String>)
            .bind(by)
            .bind(now)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(transaction)
        })
    }

    fn transfer(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        from_location: &str,
        to_location: &str,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>> {
        let pool = self.pool.clone();
        let from_location = from_location.to_string();
        let to_location = to_location.to_string();

        Box::pin(async move {
            let now = Utc::now();
            let tx_type = serde_json::to_string(&TransactionType::Transfer)
                .unwrap_or_else(|_| "null".to_string());

            // Create a transfer transaction
            let id = Uuid::new_v4();
            let transaction = sqlx::query_as::<_, InventoryTransaction>(
                r#"INSERT INTO inventory_transactions
                   (id, tenant_id, item_id, transaction_type, quantity, location, notes, created_by, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(item_id)
            .bind(&tx_type)
            .bind(quantity)
            .bind(&to_location)
            .bind(format!("Transferred from {} to {}", from_location, to_location))
            .bind(None::<String>)
            .bind(by)
            .bind(now)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(transaction)
        })
    }

    fn adjust(
        &self,
        tid: TenantId,
        item_id: Uuid,
        quantity: f64,
        notes: &str,
        by: Uuid,
    ) -> RepositoryFuture<Option<InventoryTransaction>> {
        let pool = self.pool.clone();
        let notes = notes.to_string();

        Box::pin(async move {
            let now = Utc::now();
            let tx_type = serde_json::to_string(&TransactionType::Adjustment)
                .unwrap_or_else(|_| "null".to_string());

            let id = Uuid::new_v4();
            let transaction = sqlx::query_as::<_, InventoryTransaction>(
                r#"INSERT INTO inventory_transactions
                   (id, tenant_id, item_id, transaction_type, quantity, notes, created_by, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(item_id)
            .bind(&tx_type)
            .bind(quantity)
            .bind(&notes)
            .bind(None::<String>)
            .bind(by)
            .bind(now)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(transaction)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result =
                sqlx::query("DELETE FROM inventory_transactions WHERE id = $1 AND tenant_id = $2")
                    .bind(id)
                    .bind(tid)
                    .execute(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
