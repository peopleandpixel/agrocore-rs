use agrocore_domain::entities::site::{CreateSiteDto, GeoPoint, Site, UpdateSiteDto};
use agrocore_domain::entities::spatial::SpatialObject;
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, SiteRepository, SpatialObjectRepository,
};
use agrocore_shared::SharedError;
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = agrocore_shared::Result<T>> + Send>>;

#[derive(Clone)]
pub struct PgSiteRepo {
    pool: PgPool,
}

impl PgSiteRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SiteRepository for PgSiteRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Site>(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary
                   FROM sites WHERE id = $1 AND tenant_id = $2"#,
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        roles: &[UserRole],
    ) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all =
                roles_vec.contains(&UserRole::Admin) || roles_vec.contains(&UserRole::Manager);
            if can_see_all {
                sqlx::query_as::<_, Site>(
                    r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                       area, gross_area, plots, row_config, bbch_stage, planted_date,
                       cleared_date, soil_type, slope, slope_facing, altitude, organic,
                       organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                       note1, note2, is_active, is_temporary, created_at, updated_at,
                       created_by, updated_by, center, boundary
                       FROM sites WHERE id = $1 AND tenant_id = $2"#,
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            } else {
                sqlx::query_as::<_, Site>(
                    r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                       area, gross_area, plots, row_config, bbch_stage, planted_date,
                       cleared_date, soil_type, slope, slope_facing, altitude, organic,
                       organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                       note1, note2, is_active, is_temporary, created_at, updated_at,
                       created_by, updated_by, center, boundary
                       FROM sites WHERE id = $1 AND tenant_id = $2"#,
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
            }
        })
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites WHERE tenant_id = $1")
                .bind(tid)
                .fetch_one(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<Site> = sqlx::query_as(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary
                   FROM sites WHERE tenant_id = $1 LIMIT $2 OFFSET $3"#,
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }

    fn find_all_visible(
        &self,
        tid: TenantId,
        p: Pagination,
        _user_id: Uuid,
        _roles: &[UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Site>> {
        SiteRepository::find_all(self, tid, p)
    }

    fn create(&self, tid: TenantId, dto: CreateSiteDto, _by: Uuid) -> RepositoryFuture<Site> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            sqlx::query_as::<_, Site>(
                r#"INSERT INTO sites (id, tenant_id, label, site_type, crop_type, area, center, boundary, is_active, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, NOW(), NOW())
                   RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary"#)
            .bind(id)
            .bind(tid)
            .bind(&dto.label)
            .bind(serde_json::to_value(&dto.site_type).unwrap())
            .bind(serde_json::to_value(&dto.crop_type).unwrap())
            .bind(dto.area)
            .bind(&dto.center)
            .bind(&dto.boundary)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateSiteDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, Site>(
                r#"UPDATE sites SET 
                   label = COALESCE($1, label),
                   center = COALESCE($2, center),
                   boundary = COALESCE($3, boundary),
                   updated_at = NOW()
                   WHERE id = $4 AND tenant_id = $5 
                   RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary"#,
            )
            .bind(&dto.label)
            .bind(&dto.center)
            .bind(&dto.boundary)
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query("UPDATE sites SET is_active = false WHERE id = $1 AND tenant_id = $2")
                .bind(id)
                .bind(tid)
                .execute(&pool)
                .await
                .map(|r| r.rows_affected() > 0)
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}

impl SpatialObjectRepository for PgSiteRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<SpatialObject>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, SpatialObject>(
                r#"SELECT id, tenant_id, site_id, parent_id, label, object_type, geometry,
                   area, buffer_meters, properties, custom_fields, note, is_active,
                   is_temporary, created_at, updated_at, created_by, updated_by
                   FROM spatial_objects WHERE id = $1 AND tenant_id = $2"#,
            )
            .bind(id)
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
    ) -> RepositoryFuture<PaginatedResponse<SpatialObject>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        let offset = page * per_page;

        Box::pin(async move {
            let total: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM spatial_objects WHERE tenant_id = $1")
                    .bind(tid)
                    .fetch_one(&pool)
                    .await
                    .map_err(|e| SharedError::Database(e.to_string()))?;

            let data: Vec<SpatialObject> = sqlx::query_as(
                r#"SELECT id, tenant_id, site_id, parent_id, label, object_type, geometry,
                   area, buffer_meters, properties, custom_fields, note, is_active,
                   is_temporary, created_at, updated_at, created_by, updated_by
                   FROM spatial_objects WHERE tenant_id = $1 LIMIT $2 OFFSET $3"#,
            )
            .bind(tid)
            .bind(per_page as i32)
            .bind(offset as i32)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: ((total as f64 / per_page as f64).ceil() as u64),
            })
        })
    }
    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query(
                "UPDATE spatial_objects SET is_active = false WHERE id = $1 AND tenant_id = $2",
            )
            .bind(id)
            .bind(tid)
            .execute(&pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
    fn find_containing_point(
        &self,
        tid: TenantId,
        point: GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<SpatialObject>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut query = String::from(
                r#"SELECT id, tenant_id, site_id, parent_id, label, object_type, geometry,
                   area, buffer_meters, properties, custom_fields, note, is_active,
                   is_temporary, created_at, updated_at, created_by, updated_by
                   FROM spatial_objects 
                   WHERE ST_Contains(geometry, $1) AND tenant_id = $2 AND is_active = true"#,
            );
            if site_id.is_some() {
                query.push_str(" AND site_id = $3");
            }

            let mut q = sqlx::query_as::<_, SpatialObject>(&query)
                .bind(&point)
                .bind(tid);

            if let Some(sid) = site_id {
                q = q.bind(sid);
            }

            q.fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
        })
    }
}
