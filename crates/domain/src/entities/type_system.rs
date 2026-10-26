use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::entities::tenant::TenantId;

/// A type classification category (e.g., "animal_species", "site_type", "order_type").
/// Each category contains type entries that replace a hardcoded enum.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TypeCategory {
    pub id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_hierarchical: bool,
    pub created_at: DateTime<Utc>,
}

/// A type entry within a category (e.g., "Chicken" in "animal_species").
/// Supports hierarchy via parent_id and tenant isolation via tenant_id.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct TypeEntry {
    pub id: Uuid,
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub tenant_id: Option<TenantId>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    /// User who created this entry via the "Add new type" feature.
    /// NULL means system-created (migration or admin).
    pub created_by_user_id: Option<Uuid>,
    /// True when a user-created entry needs admin review.
    pub pending_review: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight type reference for embedding in entity responses.
/// Includes id, name, and code for display purposes.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeRef {
    pub id: Uuid,
    pub name: String,
    pub code: String,
}

/// DTO for creating a new type entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTypeEntryDto {
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub code: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    /// If true, the entry is marked for admin review (user-created).
    pub pending_review: Option<bool>,
}

/// DTO for updating an existing type entry.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateTypeEntryDto {
    pub parent_id: Option<Option<Uuid>>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// DTO for creating a new type category.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTypeCategoryDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub description: Option<String>,
    pub is_hierarchical: bool,
}

/// DTO for updating an existing type category.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateTypeCategoryDto {
    pub label: Option<String>,
    pub description: Option<Option<String>>,
    pub is_hierarchical: Option<bool>,
}

/// Nested type entry for tree-like hierarchy display.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeHierarchyNode {
    pub id: Uuid,
    pub category_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub children: Vec<TypeHierarchyNode>,
}

/// DTO for a pending type entry with user information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct PendingTypeEntryDto {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub parent_id: Option<Uuid>,
    pub tenant_id: Option<TenantId>,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by_user_id: Option<Uuid>,
    pub created_by_email: Option<String>,
    pub created_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for merging a user-created type entry into an existing one
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct MergeTypeEntryDto {
    /// The user-created entry to merge away
    pub old_entry_id: Uuid,
    /// The existing entry to merge into
    pub new_entry_id: Uuid,
}

/// DTO for approving a pending type entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApproveTypeEntryDto {
    pub entry_id: Uuid,
}

/// Information about entities affected by a type entry (for merge confirmation)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeEntryUsage {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
}

impl TypeEntry {
    /// Returns true if this is a global type (visible to all tenants).
    pub fn is_global(&self) -> bool {
        self.tenant_id.is_none()
    }

    /// Returns true if this entry has children (is a parent node).
    pub fn has_children(&self) -> bool {
        // This is a hint; actual check requires querying children
        self.parent_id.is_none()
    }

    /// Returns the path from this entry to the root of the hierarchy.
    /// This is a placeholder — actual implementation requires repository queries.
    pub fn path_to_root(&self) -> Vec<Uuid> {
        let mut path = vec![self.id];
        let current_parent = self.parent_id;
        if let Some(parent_id) = current_parent {
            path.push(parent_id);
        }
        path.reverse();
        path
    }
}
