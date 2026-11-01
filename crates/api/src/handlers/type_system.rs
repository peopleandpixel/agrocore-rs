use crate::AppState;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::type_system::{
    ApproveTypeEntryDto, CreateTypeEntryDto, MergeTypeEntryDto, PendingTypeEntryDto, TypeCategory,
    TypeEntry, TypeEntryUsage, TypeRef, UpdateTypeEntryDto,
};
use agrocore_shared::SharedError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// API response for a type category
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeCategoryDto {
    pub id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_hierarchical: bool,
}

impl From<TypeCategory> for TypeCategoryDto {
    fn from(cat: TypeCategory) -> Self {
        Self {
            id: cat.id,
            name: cat.name,
            label: cat.label,
            description: cat.description,
            is_hierarchical: cat.is_hierarchical,
        }
    }
}

/// API response for a type entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeEntryDto {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: Option<String>,
    pub parent_id: Option<Uuid>,
    pub parent_name: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
}

impl From<TypeEntry> for TypeEntryDto {
    fn from(entry: TypeEntry) -> Self {
        Self {
            id: entry.id,
            category_id: entry.category_id,
            category_name: None,
            parent_id: entry.parent_id,
            parent_name: None,
            tenant_id: entry.tenant_id.map(|t| t.0),
            name: entry.name,
            code: entry.code,
            description: entry.description,
            is_active: entry.is_active,
            sort_order: entry.sort_order,
        }
    }
}

/// Query params for listing types
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListTypesQuery {
    pub category_id: Option<Uuid>,
    pub include_children: Option<bool>,
    pub tenant_id: Option<Uuid>,
}

