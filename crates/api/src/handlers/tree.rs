use crate::AppState;
use crate::dto::{CreateTreeDto, PaginatedTreeResponse, TreeDto, UpdateTreeDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/trees",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List trees", body = PaginatedTreeResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_trees(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .tree_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedTreeResponse {
        data: result.data.into_iter().map(TreeDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/trees",
    request_body = CreateTreeDto,
    responses(
        (status = 201, description = "Tree created", body = TreeDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_tree(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateTreeDto>,
) -> Result<HttpResponse, ApiError> {
    let tree = state
        .db
        .tree_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(TreeDto::from(tree)))
}

#[utoipa::path(
    get,
    path = "/trees/{id}",
    responses(
        (status = 200, description = "Tree details", body = TreeDto),
        (status = 404, description = "Tree not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_tree(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let tree = state
        .db
        .tree_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Tree not found".into()))?;
    Ok(HttpResponse::Ok().json(TreeDto::from(tree)))
}

#[utoipa::path(
    put,
    path = "/trees/{id}",
    request_body = UpdateTreeDto,
    responses(
        (status = 200, description = "Tree updated", body = TreeDto),
        (status = 404, description = "Tree not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_tree(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateTreeDto>,
) -> Result<HttpResponse, ApiError> {
    let tree = state
        .db
        .tree_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Tree not found".into()))?;
    Ok(HttpResponse::Ok().json(TreeDto::from(tree)))
}

#[utoipa::path(
    delete,
    path = "/trees/{id}",
    responses(
        (status = 200, description = "Tree deleted"),
        (status = 404, description = "Tree not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_tree(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .tree_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Tree not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/trees/by-plot/{plot_id}",
    params(
        ("plot_id" = Uuid, Path, description = "Plot UUID"),
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List trees by plot", body = PaginatedTreeResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_trees_by_plot(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .tree_repo()
        .find_by_plot(agrocore_domain::TenantId(auth.0.tenant_id), *path, query.0)
        .await?;

    let response = PaginatedTreeResponse {
        data: result.data.into_iter().map(TreeDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/trees/by-group/{group_id}",
    responses(
        (status = 200, description = "List trees by group", body = Vec<TreeDto>),
        (status = 404, description = "Group not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_trees_by_group(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let trees = state
        .db
        .tree_repo()
        .find_by_group(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?;
    Ok(HttpResponse::Ok().json(trees.into_iter().map(TreeDto::from).collect::<Vec<_>>()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trees")
            .route("", web::get().to(list_trees))
            .route("", web::post().to(create_tree))
            .route("/{id}", web::get().to(get_tree))
            .route("/{id}", web::put().to(update_tree))
            .route("/{id}", web::delete().to(delete_tree))
            .route("/by-plot/{plot_id}", web::get().to(list_trees_by_plot))
            .route("/by-group/{group_id}", web::get().to(list_trees_by_group)),
    );
}
