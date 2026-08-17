use crate::AppState;
use crate::dto::customer::{CreateCustomerDto, CustomerDto, UpdateCustomerDto};
use crate::dto::{ErrorResponse, PaginatedResponseDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::customer::Customer;
use agrocore_shared::SharedError;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/customers")
            .route(web::get().to(list_customers))
            .route(web::post().to(create_customer)),
    )
    .service(
        web::resource("/customers/{id}")
            .route(web::get().to(get_customer))
            .route(web::put().to(update_customer))
            .route(web::delete().to(delete_customer)),
    )
    .service(web::resource("/customers/search/{query}").route(web::get().to(search_customers)))
    .service(
        web::resource("/customers/number/{number}").route(web::get().to(get_customer_by_number)),
    );
}

#[utoipa::path(
    get,
    path = "/api/v1/customers",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List customers", body = PaginatedResponseDto<CustomerDto>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn list_customers(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .customer_repo()
        .find_all_visible(
            agrocore_domain::TenantId(auth.0.tenant_id),
            query.0,
            auth.0.user_id,
            &auth.roles(),
        )
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(CustomerDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/customers/{id}",
    responses(
        (status = 200, description = "Customer details", body = CustomerDto),
        (status = 404, description = "Customer not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn get_customer(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    let customer_id = *path;
    let customer = state
        .db
        .customer_repo()
        .find_by_id_visible(
            agrocore_domain::TenantId(auth.0.tenant_id),
            customer_id,
            auth.0.user_id,
            &auth.roles(),
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Customer not found".into()))?;
    Ok(HttpResponse::Ok().json(CustomerDto::from(customer)))
}

#[utoipa::path(
    post,
    path = "/api/v1/customers",
    request_body = CreateCustomerDto,
    responses(
        (status = 201, description = "Customer created", body = CustomerDto),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn create_customer(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateCustomerDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    dto.0
        .validate()
        .map_err(|e| SharedError::Validation(e.to_string()))?;
    let customer = state
        .db
        .customer_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(CustomerDto::from(customer)))
}

#[utoipa::path(
    put,
    path = "/api/v1/customers/{id}",
    request_body = UpdateCustomerDto,
    responses(
        (status = 200, description = "Customer updated", body = CustomerDto),
        (status = 404, description = "Customer not found", body = ErrorResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn update_customer(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
    dto: web::Json<UpdateCustomerDto>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let customer_id = *path;
    let customer = state
        .db
        .customer_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            customer_id,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Customer not found".into()))?;
    Ok(HttpResponse::Ok().json(CustomerDto::from(customer)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/customers/{id}",
    responses(
        (status = 200, description = "Customer deleted"),
        (status = 404, description = "Customer not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn delete_customer(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ApiError> {
    auth.require_manager()?;
    let customer_id = *path;
    let deleted = state
        .db
        .customer_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), customer_id)
        .await?;
    if deleted {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Customer not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/customers/search/{query}",
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Search customers", body = PaginatedResponseDto<CustomerDto>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn search_customers(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<String>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let search_term = path.into_inner();
    let result = state
        .db
        .customer_repo()
        .search(
            agrocore_domain::TenantId(auth.0.tenant_id),
            &search_term,
            query.0,
        )
        .await?;
    Ok(HttpResponse::Ok().json(PaginatedResponseDto {
        data: result.data.into_iter().map(CustomerDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/customers/number/{number}",
    responses(
        (status = 200, description = "Customer by number", body = CustomerDto),
        (status = 404, description = "Customer not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "customers",
    security(("bearer_auth" = []))
)]
pub async fn get_customer_by_number(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let number = path.into_inner();
    let customer = state
        .db
        .customer_repo()
        .find_by_customer_number(agrocore_domain::TenantId(auth.0.tenant_id), &number)
        .await?
        .ok_or_else(|| SharedError::NotFound("Customer not found".into()))?;
    Ok(HttpResponse::Ok().json(CustomerDto::from(customer)))
}

// Convert domain Customer to DTO
impl From<Customer> for CustomerDto {
    fn from(c: Customer) -> Self {
        Self {
            id: c.id,
            tenant_id: c.tenant_id.0,
            name: c.name,
            email: c.email,
            phone: c.phone,
            address: c.address,
            company: c.company,
            customer_number: c.customer_number,
            vat_rate: c.vat_rate,
            payment_terms: c.payment_terms,
            preferred_delivery_location: c.preferred_delivery_location,
            preferences: c.preferences,
            is_active: c.is_active,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

// Convert DTOs to domain types
impl From<CreateCustomerDto> for agrocore_domain::entities::customer::CreateCustomerDto {
    fn from(dto: CreateCustomerDto) -> Self {
        Self {
            name: dto.name,
            email: dto.email,
            phone: dto.phone,
            address: dto.address,
            company: dto.company,
            customer_number: dto.customer_number,
            vat_rate: dto.vat_rate,
            payment_terms: dto.payment_terms,
            preferred_delivery_location: dto.preferred_delivery_location,
            preferences: dto.preferences,
        }
    }
}

impl From<UpdateCustomerDto> for agrocore_domain::entities::customer::UpdateCustomerDto {
    fn from(dto: UpdateCustomerDto) -> Self {
        Self {
            name: dto.name,
            email: dto.email,
            phone: dto.phone,
            address: dto.address,
            company: dto.company,
            customer_number: dto.customer_number,
            vat_rate: dto.vat_rate,
            payment_terms: dto.payment_terms,
            preferred_delivery_location: dto.preferred_delivery_location,
            preferences: dto.preferences,
            is_active: dto.is_active,
        }
    }
}
