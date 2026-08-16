use agrocore_domain::entities::inventory::{
    CreateInventoryItemDto, InventoryBalance, InventoryItem, UpdateInventoryItemDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::repositories::{
    InventoryItemRepository, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use agrocore_shared::pg_repo;
use chrono::Utc;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

pg_repo!(PgInventoryItemRepo);

impl InventoryItemRepository for PgInventoryItemRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<InventoryItem>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryItem>(
                "SELECT * FROM inventory_items WHERE id = $1 AND tenant_id = $2 AND is_active IS DISTINCT FROM false",
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_sku(&self, tid: TenantId, sku: &str) -> RepositoryFuture<Option<InventoryItem>> {
        let pool = self.pool.clone();
        let sku = sku.to_string();
        Box::pin(async move {
            sqlx::query_as::<_, InventoryItem>(
                "SELECT * FROM inventory_items WHERE sku = $1 AND tenant_id = $2 AND is_active IS DISTINCT FROM false",
            )
            .bind(&sku)
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
    ) -> RepositoryFuture<PaginatedResponse<InventoryItem>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);

        Box::pin(async move {
            let offset = page * per_page;

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM inventory_items WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let items: Vec<InventoryItem> = sqlx::query_as(
                "SELECT * FROM inventory_items WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true) ORDER BY name LIMIT $2 OFFSET $3",
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
                data: items,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }

    fn find_below_minimum(&self, tid: TenantId) -> RepositoryFuture<Vec<InventoryBalance>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let items: Vec<InventoryItem> = sqlx::query_as(
                "SELECT * FROM inventory_items WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut balances = Vec::new();
            for item in items {
                let balance = sqlx::query_as::<_, (f64, f64)>(
                    "SELECT COALESCE(SUM(quantity), 0) as total, COALESCE(SUM(quantity * unit_cost), 0) as total_value \
                     FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2 AND (transaction_type != 'stock_out' OR transaction_type IS NULL)",
                )
                .bind(item.id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                let (total_qty, total_val) = balance.unwrap_or((0.0, 0.0));

                if total_qty < item.minimum_stock {
                    balances.push(InventoryBalance {
                        item_id: item.id,
                        item_name: item.name.clone(),
                        category: item.category,
                        unit: item.unit,
                        total_quantity: total_qty,
                        available_quantity: total_qty,
                        reserved_quantity: 0.0,
                        average_unit_cost: if total_qty > 0.0 {
                            Some(total_val / total_qty)
                        } else {
                            None
                        },
                        total_value: Some(total_val),
                        minimum_stock: item.minimum_stock,
                        is_below_minimum: total_qty < item.minimum_stock,
                        inventory_method: item.inventory_method,
                        latest_transactions: vec![],
                    });
                }
            }

            Ok(balances)
        })
    }

    fn find_balances(&self, tid: TenantId) -> RepositoryFuture<Vec<InventoryBalance>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let items: Vec<InventoryItem> = sqlx::query_as(
                "SELECT * FROM inventory_items WHERE tenant_id = $1 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut balances = Vec::new();
            for item in items {
                let total: (f64, f64) = sqlx::query_as(
                    "SELECT COALESCE(SUM(quantity), 0) as total, COALESCE(SUM(quantity * unit_cost), 0) as total_value \
                     FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2",
                )
                .bind(item.id)
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                let (total_qty, total_val) = (total.0, total.1);
                let avg_cost = if total_qty > 0.0 {
                    Some(total_val / total_qty)
                } else {
                    None
                };

                balances.push(InventoryBalance {
                    item_id: item.id,
                    item_name: item.name.clone(),
                    category: item.category,
                    unit: item.unit,
                    total_quantity: total_qty,
                    available_quantity: total_qty,
                    reserved_quantity: 0.0,
                    average_unit_cost: avg_cost,
                    total_value: Some(total_val),
                    minimum_stock: item.minimum_stock,
                    is_below_minimum: total_qty < item.minimum_stock,
                    inventory_method: item.inventory_method,
                    latest_transactions: vec![],
                });
            }

            Ok(balances)
        })
    }

    fn find_balance_by_item(
        &self,
        tid: TenantId,
        item_id: Uuid,
    ) -> RepositoryFuture<Option<InventoryBalance>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let item: Option<InventoryItem> = sqlx::query_as(
                "SELECT * FROM inventory_items WHERE id = $1 AND tenant_id = $2 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(item_id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let Some(item) = item else {
                return Ok(None);
            };

            let recent: Vec<agrocore_domain::entities::inventory::InventoryTransaction> = sqlx::query_as(
                "SELECT * FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2 ORDER BY created_at DESC LIMIT 10",
            )
            .bind(item.id)
            .bind(tid)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let total: (f64, f64) = sqlx::query_as(
                "SELECT COALESCE(SUM(quantity), 0) as total, COALESCE(SUM(quantity * unit_cost), 0) as total_value \
                 FROM inventory_transactions WHERE item_id = $1 AND tenant_id = $2",
            )
            .bind(item.id)
            .bind(tid)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let (total_qty, total_val) = (total.0, total.1);
            let avg_cost = if total_qty > 0.0 {
                Some(total_val / total_qty)
            } else {
                None
            };

            Ok(Some(InventoryBalance {
                item_id: item.id,
                item_name: item.name.clone(),
                category: item.category,
                unit: item.unit,
                total_quantity: total_qty,
                available_quantity: total_qty,
                reserved_quantity: 0.0,
                average_unit_cost: avg_cost,
                total_value: Some(total_val),
                minimum_stock: item.minimum_stock,
                is_below_minimum: total_qty < item.minimum_stock,
                inventory_method: item.inventory_method,
                latest_transactions: recent,
            }))
        })
    }

    fn create(
        &self,
        tid: TenantId,
        dto: CreateInventoryItemDto,
        _by: Uuid,
    ) -> RepositoryFuture<InventoryItem> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let id = Uuid::new_v4();

            let item = sqlx::query_as::<_, InventoryItem>(
                r#"INSERT INTO inventory_items (id, tenant_id, category, name, sku, description, unit, minimum_stock, inventory_method, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                   RETURNING *"#,
            )
            .bind(id)
            .bind(tid)
            .bind(serde_json::to_string(&dto.category).unwrap_or_default())
            .bind(&dto.name)
            .bind(&dto.sku)
            .bind(&dto.description)
            .bind(serde_json::to_string(&dto.unit).unwrap_or_default())
            .bind(dto.minimum_stock)
            .bind(serde_json::to_string(&dto.inventory_method).unwrap_or_default())
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(item)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateInventoryItemDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<InventoryItem>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();

            let result = sqlx::query_as::<_, InventoryItem>(
                r#"UPDATE inventory_items SET
                    category = $1,
                    name = $2,
                    sku = $3,
                    description = $4,
                    unit = $5,
                    minimum_stock = $6,
                    inventory_method = $7,
                    updated_at = $8
                   WHERE id = $9 AND tenant_id = $10 AND (is_active IS NULL OR is_active = true)
                   RETURNING *"#,
            )
            .bind(serde_json::to_string(&dto.category).unwrap_or_default())
            .bind(&dto.name)
            .bind(&dto.sku)
            .bind(&dto.description)
            .bind(serde_json::to_string(&dto.unit).unwrap_or_default())
            .bind(dto.minimum_stock)
            .bind(serde_json::to_string(&dto.inventory_method).unwrap_or_default())
            .bind(now)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result)
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE inventory_items SET is_active = false, updated_at = $1 WHERE id = $2 AND tenant_id = $3 AND (is_active IS NULL OR is_active = true)",
            )
            .bind(Utc::now())
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}
