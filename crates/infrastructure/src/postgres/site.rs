use agrocore_domain::TenantId;
use agrocore_domain::entities::site::{LpisData, RowConfig, SigpacData, Site, SiteProperty};
use agrocore_domain::entities::spatial::types::{Boundary, GeoPoint, Plot};
use agrocore_domain::entities::user::UserRole;
use agrocore_domain::entities::{BbchStage, CropType, SiteType};
use agrocore_domain::repositories::{
    PaginatedResponse, Pagination, RepositoryFuture, SiteRepository, SpatialObjectRepository,
};
use agrocore_shared::SharedError;
use agrocore_shared::lpis::LpisCountry;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::prelude::FromRow;
use uuid::Uuid;

agrocore_shared::pg_repo!(PgSiteRepo);

impl SiteRepository for PgSiteRepo {
    fn find_by_id(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, SiteDb>(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                   properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary
                   FROM sites WHERE id = $1 AND tenant_id = $2"#,
            )
            .bind(id)
            .bind(tid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
            .map(|opt| opt.map(Site::from))
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        roles: &[agrocore_domain::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        let roles_vec = roles.to_vec();
        Box::pin(async move {
            let can_see_all = roles_vec.contains(&agrocore_domain::entities::user::UserRole::Admin)
                || roles_vec.contains(&agrocore_domain::entities::user::UserRole::Manager);
            if can_see_all {
                sqlx::query_as::<_, SiteDb>(
                    r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                       area, gross_area, plots, row_config, bbch_stage, planted_date,
                       cleared_date, soil_type, slope, slope_facing, altitude, organic,
                       organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                       properties, custom_fields,
                       note1, note2, is_active, is_temporary, created_at, updated_at,
                       created_by, updated_by, center, boundary
                       FROM sites WHERE id = $1 AND tenant_id = $2"#,
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
                .map(|opt| opt.map(Site::from))
            } else {
                sqlx::query_as::<_, SiteDb>(
                    r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                       area, gross_area, plots, row_config, bbch_stage, planted_date,
                       cleared_date, soil_type, slope, slope_facing, altitude, organic,
                       organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                       properties, custom_fields,
                       note1, note2, is_active, is_temporary, created_at, updated_at,
                       created_by, updated_by, center, boundary
                       FROM sites WHERE id = $1 AND tenant_id = $2"#,
                )
                .bind(id)
                .bind(tid)
                .fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
                .map(|opt| opt.map(Site::from))
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

            let data: Vec<SiteDb> = sqlx::query_as::<_, SiteDb>(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                   properties, custom_fields,
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
                data: data.into_iter().map(Site::from).collect(),
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
        _roles: &[agrocore_domain::entities::user::UserRole],
    ) -> RepositoryFuture<PaginatedResponse<Site>> {
        SiteRepository::find_all(self, tid, p)
    }

    fn create(
        &self,
        tid: TenantId,
        dto: agrocore_domain::entities::site::CreateSiteDto,
        by: Uuid,
    ) -> RepositoryFuture<Site> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let id = Uuid::new_v4();
            let site_type_val = serde_json::to_value(&dto.site_type).unwrap();
            let crop_type_val = serde_json::to_value(&dto.crop_type).unwrap();
            let plots_val = serde_json::to_value(&dto.plots.unwrap_or_default()).unwrap();
            let row_config_val = dto.row_config.map(|r| serde_json::to_value(r).unwrap());
            let bbch_stage_val = dto.bbch_stage.map(|b| serde_json::to_value(b).unwrap());
            let properties_val = dto
                .properties
                .map(|p| serde_json::to_value(p).unwrap())
                .unwrap_or_else(|| serde_json::json!([]));
            let boundary_val = dto.boundary.map(|b| serde_json::to_value(b).unwrap());

            sqlx::query_as::<_, SiteDb>(
                r#"INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                   properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                   $14, $15, $16, $17, $18, $19, $20, $21, $22, $23,
                   $24, $25,
                   $26, $27, true, false, NOW(), NOW(),
                   $28, $29, $30, $31)
                   RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                   properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary"#
            )
            .bind(Uuid::new_v4())
            .bind(tid)
            .bind(None::<Uuid>)
            .bind(&dto.label)
            .bind(&site_type_val)
            .bind(&crop_type_val)
            .bind(&dto.variety)
            .bind(dto.area)
            .bind(&dto.gross_area)
            .bind(&plots_val)
            .bind(&row_config_val)
            .bind(&bbch_stage_val)
            .bind(&dto.planted_date)
            .bind(&dto.cleared_date)
            .bind(&dto.soil_type)
            .bind(&dto.slope)
            .bind(&dto.slope_facing)
            .bind(&dto.altitude)
            .bind(&dto.organic)
            .bind(&None::<bool>)
            .bind(&serde_json::json!(null))
            .bind(&None::<String>)
            .bind(&None::<agrocore_shared::lpis::LpisCountry>)
            .bind(&serde_json::json!(null))
            .bind(&properties_val)
            .bind(&serde_json::json!({}))
            .bind(&None::<String>)
            .bind(&None::<String>)
            .bind(&None::<Uuid>)
            .bind(&None::<Uuid>)
            .bind(&dto.center)
            .bind(&boundary_val)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))
            .map(Site::from)
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: agrocore_domain::entities::site::UpdateSiteDto,
        _by: Uuid,
    ) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let site_type_val = dto
                .site_type
                .as_ref()
                .map(|s| serde_json::to_value(s).unwrap());
            let crop_type_val = dto
                .crop_type
                .as_ref()
                .map(|c| serde_json::to_value(c).unwrap());
            let plots_val = dto.plots.as_ref().map(|p| serde_json::to_value(p).unwrap());
            let row_config_val = dto
                .row_config
                .as_ref()
                .map(|r| serde_json::to_value(r).unwrap());
            let bbch_stage_val = dto
                .bbch_stage
                .as_ref()
                .map(|b| serde_json::to_value(b).unwrap());
            let sigpac_data_val = dto
                .sigpac_data
                .as_ref()
                .map(|s| serde_json::to_value(s).unwrap());
            let lpis_data_val = dto
                .lpis_data
                .as_ref()
                .map(|l| serde_json::to_value(l).unwrap());
            let lpis_country_val = dto
                .lpis_country
                .as_ref()
                .map(|c| serde_json::to_value(c).unwrap());
            let properties_val = dto
                .properties
                .as_ref()
                .map(|p| serde_json::to_value(p).unwrap());
            let custom_fields_val = dto
                .custom_fields
                .as_ref()
                .map(|c| serde_json::to_value(c).unwrap());
            let boundary_val = dto
                .boundary
                .as_ref()
                .map(|b| serde_json::to_value(b).unwrap());

            let mut query = r#"UPDATE sites SET 
                   label = COALESCE($1, label),
                   site_type = COALESCE($2, site_type),
                   crop_type = COALESCE($3, crop_type),
                   variety = COALESCE($4, variety),
                   area = COALESCE($5, area),
                   gross_area = COALESCE($6, gross_area),
                   plots = COALESCE($7, plots),
                   row_config = COALESCE($8, row_config),
                   bbch_stage = COALESCE($9, bbch_stage),
                   planted_date = COALESCE($10, planted_date),
                   cleared_date = COALESCE($11, cleared_date),
                   soil_type = COALESCE($12, soil_type),
                   slope = COALESCE($13, slope),
                   slope_facing = COALESCE($14, slope_facing),
                   altitude = COALESCE($15, altitude),
                   organic = COALESCE($16, organic),
                   organic_eligible = COALESCE($17, organic_eligible),
                   sigpac_data = COALESCE($18, sigpac_data),
                   regepac_id = COALESCE($19, regepac_id),
                   lpis_country = COALESCE($20, lpis_country),
                   lpis_data = COALESCE($21, lpis_data),
                   properties = COALESCE($22, properties),
                   custom_fields = COALESCE($23, custom_fields),
                   note1 = COALESCE($24, note1),
                   note2 = COALESCE($25, note2),
                   is_active = COALESCE($26, is_active),
                   is_temporary = COALESCE($27, is_temporary),
                   updated_at = NOW(),
                   center = COALESCE($28, center),
                   boundary = COALESCE($29, boundary)
                   WHERE id = $30 AND tenant_id = $31
                   RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, plots, row_config, bbch_stage, planted_date,
                   cleared_date, soil_type, slope, slope_facing, altitude, organic,
                   organic_eligible, sigpac_data, regepac_id, lpis_country, lpis_data,
                   properties, custom_fields,
                   note1, note2, is_active, is_temporary, created_at, updated_at,
                   created_by, updated_by, center, boundary"#
                .to_string();

            let mut q = sqlx::query_as::<_, SiteDb>(&query);
            q = q.bind(&dto.label);
            q = q.bind(&site_type_val);
            q = q.bind(&crop_type_val);
            q = q.bind(&dto.variety);
            q = q.bind(&dto.area);
            q = q.bind(&dto.gross_area);
            q = q.bind(&plots_val);
            q = q.bind(&row_config_val);
            q = q.bind(&bbch_stage_val);
            q = q.bind(&dto.planted_date);
            q = q.bind(&dto.cleared_date);
            q = q.bind(&dto.soil_type);
            q = q.bind(&dto.slope);
            q = q.bind(&dto.slope_facing);
            q = q.bind(&dto.altitude);
            q = q.bind(&dto.organic);
            q = q.bind(&dto.organic_eligible);
            q = q.bind(&sigpac_data_val);
            q = q.bind(&dto.regepac_id);
            q = q.bind(&lpis_country_val);
            q = q.bind(&lpis_data_val);
            q = q.bind(&properties_val);
            q = q.bind(&custom_fields_val);
            q = q.bind(&dto.note1);
            q = q.bind(&dto.note2);
            q = q.bind(&dto.is_active);
            q = q.bind(&dto.is_temporary);
            q = q.bind(&dto.center);
            q = q.bind(&boundary_val);
            q = q.bind(id);
            q = q.bind(tid);

            q.fetch_optional(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))
                .map(|opt| opt.map(Site::from))
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

#[derive(Debug, FromRow)]
struct SiteDb {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub business_id: Option<Uuid>,
    pub label: String,
    pub site_type: serde_json::Value,
    pub crop_type: serde_json::Value,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub plots: serde_json::Value,
    pub row_config: Option<serde_json::Value>,
    pub bbch_stage: Option<serde_json::Value>,
    pub planted_date: Option<DateTime<Utc>>,
    pub cleared_date: Option<DateTime<Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub sigpac_data: Option<serde_json::Value>,
    pub regepac_id: Option<String>,
    pub lpis_country: Option<serde_json::Value>,
    pub lpis_data: Option<serde_json::Value>,
    pub properties: serde_json::Value,
    pub custom_fields: serde_json::Value,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: bool,
    pub is_temporary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub center: Option<serde_json::Value>,
    pub boundary: Option<serde_json::Value>,
}

impl From<SiteDb> for Site {
    fn from(db: SiteDb) -> Self {
        Site {
            id: db.id,
            tenant_id: db.tenant_id.into(),
            business_id: db.business_id,
            label: db.label,
            site_type: serde_json::from_value(db.site_type)
                .unwrap_or(SiteType::Other("other".to_string())),
            crop_type: serde_json::from_value(db.crop_type).unwrap_or(CropType::Unknown),
            variety: db.variety,
            area: db.area,
            gross_area: db.gross_area,
            plots: serde_json::from_value(db.plots).unwrap_or_default(),
            row_config: db.row_config.and_then(|v| serde_json::from_value(v).ok()),
            bbch_stage: db.bbch_stage.and_then(|v| serde_json::from_value(v).ok()),
            planted_date: db.planted_date,
            cleared_date: db.cleared_date,
            soil_type: db.soil_type,
            slope: db.slope,
            slope_facing: db.slope_facing,
            altitude: db.altitude,
            organic: db.organic,
            organic_eligible: db.organic_eligible,
            sigpac_data: db.sigpac_data.and_then(|v| serde_json::from_value(v).ok()),
            regepac_id: db.regepac_id,
            lpis_country: db.lpis_country.and_then(|v| serde_json::from_value(v).ok()),
            lpis_data: db.lpis_data.and_then(|v| serde_json::from_value(v).ok()),
            properties: serde_json::from_value(db.properties).unwrap_or_default(),
            custom_fields: Some(db.custom_fields),
            note1: db.note1,
            note2: db.note2,
            is_active: db.is_active,
            is_temporary: db.is_temporary,
            created_at: db.created_at,
            updated_at: db.updated_at,
            created_by: db.created_by,
            updated_by: db.updated_by,
            center: db.center.and_then(|v| serde_json::from_value(v).ok()),
            boundary: db.boundary.and_then(|v| serde_json::from_value(v).ok()),
        }
    }
}

impl SpatialObjectRepository for PgSiteRepo {
    fn find_by_id(
        &self,
        tid: TenantId,
        id: Uuid,
    ) -> RepositoryFuture<Option<agrocore_domain::entities::spatial::SpatialObject>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            sqlx::query_as::<_, agrocore_domain::entities::spatial::SpatialObject>(
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
    ) -> RepositoryFuture<PaginatedResponse<agrocore_domain::entities::spatial::SpatialObject>>
    {
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

            let data: Vec<agrocore_domain::entities::spatial::SpatialObject> = sqlx::query_as(
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
            sqlx::query("DELETE FROM spatial_objects WHERE id = $1 AND tenant_id = $2")
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
        point: agrocore_domain::entities::spatial::types::GeoPoint,
        site_id: Option<Uuid>,
    ) -> RepositoryFuture<Vec<agrocore_domain::entities::spatial::SpatialObject>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut query =
                r#"SELECT id, tenant_id, site_id, parent_id, label, object_type, geometry,
                   area, buffer_meters, properties, custom_fields, note, is_active,
                   is_temporary, created_at, updated_at, created_by, updated_by
                   FROM spatial_objects WHERE tenant_id = $1 AND is_active = true"#
                    .to_string();
            let mut param_idx = 2;

            if let Some(site_id) = site_id {
                query.push_str(&format!(" AND site_id = ${}", param_idx));
                param_idx += 1;
            }

            let mut q =
                sqlx::query_as::<_, agrocore_domain::entities::spatial::SpatialObject>(&query)
                    .bind(tid);

            if let Some(site_id) = site_id {
                q = q.bind(site_id);
            }

            let objects: Vec<agrocore_domain::entities::spatial::SpatialObject> = q
                .fetch_all(&pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

            // Filter objects containing the point in Rust
            Ok(objects
                .into_iter()
                .filter(|obj| obj.contains_point(&point))
                .collect())
        })
    }
}
