use agrocore_domain::entities::site::{CreateSiteDto, GeoPoint, Site, UpdateSiteDto};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_domain::repositories::{PaginatedResponse, Pagination, RepositoryFuture, SiteRepository};
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use sqlx::PgPool;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

/// Helper to convert Vec<GeoPoint> to PostGIS WKT polygon string
fn boundary_to_wkt(points: &[GeoPoint]) -> String {
    if points.len() < 3 {
        return "POLYGON EMPTY".to_string();
    }
    let coords: Vec<String> = points
        .iter()
        .map(|p| format!("{} {}", p.lng, p.lat))
        .collect();
    format!("POLYGON(({}))", coords.join(", "))
}

/// Helper to convert PostGIS WKT geometry back to Vec<GeoPoint>
fn wkt_to_boundary(wkt: Option<String>) -> Option<Vec<GeoPoint>> {
    wkt.and_then(|s| {
        if !s.starts_with("POLYGON") {
            return None;
        }
        s.strip_prefix("POLYGON((")
            .and_then(|s| s.strip_suffix("))"))
            .map(|coords_str| {
                coords_str
                    .split(',')
                    .filter_map(|coord| {
                        let parts: Vec<&str> = coord.trim().split_whitespace().collect();
                        if parts.len() == 2 {
                            Some(GeoPoint {
                                lng: parts[0].parse().unwrap_or(0.0),
                                lat: parts[1].parse().unwrap_or(0.0),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
    })
}

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
            let row = sqlx::query(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, 
                   ST_AsText(boundary) as "boundary_wkt",
                   ST_AsText(center) as "center_wkt",
                   is_active, is_temporary, created_at, updated_at, created_by, updated_by,
                   plots, row_config, bbch_stage, planted_date, cleared_date,
                   soil_type, slope, slope_facing, altitude, organic, organic_eligible,
                   sigpac_data, properties, custom_fields, note1, note2
                FROM sites WHERE tenant_id = $1 AND id = $2"#
            )
            .bind(tid.to_string())
            .bind(id.to_string())
            .fetch_optional(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))?;

            row.map(|r| {
                let boundary_wkt: Option<String> = r.get("boundary_wkt");
                let center_wkt: Option<String> = r.get("center_wkt");
                row_to_site(r, boundary_wkt, center_wkt)
            })
        })
    }

    fn find_by_id_visible(
        &self,
        tid: TenantId,
        id: Uuid,
        _user_id: Uuid,
        _roles: &[agrocore_domain::entities::user::UserRole],
    ) -> RepositoryFuture<Option<Site>> {
        self.find_by_id(tid, id)
    }

    fn find_all(&self, tid: TenantId, p: Pagination) -> RepositoryFuture<PaginatedResponse<Site>> {
        let pool = self.pool.clone();
        let page = p.page.unwrap_or(0);
        let per_page = p.per_page.unwrap_or(20);
        Box::pin(async move {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sites WHERE tenant_id = $1"
            )
            .bind(tid.to_string())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))? as i64;

            let rows = sqlx::query(
                r#"SELECT id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, 
                   ST_AsText(boundary) as "boundary_wkt",
                   ST_AsText(center) as "center_wkt",
                   is_active, is_temporary, created_at, updated_at, created_by, updated_by,
                   plots, row_config, bbch_stage, planted_date, cleared_date,
                   soil_type, slope, slope_facing, altitude, organic, organic_eligible,
                   sigpac_data, properties, custom_fields, note1, note2
                FROM sites WHERE tenant_id = $1 AND is_active = true
                ORDER BY updated_at DESC LIMIT $2 OFFSET $3"#
            )
            .bind(tid.to_string())
            .bind(per_page as i64)
            .bind((page * per_page) as i64)
            .fetch_all(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))?;

            let data: Vec<Site> = rows.into_iter().map(|r| {
                let boundary_wkt: Option<String> = r.get("boundary_wkt");
                let center_wkt: Option<String> = r.get("center_wkt");
                row_to_site(r, boundary_wkt, center_wkt)
            }).collect();

            Ok(PaginatedResponse {
                data,
                total: total as u64,
                page,
                per_page,
                total_pages: (total as f64 / per_page as f64).ceil() as u64,
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
        self.find_all(tid, p)
    }

    fn create(&self, tid: TenantId, dto: CreateSiteDto, by: Uuid) -> RepositoryFuture<Site> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let boundary_wkt = boundary_to_wkt(&dto.boundary.unwrap_or_default());
            let boundary_wkt = if boundary_wkt == "POLYGON EMPTY" { None } else { Some(boundary_wkt) };
            let plots_json = serde_json::to_value(&dto.plots.unwrap_or_default()).unwrap_or(serde_json::Value::Array(vec![]));

            let row = sqlx::query(
                r#"INSERT INTO sites (id, tenant_id, business_id, label, site_type, crop_type, variety,
                   area, gross_area, boundary, center,
                   is_active, is_temporary, created_at, updated_at, created_by, updated_by,
                   plots, row_config, bbch_stage, planted_date, cleared_date,
                   soil_type, slope, slope_facing, altitude, organic, organic_eligible,
                   sigpac_data, properties, custom_fields, note1, note2)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 
                        ST_GeomFromText($10, 4326), ST_Point($11, $12),
                        true, false, $13, $14, $15, $16,
                        $17, $18, $19, $20, $21,
                        $22, $23, $24, $25, $26, $27,
                        $28, $29, $30, $31, $32)
                RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                          area, gross_area,
                          ST_AsText(boundary) as "boundary_wkt",
                          ST_AsText(center) as "center_wkt",
                          is_active, is_temporary, created_at, updated_at, created_by, updated_by,
                          plots, row_config, bbch_stage, planted_date, cleared_date,
                          soil_type, slope, slope_facing, altitude, organic, organic_eligible,
                          sigpac_data, properties, custom_fields, note1, note2"#
            )
            .bind(Uuid::new_v4().to_string())
            .bind(tid.to_string())
            .bind(dto.business_id.map(|v| v.to_string()))
            .bind(&dto.label)
            .bind(format!("{:?}", dto.site_type))
            .bind(format!("{:?}", dto.crop_type))
            .bind(&dto.variety)
            .bind(dto.area)
            .bind(dto.gross_area)
            .bind(&boundary_wkt)
            .bind(dto.center.as_ref().map(|c| c.lng))
            .bind(dto.center.as_ref().map(|c| c.lat))
            .bind(now)
            .bind(now)
            .bind(by.to_string())
            .bind(by.to_string())
            .bind(&plots_json)
            .bind(dto.row_config.as_ref().map(|c| serde_json::to_value(c).ok().flatten()))
            .bind(dto.bbch_stage.as_ref().map(|v| serde_json::to_value(v).ok().flatten()))
            .bind(dto.planted_date)
            .bind(dto.cleared_date)
            .bind(&dto.soil_type)
            .bind(dto.slope)
            .bind(&dto.slope_facing)
            .bind(dto.altitude)
            .bind(dto.organic)
            .bind(dto.boundary.as_ref().map(|_| serde_json::to_value(&dto.boundary).ok().flatten()))
            .bind(serde_json::to_value(&dto.sigpac_data).ok())
            .bind(serde_json::to_value(&dto.properties).ok())
            .bind(dto.custom_fields)
            .bind(&dto.note1)
            .bind(&dto.note2)
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))?;

            Ok(row_to_site(row, row.get("boundary_wkt"), row.get("center_wkt")))
        })
    }

    fn update(
        &self,
        tid: TenantId,
        id: Uuid,
        dto: UpdateSiteDto,
        by: Uuid,
    ) -> RepositoryFuture<Option<Site>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let now = Utc::now();
            let boundary_wkt = dto.boundary.as_ref().map(|b| boundary_to_wkt(b));

            let row = sqlx::query(
                r#"UPDATE sites SET 
                    label = COALESCE($1, label),
                    variety = COALESCE($2, variety),
                    area = COALESCE($3, area),
                    gross_area = COALESCE($4, gross_area),
                    is_active = COALESCE($5, is_active),
                    updated_at = $6,
                    updated_by = $7,
                    center = CASE WHEN $8 IS NOT NULL THEN ST_Point($8::double precision, $9::double precision) ELSE center END,
                    boundary = CASE WHEN $10 IS NOT NULL THEN ST_GeomFromText($10, 4326) ELSE boundary END
                   WHERE tenant_id = $11 AND id = $12
                   RETURNING id, tenant_id, business_id, label, site_type, crop_type, variety,
                             area, gross_area,
                             ST_AsText(boundary) as "boundary_wkt",
                             ST_AsText(center) as "center_wkt",
                             is_active, is_temporary, created_at, updated_at, created_by, updated_by,
                             plots, row_config, bbch_stage, planted_date, cleared_date,
                             soil_type, slope, slope_facing, altitude, organic, organic_eligible,
                             sigpac_data, properties, custom_fields, note1, note2"#
            )
            .bind(dto.label)
            .bind(dto.variety)
            .bind(dto.area)
            .bind(dto.gross_area)
            .bind(dto.is_active)
            .bind(now)
            .bind(by.to_string())
            .bind(dto.center.as_ref().map(|c| c.lng))
            .bind(dto.center.as_ref().map(|c| c.lat))
            .bind(boundary_wkt)
            .bind(tid.to_string())
            .bind(id.to_string())
            .fetch_one(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))?;

            Ok(Some(row_to_site(row, row.get("boundary_wkt"), row.get("center_wkt"))))
        })
    }

    fn delete(&self, tid: TenantId, id: Uuid) -> RepositoryFuture<bool> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let result = sqlx::query(
                "UPDATE sites SET is_active = false, updated_at = NOW() WHERE tenant_id = $1 AND id = $2"
            )
            .bind(tid.to_string())
            .bind(id.to_string())
            .execute(&pool)
            .await
            .map_err(|e| SharedError::Internal(e.to_string()))?;

            Ok(result.rows_affected() > 0)
        })
    }
}

