//! SIGPAC Parcel API Handlers
//!
//! Provides endpoints for querying official Spanish SIGPAC parcel reference data
//! imported from fiboa GeoParquet files on source.coop.

use crate::AppState;
use crate::dto::ErrorResponse;
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_shared::SharedError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SigpacParcelQuery {
    pub province: Option<u8>,
    pub municipality: Option<u16>,
    pub aggregate: Option<u16>,
    pub zone: Option<u16>,
    pub polygon: Option<u16>,
    pub parcel: Option<u16>,
    pub enclosure: Option<u16>,
    pub sigpac_reference: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SigpacParcelDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sigpac_reference: String,
    pub province: i16,
    pub municipality: i16,
    pub aggregate: i16,
    pub zone: i16,
    pub polygon: i16,
    pub parcel: i16,
    pub enclosure: i16,
    pub usage_code: Option<String>,
    pub usage_description: Option<String>,
    pub geometry: serde_json::Value,
    pub area_hectares: Option<f64>,
    pub official_area_ha: Option<f64>,
    pub source_dataset: Option<String>,
    pub source_year: Option<i16>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedSigpacParcelResponse {
    pub data: Vec<SigpacParcelDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[utoipa::path(
    get,
    path = "/api/v1/sigpac/parcels",
    params(
        ("province" = Option<u8>, Query, description = "Province code (2 digits: 01-52)"),
        ("municipality" = Option<u16>, Query, description = "Municipality code (3 digits: 001-999)"),
        ("aggregate" = Option<u16>, Query, description = "Aggregate code (3 digits)"),
        ("zone" = Option<u16>, Query, description = "Zone code (3 digits)"),
        ("polygon" = Option<u16>, Query, description = "Polygon code (3 digits)"),
        ("parcel" = Option<u16>, Query, description = "Parcel code (5 digits)"),
        ("enclosure" = Option<u16>, Query, description = "Enclosure code (3 digits)"),
        ("sigpac_reference" = Option<String>, Query, description = "Full SIGPAC reference (20 digits)"),
        ("page" = Option<u64>, Query, description = "Page number (default: 1)"),
        ("per_page" = Option<u64>, Query, description = "Items per page (default: 50, max: 200)")
    ),
    responses(
        (status = 200, description = "List SIGPAC parcels", body = PaginatedSigpacParcelResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sigpac",
    security(("bearer_auth" = []))
)]
pub async fn list_sigpac_parcels(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<SigpacParcelQuery>,
) -> Result<HttpResponse, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    // Build WHERE clause dynamically
    let mut where_clauses = vec!["tenant_id = $1".to_string()];
    let mut param_idx = 2;

    if query.province.is_some() {
        where_clauses.push(format!("province = ${}", param_idx));
        param_idx += 1;
    }
    if query.municipality.is_some() {
        where_clauses.push(format!("municipality = ${}", param_idx));
        param_idx += 1;
    }
    if query.aggregate.is_some() {
        where_clauses.push(format!("aggregate = ${}", param_idx));
        param_idx += 1;
    }
    if query.zone.is_some() {
        where_clauses.push(format!("zone = ${}", param_idx));
        param_idx += 1;
    }
    if query.polygon.is_some() {
        where_clauses.push(format!("polygon = ${}", param_idx));
        param_idx += 1;
    }
    if query.parcel.is_some() {
        where_clauses.push(format!("parcel = ${}", param_idx));
        param_idx += 1;
    }
    if query.enclosure.is_some() {
        where_clauses.push(format!("enclosure = ${}", param_idx));
        param_idx += 1;
    }
    if query.sigpac_reference.is_some() {
        where_clauses.push(format!("sigpac_reference = ${}", param_idx));
        param_idx += 1;
    }

    let where_clause = where_clauses.join(" AND ");

    // Count total using separate query per condition to avoid dynamic binding issues
    let count_sql = format!("SELECT COUNT(*) FROM sigpac_parcels WHERE {}", where_clause);

    // Build count query with proper parameter binding
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    count_query = count_query.bind(auth.0.tenant_id);

    if let Some(v) = query.province {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.municipality {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.aggregate {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.zone {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.polygon {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.parcel {
        count_query = count_query.bind(v as i16);
    }
    if let Some(v) = query.enclosure {
        count_query = count_query.bind(v as i16);
    }
    if let Some(ref v) = query.sigpac_reference {
        count_query = count_query.bind(v.clone());
    }

    let total: i64 = count_query.fetch_one(state.db.pool()).await?;

    // Fetch page
    let select_sql = format!(
        "SELECT id, tenant_id, sigpac_reference, province, municipality, aggregate, zone, polygon, parcel, enclosure, 
         usage_code, usage_description, ST_AsGeoJSON(geometry) as geometry, area_hectares, official_area_ha, 
         source_dataset, source_year, created_at, updated_at 
         FROM sigpac_parcels 
         WHERE {} 
         ORDER BY sigpac_reference 
         LIMIT ${} OFFSET ${}",
        where_clause, param_idx, param_idx + 1
    );

    let mut select_query = sqlx::query(&select_sql);
    select_query = select_query.bind(auth.0.tenant_id);

    if let Some(v) = query.province {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.municipality {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.aggregate {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.zone {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.polygon {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.parcel {
        select_query = select_query.bind(v as i16);
    }
    if let Some(v) = query.enclosure {
        select_query = select_query.bind(v as i16);
    }
    if let Some(ref v) = query.sigpac_reference {
        select_query = select_query.bind(v.clone());
    }

    select_query = select_query.bind(per_page as i64).bind(offset as i64);

    let rows = select_query.fetch_all(state.db.pool()).await?;

    let mut parcels = Vec::new();
    for row in rows {
        let geometry_json: String = row.try_get("geometry")?;
        let geometry: serde_json::Value =
            serde_json::from_str(&geometry_json).unwrap_or(serde_json::Value::Null);

        parcels.push(SigpacParcelDto {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            sigpac_reference: row.try_get("sigpac_reference")?,
            province: row.try_get::<i16, _>("province")?,
            municipality: row.try_get::<i16, _>("municipality")?,
            aggregate: row.try_get::<i16, _>("aggregate")?,
            zone: row.try_get::<i16, _>("zone")?,
            polygon: row.try_get::<i16, _>("polygon")?,
            parcel: row.try_get::<i16, _>("parcel")?,
            enclosure: row.try_get::<i16, _>("enclosure")?,
            usage_code: row.try_get("usage_code")?,
            usage_description: row.try_get("usage_description")?,
            geometry,
            area_hectares: row.try_get("area_hectares")?,
            official_area_ha: row.try_get("official_area_ha")?,
            source_dataset: row.try_get("source_dataset")?,
            source_year: row.try_get::<Option<i16>, _>("source_year")?,
            created_at: row
                .try_get::<DateTime<chrono::Utc>, _>("created_at")?
                .to_rfc3339(),
            updated_at: row
                .try_get::<DateTime<chrono::Utc>, _>("updated_at")?
                .to_rfc3339(),
        });
    }

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u64;

    Ok(HttpResponse::Ok().json(PaginatedSigpacParcelResponse {
        data: parcels,
        total: total as u64,
        page,
        per_page,
        total_pages,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/sigpac/parcels/{id}",
    responses(
        (status = 200, description = "SIGPAC parcel details", body = SigpacParcelDto),
        (status = 404, description = "Parcel not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sigpac",
    security(("bearer_auth" = []))
)]
pub async fn get_sigpac_parcel(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let parcel_id = *path;

    let row = sqlx::query(
        "SELECT id, tenant_id, sigpac_reference, province, municipality, aggregate, zone, polygon, parcel, enclosure, 
         usage_code, usage_description, ST_AsGeoJSON(geometry) as geometry, area_hectares, official_area_ha, 
         source_dataset, source_year, created_at, updated_at 
         FROM sigpac_parcels 
         WHERE id = $1 AND tenant_id = $2"
    )
    .bind(parcel_id)
    .bind(auth.0.tenant_id)
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| SharedError::NotFound("SIGPAC parcel not found".into()))?;

    let geometry_json: String = row.try_get("geometry")?;
    let geometry: serde_json::Value =
        serde_json::from_str(&geometry_json).unwrap_or(serde_json::Value::Null);

    Ok(HttpResponse::Ok().json(SigpacParcelDto {
        id: row.try_get("id")?,
        tenant_id: row.try_get("tenant_id")?,
        sigpac_reference: row.try_get("sigpac_reference")?,
        province: row.try_get("province")?,
        municipality: row.try_get("municipality")?,
        aggregate: row.try_get("aggregate")?,
        zone: row.try_get("zone")?,
        polygon: row.try_get("polygon")?,
        parcel: row.try_get("parcel")?,
        enclosure: row.try_get("enclosure")?,
        usage_code: row.try_get("usage_code")?,
        usage_description: row.try_get("usage_description")?,
        geometry,
        area_hectares: row.try_get("area_hectares")?,
        official_area_ha: row.try_get("official_area_ha")?,
        source_dataset: row.try_get("source_dataset")?,
        source_year: row.try_get("source_year")?,
        created_at: row
            .try_get::<DateTime<chrono::Utc>, _>("created_at")?
            .to_rfc3339(),
        updated_at: row
            .try_get::<DateTime<chrono::Utc>, _>("updated_at")?
            .to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NearPointQuery {
    pub lng: f64,
    pub lat: f64,
    pub radius_m: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/sigpac/parcels/search/near-point",
    params(
        ("lng" = f64, Query, description = "Longitude"),
        ("lat" = f64, Query, description = "Latitude"),
        ("radius_m" = Option<f64>, Query, description = "Search radius in meters (default 1000, max 10000)")
    ),
    responses(
        (status = 200, description = "Parcels near point", body = PaginatedSigpacParcelResponse),
        (status = 401, description = "Unauthorized")
    ),
    tag = "sigpac",
    security(("bearer_auth" = []))
)]
pub async fn search_parcels_near_point(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<NearPointQuery>,
) -> Result<HttpResponse, ApiError> {
    let radius = query.radius_m.unwrap_or(1000.0).min(10000.0);

    let rows = sqlx::query(
        "SELECT id, tenant_id, sigpac_reference, province, municipality, aggregate, zone, polygon, parcel, enclosure, 
         usage_code, usage_description, ST_AsGeoJSON(geometry) as geometry, area_hectares, official_area_ha, 
         source_dataset, source_year, created_at, updated_at
         FROM sigpac_parcels
         WHERE tenant_id = $1
         AND ST_DWithin(geography(geometry), geography(ST_SetSRID(ST_MakePoint($2, $3), 4326)), $4)
         ORDER BY ST_Distance(geography(geometry), geography(ST_SetSRID(ST_MakePoint($2, $3), 4326)))
         LIMIT 100"
    )
    .bind(auth.0.tenant_id)
    .bind(query.lng)
    .bind(query.lat)
    .bind(radius)
    .fetch_all(state.db.pool())
    .await?;

    let mut parcels = Vec::new();
    for row in rows {
        let geometry_json: String = row.try_get("geometry")?;
        let geometry: serde_json::Value =
            serde_json::from_str(&geometry_json).unwrap_or(serde_json::Value::Null);

        parcels.push(SigpacParcelDto {
            id: row.try_get("id")?,
            tenant_id: row.try_get("tenant_id")?,
            sigpac_reference: row.try_get("sigpac_reference")?,
            province: row.try_get::<i16, _>("province")?,
            municipality: row.try_get::<i16, _>("municipality")?,
            aggregate: row.try_get::<i16, _>("aggregate")?,
            zone: row.try_get::<i16, _>("zone")?,
            polygon: row.try_get::<i16, _>("polygon")?,
            parcel: row.try_get::<i16, _>("parcel")?,
            enclosure: row.try_get::<i16, _>("enclosure")?,
            usage_code: row.try_get("usage_code")?,
            usage_description: row.try_get("usage_description")?,
            geometry,
            area_hectares: row.try_get("area_hectares")?,
            official_area_ha: row.try_get("official_area_ha")?,
            source_dataset: row.try_get("source_dataset")?,
            source_year: row.try_get::<Option<i16>, _>("source_year")?,
            created_at: row
                .try_get::<DateTime<chrono::Utc>, _>("created_at")?
                .to_rfc3339(),
            updated_at: row
                .try_get::<DateTime<chrono::Utc>, _>("updated_at")?
                .to_rfc3339(),
        });
    }

    Ok(HttpResponse::Ok().json(PaginatedSigpacParcelResponse {
        data: parcels,
        total: 0,
        page: 1,
        per_page: 100,
        total_pages: 1,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sigpac")
            .route("/parcels", web::get().to(list_sigpac_parcels))
            .route("/parcels/{id}", web::get().to(get_sigpac_parcel))
            .route(
                "/parcels/search/near-point",
                web::get().to(search_parcels_near_point),
            ),
    );
}
