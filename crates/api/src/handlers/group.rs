use crate::AppState;
use crate::dto::{CreateGroupDto, GroupDto, PaginatedGroupResponse, UpdateGroupDto};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/groups",
    params(
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List groups", body = PaginatedGroupResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_groups(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .group_repo()
        .find_all(agrocore_domain::TenantId(auth.0.tenant_id), query.0)
        .await?;

    let response = PaginatedGroupResponse {
        data: result.data.into_iter().map(GroupDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/groups",
    request_body = CreateGroupDto,
    responses(
        (status = 201, description = "Group created", body = GroupDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_group(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateGroupDto>,
) -> Result<HttpResponse, ApiError> {
    let group = state
        .db
        .group_repo()
        .create(
            agrocore_domain::TenantId(auth.0.tenant_id),
            dto.0.into(),
            auth.0.user_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(GroupDto::from(group)))
}

#[utoipa::path(
    get,
    path = "/groups/{id}",
    responses(
        (status = 200, description = "Group details", body = GroupDto),
        (status = 404, description = "Group not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_group(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let group = state
        .db
        .group_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Group not found".into()))?;
    Ok(HttpResponse::Ok().json(GroupDto::from(group)))
}

#[utoipa::path(
    put,
    path = "/groups/{id}",
    request_body = UpdateGroupDto,
    responses(
        (status = 200, description = "Group updated", body = GroupDto),
        (status = 404, description = "Group not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_group(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateGroupDto>,
) -> Result<HttpResponse, ApiError> {
    let group = state
        .db
        .group_repo()
        .update(
            agrocore_domain::TenantId(auth.0.tenant_id),
            *path,
            dto.0.into(),
            auth.0.user_id,
        )
        .await?
        .ok_or_else(|| SharedError::NotFound("Group not found".into()))?;
    Ok(HttpResponse::Ok().json(GroupDto::from(group)))
}

#[utoipa::path(
    delete,
    path = "/groups/{id}",
    responses(
        (status = 200, description = "Group deleted"),
        (status = 404, description = "Group not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_group(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    if state
        .db
        .group_repo()
        .delete(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?
    {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Group not found".into()).into())
    }
}

#[utoipa::path(
    get,
    path = "/groups/by-plot/{plot_id}",
    params(
        ("plot_id" = Uuid, Path, description = "Plot UUID"),
        ("page" = Option<u64>, Query, description = "Page number (0-indexed)"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List groups by plot", body = PaginatedGroupResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_groups_by_plot(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    query: web::Query<agrocore_shared::Pagination>,
) -> Result<HttpResponse, ApiError> {
    let result = state
        .db
        .group_repo()
        .find_by_plot(agrocore_domain::TenantId(auth.0.tenant_id), *path, query.0)
        .await?;

    let response = PaginatedGroupResponse {
        data: result.data.into_iter().map(GroupDto::from).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
        total_pages: result.total_pages,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/groups/{id}/children",
    responses(
        (status = 200, description = "List child groups", body = Vec<GroupDto>),
        (status = 404, description = "Group not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_children(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let children = state
        .db
        .group_repo()
        .find_children(agrocore_domain::TenantId(auth.0.tenant_id), *path)
        .await?;
    Ok(HttpResponse::Ok().json(children.into_iter().map(GroupDto::from).collect::<Vec<_>>()))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/groups")
            .route("", web::get().to(list_groups))
            .route("", web::post().to(create_group))
            .route("/{id}", web::get().to(get_group))
            .route("/{id}", web::put().to(update_group))
            .route("/{id}", web::delete().to(delete_group))
            .route("/by-plot/{plot_id}", web::get().to(list_groups_by_plot))
            .route("/{id}/children", web::get().to(list_children)),
    );
}