/// List all type categories
#[utoipa::path(
    get,
    path = "/types/categories",
    responses(
        (status = 200, description = "List all type categories", body = Vec<TypeCategoryDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_type_categories(
    state: web::Data<AppState>,
    _auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let categories = state.db.type_category_repo().find_all().await?;
    let result: Vec<TypeCategoryDto> = categories.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(result))
}

/// Get a type category by ID
#[utoipa::path(
    get,
    path = "/types/categories/{id}",
    responses(
        (status = 200, description = "Category details", body = TypeCategoryDto),
        (status = 404, description = "Category not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_type_category(
    state: web::Data<AppState>,
    _auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let category = state
        .db
        .type_category_repo()
        .find_by_id(*path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Type category not found".into()))?;
    Ok(HttpResponse::Ok().json(TypeCategoryDto::from(category)))
}

/// List type entries with optional filters
#[utoipa::path(
    get,
    path = "/types",
    params(
        ("category_id" = Option<Uuid>, Query, description = "Filter by category"),
        ("include_children" = Option<bool>, Query, description = "Include child entries")
    ),
    responses(
        (status = 200, description = "List type entries", body = Vec<TypeEntryDto>),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_types(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<ListTypesQuery>,
) -> Result<HttpResponse, ApiError> {
    let tid = TenantId(auth.0.tenant_id);

    let entries = if let Some(category_id) = query.category_id {
        state
            .db
            .type_entry_repo()
            .find_by_category(tid, category_id)
            .await?
    } else {
        // If no category specified, return root entries from all categories
        let categories = state.db.type_category_repo().find_all().await?;
        let mut all_entries = Vec::new();
        for cat in categories {
            let entries = state
                .db
                .type_entry_repo()
                .find_root_entries(tid, cat.id)
                .await?;
            all_entries.extend(entries);
        }
        all_entries
    };

    let result: Vec<TypeEntryDto> = entries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(result))
}

/// Get a type entry by ID
#[utoipa::path(
    get,
    path = "/types/{id}",
    responses(
        (status = 200, description = "Type entry details", body = TypeEntryDto),
        (status = 404, description = "Type not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_type(
    state: web::Data<AppState>,
    _auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let entry = state
        .db
        .type_entry_repo()
        .find_by_id(*path)
        .await?
        .ok_or_else(|| SharedError::NotFound("Type entry not found".into()))?;
    Ok(HttpResponse::Ok().json(TypeEntryDto::from(entry)))
}

/// Get child type entries
#[utoipa::path(
    get,
    path = "/types/{id}/children",
    responses(
        (status = 200, description = "Child type entries", body = Vec<TypeEntryDto>),
        (status = 404, description = "Type not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_type_children(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let tid = TenantId(auth.0.tenant_id);
    let children = state.db.type_entry_repo().find_children(tid, *path).await?;
    let result: Vec<TypeEntryDto> = children.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(result))
}

/// Create a new type entry (Admin/SuperAdmin only)
#[utoipa::path(
    post,
    path = "/types",
    request_body = CreateTypeEntryDto,
    responses(
        (status = 201, description = "Type created", body = TypeEntryDto),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_type(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateTypeEntryDto>,
) -> Result<HttpResponse, ApiError> {
    // Check Admin/SuperAdmin permission
    if !auth.is_admin() {
        return Err(SharedError::Forbidden("Only Admin/SuperAdmin can create types".into()).into());
    }

    let tid = TenantId(auth.0.tenant_id);
    let entry = state
        .db
        .type_entry_repo()
        .create(tid, dto.0.clone())
        .await?;
    Ok(HttpResponse::Created().json(TypeEntryDto::from(entry)))
}

/// Update a type entry (Admin/SuperAdmin only)
#[utoipa::path(
    put,
    path = "/types/{id}",
    request_body = UpdateTypeEntryDto,
    responses(
        (status = 200, description = "Type updated", body = TypeEntryDto),
        (status = 404, description = "Type not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_type(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
    dto: web::Json<UpdateTypeEntryDto>,
) -> Result<HttpResponse, ApiError> {
    // Check Admin/SuperAdmin permission
    if !auth.is_admin() {
        return Err(SharedError::Forbidden("Only Admin/SuperAdmin can update types".into()).into());
    }

    let tid = TenantId(auth.0.tenant_id);
    let entry = state
        .db
        .type_entry_repo()
        .update(tid, *path, dto.0.clone())
        .await?
        .ok_or_else(|| SharedError::NotFound("Type entry not found".into()))?;
    Ok(HttpResponse::Ok().json(TypeEntryDto::from(entry)))
}

/// Soft delete a type entry (Admin/SuperAdmin only)
#[utoipa::path(
    delete,
    path = "/types/{id}",
    responses(
        (status = 200, description = "Type deleted"),
        (status = 404, description = "Type not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_type(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    // Check Admin/SuperAdmin permission
    if !auth.is_admin() {
        return Err(SharedError::Forbidden("Only Admin/SuperAdmin can delete types".into()).into());
    }

    let tid = TenantId(auth.0.tenant_id);
    if state.db.type_entry_repo().soft_delete(tid, *path).await? {
        Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": true})))
    } else {
        Err(SharedError::NotFound("Type entry not found".into()).into())
    }
}

/// Get type reference by category name and code
#[utoipa::path(
    get,
    path = "/types/lookup/{category}/{code}",
    responses(
        (status = 200, description = "Type reference", body = TypeRef),
        (status = 404, description = "Type not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn lookup_type(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (category_name, code) = path.into_inner();
    let tid = TenantId(auth.0.tenant_id);

    // Find category by name
    let category = state
        .db
        .type_category_repo()
        .find_by_name(&category_name)
        .await?
        .ok_or_else(|| SharedError::NotFound("Type category not found".into()))?;

    // Find entry by category and code
    let entry = state
        .db
        .type_entry_repo()
        .find_by_category_and_code(tid, category.id, &code)
        .await?
        .ok_or_else(|| SharedError::NotFound("Type entry not found".into()))?;

    let type_ref = TypeRef {
        id: entry.id,
        name: entry.name,
        code: entry.code,
    };
    Ok(HttpResponse::Ok().json(type_ref))
}

/// Create a new type entry by a regular user (marked for review)
/// Any authenticated user can create a type entry; it will be pending review.
#[utoipa::path(
    post,
    path = "/types/user-created",
    request_body = CreateTypeEntryDto,
    responses(
        (status = 201, description = "Type created pending review", body = TypeEntryDto),
        (status = 400, description = "Invalid input")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_type_by_user(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<CreateTypeEntryDto>,
) -> Result<HttpResponse, ApiError> {
    let tid = TenantId(auth.0.tenant_id);
    let mut create_dto = dto.0.clone();
    // Always mark user-created entries for review
    create_dto.pending_review = Some(true);

    let entry = state
        .db
        .type_entry_repo()
        .create_with_user_id(tid, create_dto, auth.0.user_id)
        .await?;
    Ok(HttpResponse::Created().json(TypeEntryDto::from(entry)))
}

/// List all pending type entries (Admin/SuperAdmin only)
#[utoipa::path(
    get,
    path = "/types/pending-review",
    responses(
        (status = 200, description = "Pending type entries", body = Vec<PendingTypeEntryDto>),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_pending_type_entries(
    state: web::Data<AppState>,
    auth: AuthUser,
) -> Result<HttpResponse, ApiError> {
    if !auth.is_admin() {
        return Err(SharedError::Forbidden(
            "Only Admin/SuperAdmin can view pending type entries".into(),
        )
        .into());
    }

    let entries = state.db.type_entry_repo().find_pending_review().await?;
    Ok(HttpResponse::Ok().json(entries))
}

/// Approve a pending type entry (Admin/SuperAdmin only)
#[utoipa::path(
    post,
    path = "/types/approve",
    request_body = ApproveTypeEntryDto,
    responses(
        (status = 200, description = "Type entry approved"),
        (status = 404, description = "Type not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn approve_type_entry(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<ApproveTypeEntryDto>,
) -> Result<HttpResponse, ApiError> {
    if !auth.is_admin() {
        return Err(SharedError::Forbidden(
            "Only Admin/SuperAdmin can approve type entries".into(),
        )
        .into());
    }

    state
        .db
        .type_entry_repo()
        .approve_entry(dto.entry_id)
        .await?
        .ok_or_else(|| SharedError::NotFound("Type entry not found".into()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"approved": true})))
}

/// Merge a user-created type entry into an existing one (Admin/SuperAdmin only)
/// Updates all entity references from the old entry to the new one.
#[utoipa::path(
    post,
    path = "/types/merge",
    request_body = MergeTypeEntryDto,
    responses(
        (status = 200, description = "Type entries merged", body = serde_json::Value),
        (status = 404, description = "Type not found"),
        (status = 403, description = "Forbidden")
    ),
    security(("bearer_auth" = []))
)]
pub async fn merge_type_entry(
    state: web::Data<AppState>,
    auth: AuthUser,
    dto: web::Json<MergeTypeEntryDto>,
) -> Result<HttpResponse, ApiError> {
    if !auth.is_admin() {
        return Err(
            SharedError::Forbidden("Only Admin/SuperAdmin can merge type entries".into()).into(),
        );
    }

    let tid = TenantId(auth.0.tenant_id);
    let result = state
        .db
        .type_entry_repo()
        .merge_entry(tid, dto.old_entry_id, dto.new_entry_id)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "merged": true,
        "updated_entities": result,
    })))
}

/// Get entities using a specific type entry (for merge confirmation)
#[utoipa::path(
    get,
    path = "/types/{id}/usages",
    responses(
        (status = 200, description = "Entities using this type", body = Vec<TypeEntryUsage>),
        (status = 404, description = "Type not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_type_usages(
    state: web::Data<AppState>,
    _auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let usages = state.db.type_entry_repo().find_usages(*path).await?;
    Ok(HttpResponse::Ok().json(usages))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/types")
            .route("/categories", web::get().to(list_type_categories))
            .route("/categories/{id}", web::get().to(get_type_category))
            .route("", web::get().to(list_types))
            .route("", web::post().to(create_type))
            .route("/user-created", web::post().to(create_type_by_user))
            .route("/pending-review", web::get().to(list_pending_type_entries))
            .route("/approve", web::post().to(approve_type_entry))
            .route("/merge", web::post().to(merge_type_entry))
            .route("/{id}", web::get().to(get_type))
            .route("/{id}", web::put().to(update_type))
            .route("/{id}", web::delete().to(delete_type))
            .route("/{id}/children", web::get().to(get_type_children))
            .route("/{id}/usages", web::get().to(get_type_usages))
            .route("/lookup/{category}/{code}", web::get().to(lookup_type)),
    );
}
