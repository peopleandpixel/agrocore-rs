use crate::AppState;
use crate::dto::{
    ErrorResponse, PaginatedCostCenterResponse, PaginatedFinancialRecordResponse,
    PaginatedPACApplicationResponse,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::finance::{
    CostCenter, CreateCostCenterDto, CreateFinancialRecordDto, CreatePACApplicationDto,
    FinancialRecord, PACApplication,
};
use agrocore_shared::{Pagination, SharedError};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance")
            .service(
                web::resource("/pac-applications")
                    .route(web::get().to(list_pac_applications))
                    .route(web::post().to(create_pac_application)),
            )
            .service(
                web::resource("/pac-applications/{id}").route(web::get().to(get_pac_application)),
            )
            .service(
                web::resource("/cost-centers")
                    .route(web::get().to(list_cost_centers))
                    .route(web::post().to(create_cost_center)),
            )
            .service(web::resource("/cost-centers/{id}").route(web::get().to(get_cost_center)))
            .service(
                web::resource("/financial-records")
                    .route(web::get().to(list_financial_records))
                    .route(web::post().to(create_financial_record)),
            )
            .service(
                web::resource("/financial-records/{id}").route(web::get().to(get_financial_record)),
            ),
    );
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/pac-applications",
    responses(
        (status = 200, description = "List PAC applications", body = PaginatedPACApplicationResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_pac_applications(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let apps = state
        .db
        .pac_application_repo()
        .find_all(auth.0.tenant_id, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(apps))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/pac-applications",
    request_body = CreatePACApplicationDto,
    responses(
        (status = 201, description = "PAC application created", body = PACApplication),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_pac_application(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    dto: web::Json<CreatePACApplicationDto>,
) -> Result<HttpResponse, ApiError> {
    let app = state
        .db
        .pac_application_repo()
        .create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(app))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/pac-applications/{id}",
    responses(
        (status = 200, description = "Get PAC application by ID", body = PACApplication),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "PAC application ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_pac_application(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let roles = auth.roles();
    let app = state
        .db
        .pac_application_repo()
        .find_by_id_visible(auth.0.tenant_id, id.into_inner(), auth.0.user_id, &roles)
        .await?
        .ok_or_else(|| SharedError::NotFound("Not found".into()))?;
    Ok(HttpResponse::Ok().json(app))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/cost-centers",
    responses(
        (status = 200, description = "List cost centers", body = PaginatedCostCenterResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_cost_centers(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let ccs = state
        .db
        .cost_center_repo()
        .find_all(auth.0.tenant_id, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(ccs))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/cost-centers",
    request_body = CreateCostCenterDto,
    responses(
        (status = 201, description = "Cost center created", body = CostCenter),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_cost_center(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    dto: web::Json<CreateCostCenterDto>,
) -> Result<HttpResponse, ApiError> {
    let cc = state
        .db
        .cost_center_repo()
        .create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(cc))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/cost-centers/{id}",
    responses(
        (status = 200, description = "Get cost center by ID", body = CostCenter),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Cost center ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_cost_center(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let roles = auth.roles();
    let cc = state
        .db
        .cost_center_repo()
        .find_by_id_visible(auth.0.tenant_id, id.into_inner(), auth.0.user_id, &roles)
        .await?
        .ok_or_else(|| SharedError::NotFound("Not found".into()))?;
    Ok(HttpResponse::Ok().json(cc))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/financial-records",
    responses(
        (status = 200, description = "List financial records", body = PaginatedFinancialRecordResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_financial_records(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, ApiError> {
    let recs = state
        .db
        .financial_record_repo()
        .find_all(auth.0.tenant_id, query.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(recs))
}

#[utoipa::path(
    post,
    path = "/api/v1/finance/financial-records",
    request_body = CreateFinancialRecordDto,
    responses(
        (status = 201, description = "Financial record created", body = FinancialRecord),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_financial_record(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    dto: web::Json<CreateFinancialRecordDto>,
) -> Result<HttpResponse, ApiError> {
    let rec = state
        .db
        .financial_record_repo()
        .create(auth.0.tenant_id, dto.into_inner(), auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(rec))
}

#[utoipa::path(
    get,
    path = "/api/v1/finance/financial-records/{id}",
    responses(
        (status = 200, description = "Get financial record by ID", body = FinancialRecord),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Financial record ID")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_financial_record(
    state: web::Data<AppState>,
    auth: AuthExtractor,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let roles = auth.roles();
    let rec = state
        .db
        .financial_record_repo()
        .find_by_id_visible(auth.0.tenant_id, id.into_inner(), auth.0.user_id, &roles)
        .await?
        .ok_or_else(|| SharedError::NotFound("Not found".into()))?;
    Ok(HttpResponse::Ok().json(rec))
}