/// Helper function to convert sqlx Row to Site entity
fn row_to_site(
    row: sqlx::postgres::PgRow, 
    boundary_wkt: Option<String>,
    center_wkt: Option<String>
) -> Site {
    let center = center_wkt.and_then(|wkt| {
        wkt.strip_prefix("POINT(").and_then(|s| {
            s.strip_suffix(")").map(|s| {
                let parts: Vec<&str> = s.split_whitespace().collect();
                GeoPoint {
                    lng: parts.get(0).and_then(|p| p.parse().ok()).unwrap_or(0.0),
                    lat: parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0.0),
                }
            })
        })
    });

    Site {
        id: row.get::<String, _>("id").parse().unwrap_or_default(),
        tenant_id: row.get::<String, _>("tenant_id").parse().unwrap_or_default(),
        business_id: row.get::<Option<String>, _>("business_id").and_then(|s| s.parse().ok()),
        label: row.get("label"),
        site_type: serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("site_type"))).unwrap_or(SiteType::Other("".into())),
        crop_type: serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("crop_type"))).unwrap_or(CropType::Other("".into())),
        variety: row.get("variety"),
        area: row.get("area"),
        gross_area: row.get("gross_area"),
        plots: row.get::<Option<serde_json::Value>, _>("plots")
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        row_config: row.get::<Option<serde_json::Value>, _>("row_config")
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        bbch_stage: row.get::<Option<serde_json::Value>, _>("bbch_stage")
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        planted_date: row.get("planted_date"),
        cleared_date: row.get("cleared_date"),
        soil_type: row.get("soil_type"),
        slope: row.get("slope"),
        slope_facing: row.get("slope_facing"),
        altitude: row.get("altitude"),
        organic: row.get("organic"),
        organic_eligible: row.get("organic_eligible"),
        center,
        sigpac_data: row.get::<Option<serde_json::Value>, _>("sigpac_data")
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        regepac_id: None,
        boundary: wkt_to_boundary(boundary_wkt),
        properties: row.get::<Option<serde_json::Value>, _>("properties")
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        custom_fields: row.get("custom_fields"),
        note1: row.get("note1"),
        note2: row.get("note2"),
        is_active: row.get("is_active"),
        is_temporary: row.get("is_temporary"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        created_by: row.get::<Option<String>, _>("created_by").and_then(|s| s.parse().ok()),
        updated_by: row.get::<Option<String>, _>("updated_by").and_then(|s| s.parse().ok()),
    }
}