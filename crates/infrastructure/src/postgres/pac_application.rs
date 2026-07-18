use crate::postgres::audit_log::PgAuditLogRepo;
use agrocore_domain::entities::finance::{
    CreatePACApplicationDto, PACApplication, UpdatePACApplicationDto,
};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::repositories::{
    AuditLogRepo, PACApplicationRepo, PaginatedResponse, Pagination, RepositoryFuture,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PgPACApplicationRepo {
    pool: PgPool,
}
impl PgPACApplicationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PACApplicationRepo for PgPACApplicationRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<PACApplication>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, PACApplication>(
                "SELECT * FROM pac_applications WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tid)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<PACApplication>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);
            if can_see_all {
                sqlx::query_as::<_, PACApplication>(
                    "SELECT * FROM pac_applications WHERE tenant_id = $1 AND id = $2",
                )
                .bind(tid)
                .bind(id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                // Workers might not see financial records at all or only those they are linked to.
                // For now, only Admin/Manager can see PAC applications.
                Ok(None)
            }
        })
    }
    fn find_all(
        &self,
        tid: TenantId,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pac_applications WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PACApplication> = sqlx::query_as("SELECT * FROM pac_applications WHERE tenant_id = $1 ORDER BY year DESC, created_at DESC LIMIT $2 OFFSET $3")
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
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }
    fn create(
        &self,
        tid: TenantId,
        dto: CreatePACApplicationDto,
        by: Uuid,
    ) -> RepositoryFuture<PACApplication> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            let entity = sqlx::query_as::<_, PACApplication>(
                "INSERT INTO pac_applications (id, tenant_id, year, application_number, status, total_eligible_area, eco_schemes, documents_urls) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
            )
            .bind(Uuid::new_v4())
            .bind(tid)
            .bind(dto.year)
            .bind(dto.application_number)
            .bind(serde_json::to_value(agrocore_domain::entities::finance::PACStatus::Draft).unwrap())
            .bind(dto.total_eligible_area)
            .bind(serde_json::to_value(dto.eco_schemes).unwrap())
            .bind(serde_json::to_value(Vec::<String>::new()).unwrap())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let _ = audit_repo
                .create(
                    tid,
                    agrocore_domain::entities::compliance::CreateAuditLogDto {
                        tenant_id: tid,
                        user_id: by,
                        action: agrocore_domain::entities::compliance::AuditAction::Created,
                        entity_type: "PACApplication".into(),
                        entity_id: entity.id,
                        old_value: None,
                        new_value: Some(serde_json::to_value(&entity).unwrap()),
                        ip_address: None,
                    },
                )
                .await;

            Ok(entity)
        })
    }
    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdatePACApplicationDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<PACApplication>> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            let old_val = sqlx::query_as::<_, PACApplication>(
                "SELECT * FROM pac_applications WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tid)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let mut query = String::from("UPDATE pac_applications SET updated_at = NOW()");
            let mut idx = 1;

            if dto.application_number.is_some() {
                query.push_str(&format!(", application_number = ${}", idx + 2));
                idx += 1;
            }
            if dto.status.is_some() {
                query.push_str(&format!(", status = ${}", idx + 2));
                idx += 1;
            }
            if dto.total_eligible_area.is_some() {
                query.push_str(&format!(", total_eligible_area = ${}", idx + 2));
                idx += 1;
            }
            if dto.eco_schemes.is_some() {
                query.push_str(&format!(", eco_schemes = ${}", idx + 2));
                idx += 1;
            }
            if dto.documents_urls.is_some() {
                query.push_str(&format!(", documents_urls = ${}", idx + 2));
            }

            query.push_str(" WHERE tenant_id = $1 AND id = $2 RETURNING *");

            let mut q = sqlx::query_as::<_, PACApplication>(&query)
                .bind(tid)
                .bind(id);

            if let Some(v) = dto.application_number {
                q = q.bind(v);
            }
            if let Some(v) = dto.status {
                q = q.bind(serde_json::to_value(v).unwrap());
            }
            if let Some(v) = dto.total_eligible_area {
                q = q.bind(v);
            }
            if let Some(v) = dto.eco_schemes {
                q = q.bind(serde_json::to_value(v).unwrap());
            }
            if let Some(v) = dto.documents_urls {
                q = q.bind(serde_json::to_value(v).unwrap());
            }

            let new_val = q
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            if let (Some(old), Some(new)) = (&old_val, &new_val) {
                let _ = audit_repo
                    .create(
                        tid,
                        agrocore_domain::entities::compliance::CreateAuditLogDto {
                            tenant_id: tid,
                            user_id: by,
                            action: agrocore_domain::entities::compliance::AuditAction::Updated,
                            entity_type: "PACApplication".into(),
                            entity_id: new.id,
                            old_value: Some(serde_json::to_value(old).unwrap()),
                            new_value: Some(serde_json::to_value(new).unwrap()),
                            ip_address: None,
                        },
                    )
                    .await;
            }

            Ok(new_val)
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        let audit_repo = PgAuditLogRepo::new(pool.clone());
        Box::pin(async move {
            // Get old value for audit before deletion
            let old_val = sqlx::query_as::<_, PACApplication>(
                "SELECT * FROM pac_applications WHERE tenant_id = $1 AND id = $2",
            )
            .bind(tid)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let res = sqlx::query("DELETE FROM pac_applications WHERE tenant_id = $1 AND id = $2")
                .bind(tid)
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let success = res.rows_affected() > 0;

            #[allow(clippy::collapsible_if)]
            if success {
                if let Some(old) = old_val {
                    let _ = audit_repo
                        .create(
                            tid,
                            agrocore_domain::entities::compliance::CreateAuditLogDto {
                                tenant_id: tid,
                                user_id: Uuid::nil(), // No user_id in delete trait, using nil
                                action: agrocore_domain::entities::compliance::AuditAction::Deleted,
                                entity_type: "PACApplication".into(),
                                entity_id: id,
                                old_value: Some(serde_json::to_value(old).unwrap()),
                                new_value: None,
                                ip_address: None,
                            },
                        )
                        .await;
                }
            }

            Ok(success)
        })
    }
    fn find_by_year(
        &self,
        tid: TenantId,
        year: i32,
        p: Pagination,
    ) -> RepositoryFuture<PaginatedResponse<PACApplication>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pac_applications WHERE tenant_id = $1 AND year = $2",
            )
            .bind(tid)
            .bind(year)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<PACApplication> = sqlx::query_as("SELECT * FROM pac_applications WHERE tenant_id = $1 AND year = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
                .bind(tid)
                .bind(year)
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
                data,
                total: total as u64,
                page,
                per_page,
                total_pages,
            })
        })
    }
}
