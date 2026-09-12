//! Inventory Management Handlers
//!
//! CRUD für Inventarartikel, Transaktionen, Standorte und Bestandsbewegungen.

use crate::AppState;
use crate::dto::{
    AdjustRequest, CreateInventoryItemRequest, CreateInventoryLocationRequest, InventoryBalanceDto,
    InventoryItemDto, InventoryLocationDto, InventoryTransactionDto,
    PaginatedInventoryItemResponse, PaginatedInventoryLocationResponse,
    PaginatedInventoryTransactionResponse, StockInRequest, StockOutRequest, TransferRequest,
    UpdateInventoryItemRequest,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::inventory::{
    CreateInventoryItemDto, CreateInventoryLocationDto, CreateInventoryTransactionDto,
    UpdateInventoryItemDto,
};
use agrocore_logging::info;
use agrocore_shared::SharedError;

#[utoipa::path(
    get,
    path = "/api/v1/inventory/items",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List inventory items", body = PaginatedInventoryItemResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn list_inventory_items(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    info!(
        "Listing inventory items for tenant: {}",
        agrocore_domain::TenantId(auth.0.tenant_id)
    );
    let result = state
        .db
        .inventory_item_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedInventoryItemResponse {
        data: result
            .data
            .into_iter()
            .map(InventoryItemDto::from)
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/items/{id}",
    responses(
        (status = 200, description = "Inventory item", body = InventoryItemDto),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn get_inventory_item(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let item_id = *path;
    let item = state
        .db
        .inventory_item_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), item_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Inventory item not found".into()))?;
    Ok(HttpResponse::Ok().json(InventoryItemDto::from(item)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/items",
    request_body = CreateInventoryItemRequest,
    responses(
        (status = 201, description = "Item created", body = InventoryItemDto),
        (status = 400, description = "Validation failed", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn create_inventory_item(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateInventoryItemRequest>,
) -> Result<HttpResponse, ApiError> {
    let category =
        crate::dto::parse_category_owned(&dto.category).map_err(SharedError::Validation)?;
    let unit = crate::dto::parse_unit_owned(&dto.unit).map_err(SharedError::Validation)?;
    let method =
        crate::dto::parse_method_owned(&dto.inventory_method).map_err(SharedError::Validation)?;

    let domain_dto = CreateInventoryItemDto {
        category,
        name: dto.name.clone(),
        sku: dto.sku.clone(),
        description: dto.description.clone(),
        unit,
        minimum_stock: dto.minimum_stock,
        inventory_method: method,
    };

    let item = state
        .db
        .inventory_item_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            domain_dto,
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(InventoryItemDto::from(item)))
}

#[utoipa::path(
    put,
    path = "/api/v1/inventory/items/{id}",
    request_body = UpdateInventoryItemRequest,
    responses(
        (status = 200, description = "Item updated", body = InventoryItemDto),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn update_inventory_item(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateInventoryItemRequest>,
) -> Result<HttpResponse, ApiError> {
    let item_id = *path;
    let category =
        crate::dto::parse_category_owned(&dto.category).map_err(SharedError::Validation)?;
    let unit = crate::dto::parse_unit_owned(&dto.unit).map_err(SharedError::Validation)?;
    let method =
        crate::dto::parse_method_owned(&dto.inventory_method).map_err(SharedError::Validation)?;

    let domain_dto = UpdateInventoryItemDto {
        category,
        name: dto.name.clone(),
        sku: dto.sku.clone(),
        description: dto.description.clone(),
        unit,
        minimum_stock: dto.minimum_stock,
        inventory_method: method,
    };

    let item = state
        .db
        .inventory_item_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            item_id,
            domain_dto,
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Inventory item not found".into()))?;
    Ok(HttpResponse::Ok().json(InventoryItemDto::from(item)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/inventory/items/{id}",
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn delete_inventory_item(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let item_id = *path;
    let result = state
        .db
        .inventory_item_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), item_id)
        .await?;
    if result {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::not_found("Inventory item not found"))
    }
}

// Inventory Balances
#[utoipa::path(
    get,
    path = "/api/v1/inventory/balances",
    responses(
        (status = 200, description = "List item balances", body = Vec<InventoryBalanceDto>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn list_inventory_balances(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let balances = state
        .db
        .inventory_item_repo()
        .find_balances(agrocore_domain::TenantId(auth.0.tenant_id))
        .await?;
    let dto: Vec<InventoryBalanceDto> = balances
        .into_iter()
        .map(InventoryBalanceDto::from)
        .collect();
    Ok(HttpResponse::Ok().json(dto))
}

#[utoipa::path(
    get,
    path = "/api/v1/inventory/balances/below-minimum",
    responses(
        (status = 200, description = "Items below minimum stock", body = Vec<InventoryBalanceDto>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn list_below_minimum(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let balances = state
        .db
        .inventory_item_repo()
        .find_below_minimum(agrocore_domain::TenantId(auth.0.tenant_id))
        .await?;
    let dto: Vec<InventoryBalanceDto> = balances
        .into_iter()
        .map(InventoryBalanceDto::from)
        .collect();
    Ok(HttpResponse::Ok().json(dto))
}

// Inventory Transactions
#[utoipa::path(
    get,
    path = "/api/v1/inventory/items/{id}/transactions",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List transactions", body = PaginatedInventoryTransactionResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn list_item_transactions(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let item_id = *path;
    let result = state
        .db
        .inventory_transaction_repo()
        .find_by_item(
            agrocore_domain::TenantId(auth.0.tenant_id),
            item_id,
            query.0,
        )
        .await?;
    Ok(
        HttpResponse::Ok().json(PaginatedInventoryTransactionResponse {
            data: result
                .data
                .into_iter()
                .map(InventoryTransactionDto::from)
                .collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/transactions",
    request_body = CreateInventoryTransactionDto,
    responses(
        (status = 201, description = "Transaction created", body = InventoryTransactionDto),
        (status = 400, description = "Validation failed", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn create_transaction(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateInventoryTransactionDto>,
) -> Result<HttpResponse, ApiError> {
    let tx = state
        .db
        .inventory_transaction_repo()
        .create_transaction(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0,
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(InventoryTransactionDto::from(tx)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-in",
    request_body = StockInRequest,
    responses(
        (status = 201, description = "Stock in recorded", body = InventoryTransactionDto),
        (status = 400, description = "Validation failed", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn stock_in(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<StockInRequest>,
) -> Result<HttpResponse, ApiError> {
    let tx = state
        .db
        .inventory_transaction_repo()
        .stock_in(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.item_id,
            dto.quantity,
            dto.unit_cost,
            dto.batch_number.clone(),
            dto.expiration_date.clone(),
            dto.location.clone(),
            dto.notes.clone(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(InventoryTransactionDto::from(tx)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/stock-out",
    request_body = StockOutRequest,
    responses(
        (status = 200, description = "Stock out recorded", body = InventoryTransactionDto),
        (status = 400, description = "Insufficient stock", body = crate::dto::ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn stock_out(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<StockOutRequest>,
) -> Result<HttpResponse, ApiError> {
    let tx = state
        .db
        .inventory_transaction_repo()
        .stock_out(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.item_id,
            dto.quantity,
            dto.location.clone(),
            dto.notes.clone(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Could not record stock out".into()))?;
    Ok(HttpResponse::Ok().json(InventoryTransactionDto::from(tx)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/transfer",
    request_body = TransferRequest,
    responses(
        (status = 200, description = "Transfer recorded", body = InventoryTransactionDto),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn transfer_inventory(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<TransferRequest>,
) -> Result<HttpResponse, ApiError> {
    let tx = state
        .db
        .inventory_transaction_repo()
        .transfer(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.item_id,
            dto.quantity,
            &dto.from_location,
            &dto.to_location,
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Could not record transfer".into()))?;
    Ok(HttpResponse::Ok().json(InventoryTransactionDto::from(tx)))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/adjust",
    request_body = AdjustRequest,
    responses(
        (status = 200, description = "Adjustment recorded", body = InventoryTransactionDto),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn adjust_inventory(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<AdjustRequest>,
) -> Result<HttpResponse, ApiError> {
    let tx = state
        .db
        .inventory_transaction_repo()
        .adjust(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.item_id,
            dto.quantity,
            &dto.notes,
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Could not record adjustment".into()))?;
    Ok(HttpResponse::Ok().json(InventoryTransactionDto::from(tx)))
}

// Inventory Locations
#[utoipa::path(
    get,
    path = "/api/v1/inventory/locations",
    responses(
        (status = 200, description = "List inventory locations", body = PaginatedInventoryLocationResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn list_inventory_locations(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .inventory_location_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedInventoryLocationResponse {
        data: result
            .data
            .into_iter()
            .map(InventoryLocationDto::from)
            .collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/inventory/locations",
    request_body = CreateInventoryLocationRequest,
    responses(
        (status = 201, description = "Location created", body = InventoryLocationDto),
        (status = 401, description = "Unauthorized")
    ),
    tag = "inventory",
    security(("bearer_auth" = []))
)]
pub async fn create_inventory_location(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateInventoryLocationRequest>,
) -> Result<HttpResponse, ApiError> {
    let domain_dto = CreateInventoryLocationDto {
        name: dto.name.clone(),
        code: dto.code.clone(),
        description: dto.description.clone(),
    };
    let location = state
        .db
        .inventory_location_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            domain_dto,
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(InventoryLocationDto::from(location)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_category_seed() {
        let result = crate::dto::parse_category_owned("\"seed\"");
        assert!(result.is_ok());
        assert_eq!(serde_json::to_string(&result.unwrap()).unwrap(), "\"seed\"");
    }

    #[test]
    fn test_parse_category_invalid() {
        let result = crate::dto::parse_category_owned("\"invalid_category\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unit() {
        let result = crate::dto::parse_unit_owned("\"kg\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_unit_custom() {
        let result = crate::dto::parse_unit_owned("\"custom\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_method() {
        let result = crate::dto::parse_method_owned("\"FIFO\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_invalid() {
        let result = crate::dto::parse_method_owned("\"LIFO\"");
        assert!(result.is_err());
    }
}
