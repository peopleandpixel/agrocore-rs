use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use agrocore_domain::entities::finance::{PACApplication, CostCenter, FinancialRecord, CreatePACApplicationDto, CreateCostCenterDto, CreateFinancialRecordDto};
use agrocore_shared::Pagination;
use crate::AppState;
use crate::middleware::AuthExtractor;
use crate::dto::{ErrorResponse, PaginatedPACApplicationResponse, PaginatedCostCenterResponse, PaginatedFinancialRecordResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance")
            .service(
                web::resource("/pac-applications")
                    .route(web::get().to(list_pac_applications))
                    .route(web::post().to(create_pac_application))
            )
            .service(
                web::resource("/pac-applications/{id}")
                    .route(web::get().to(get_pac_application))
            )
            .service(
                web::resource("/cost-centers")
                    .route(web::get().to(list_cost_centers))
                    .route(web::post().to(create_cost_center))
            )
            .service(
                web::resource("/cost-centers/{id}")
                    .route(web::get().to(get_cost_center))
            )
            .service(
                web::resource("/financial-records")
                    .route(web::get().to(list_financial_records))
                    .route(web::post().to(create_financial_record))
            )
            .service(
                web::resource("/financial-records/{id}")
                    .route(web::get().to(get_financial_record))
            )
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
) -> impl Responder {
    match state.db.pac_application_repo().find_all(auth.0.tenant_id, query.into_inner()).await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.pac_application_repo().create(auth.0.tenant_id, dto.into_inner()).await {
        Ok(app) => HttpResponse::Created().json(app),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.pac_application_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await {
        Ok(Some(app)) => HttpResponse::Ok().json(app),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "Not found".into(), message: "Not found".to_string() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.cost_center_repo().find_all(auth.0.tenant_id, query.into_inner()).await {
        Ok(ccs) => HttpResponse::Ok().json(ccs),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.cost_center_repo().create(auth.0.tenant_id, dto.into_inner()).await {
        Ok(cc) => HttpResponse::Created().json(cc),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.cost_center_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await {
        Ok(Some(cc)) => HttpResponse::Ok().json(cc),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "Not found".into(), message: "Not found".to_string() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.financial_record_repo().find_all(auth.0.tenant_id, query.into_inner()).await {
        Ok(recs) => HttpResponse::Ok().json(recs),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.financial_record_repo().create(auth.0.tenant_id, dto.into_inner()).await {
        Ok(rec) => HttpResponse::Created().json(rec),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
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
) -> impl Responder {
    match state.db.financial_record_repo().find_by_id(auth.0.tenant_id, id.into_inner()).await {
        Ok(Some(rec)) => HttpResponse::Ok().json(rec),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse { error: "Not found".into(), message: "Not found".to_string() }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: "Database error".into(), message: e.to_string() }),
    }
}
