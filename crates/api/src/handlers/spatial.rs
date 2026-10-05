//! Spatial object reads for the map.
//!
//! The map needs three shapes of question and they are genuinely different queries:
//!
//!   * what is in this viewport — a bounding box, answered from the geometry
//!   * what is on this plot — a `site_id`
//!   * what of this kind is anywhere — an `object_type`
//!
//! `find_all` pages the whole tenant, which is the wrong query for all three: a farm with
//! 40,000 trees asking for "the olives on plot 3" would otherwise read every row in the
//! tenant and discard most of it.
//!
//! All three are served by one endpoint with optional filters rather than three endpoints.
//! The alternatives are three URLs that mean one thing, and a caller who has a plot and a
//! viewport has to know which to use.

use crate::AppState;
use crate::dto::spatial::{FeatureCollection, FeatureCollectionResponse};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::repositories::{PlantedAtFilter, SpatialObjectFilter};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

/// Default cap on a viewport response.
///
/// Chosen so a dense olive grove renders completely at a reasonable zoom, while a
/// tenant-wide query cannot pull the whole estate in one request. Overridable per request,
/// because the right number depends on the viewport and the client's own budget.
const DEFAULT_LIMIT: i64 = 5_000;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SpatialObjectQuery {
    /// Plot (site) the objects belong to.
    pub site_id: Option<Uuid>,
    /// Stored object type, e.g. `olive_tree`, `vine`, `barn`, `tree`.
    pub object_type: Option<String>,
    pub parent_id: Option<Uuid>,
    /// Viewport as `min_lng,min_lat,max_lng,max_lat` in WGS84. A single comma-separated
    /// parameter rather than four, so a map can build the URL without knowing which order
    /// we expect.
    pub bbox: Option<String>,
    /// `before:YYYY-MM-DD` or `after:YYYY-MM-DD` — objects planted before or after a
    /// date. Useful for "which trees were already there when we planted this block".
    pub planted_at: Option<String>,
    /// Include objects retired or deactivated. Off by default: last season's block should
    /// not be drawn by accident.
    pub include_inactive: Option<bool>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/spatial/objects",
    params(SpatialObjectQuery),
    responses(
        (status = 200, description = "Objects as a GeoJSON FeatureCollection", body = FeatureCollectionResponse),
        (status = 400, description = "Invalid filter"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_spatial_objects(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<SpatialObjectQuery>,
) -> Result<HttpResponse, ApiError> {
    let bbox = match &query.bbox {
        Some(raw) => Some(parse_bbox(raw)?),
        None => None,
    };
    let planted_at = match &query.planted_at {
        Some(raw) => Some(parse_planted_at(raw)?),
        None => None,
    };

    // A caller-supplied limit of zero or a negative number would return nothing and look
    // like an empty plot. Clamped rather than rejected: a map sending a bad number should
    // get data, not an error it cannot handle.
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 50_000);

    let filter = SpatialObjectFilter {
        site_id: query.site_id,
        object_type: query.object_type.clone(),
        parent_id: query.parent_id,
        bbox,
        planted_at,
        include_inactive: query.include_inactive.unwrap_or(false),
        limit: Some(limit),
    };

    let objects = state
        .db
        .spatial_object_repo()
        .find_by_filter(agrocore_domain::TenantId(auth.0.tenant_id), filter)
        .await?;

    // Ask for one more than the limit: if it comes back, the limit cut something off, and
    // saying so is better than a map that silently renders an incomplete grove.
    let truncated = objects.len() as i64 > limit;
    let mut objects = objects;
    if truncated {
        objects.truncate(limit as usize);
    }

    let response = FeatureCollectionResponse::new(FeatureCollection::from(objects), truncated);
    Ok(HttpResponse::Ok().json(response))
}

/// Parse `min_lng,min_lat,max_lng,max_lat`.
///
/// PostGIS order, longitude first. Validated against the coordinate ranges rather than
/// passed through: a bbox with latitude 140 silently returns nothing and the map looks
/// empty, while a 400 says the client sent nonsense.
fn parse_bbox(raw: &str) -> Result<(f64, f64, f64, f64), ApiError> {
    let parts: Vec<&str> = raw.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(ApiError::validation(format!(
            "bbox must be four comma-separated numbers, min_lng,min_lat,max_lng,max_lat; got '{raw}'"
        )));
    }
    let mut values = [0.0_f64; 4];
    for (i, part) in parts.iter().enumerate() {
        values[i] = part.parse::<f64>().map_err(|_| {
            ApiError::validation(format!("bbox component '{part}' is not a number"))
        })?;
    }
    let (min_lng, min_lat, max_lng, max_lat) = (values[0], values[1], values[2], values[3]);
    if !(-180.0..=180.0).contains(&min_lng)
        || !(-180.0..=180.0).contains(&max_lng)
        || !(-90.0..=90.0).contains(&min_lat)
        || !(-90.0..=90.0).contains(&max_lat)
    {
        return Err(ApiError::validation(
            "bbox is outside the WGS84 range: longitude -180..180, latitude -90..90".to_string(),
        ));
    }
    if min_lng > max_lng || min_lat > max_lat {
        return Err(ApiError::validation(
            "bbox minimum must not exceed its maximum".to_string(),
        ));
    }
    Ok((min_lng, min_lat, max_lng, max_lat))
}

/// Parse `before:YYYY-MM-DD`, `after:YYYY-MM-DD`, `set` or `null`.
fn parse_planted_at(raw: &str) -> Result<PlantedAtFilter, ApiError> {
    let (prefix, rest) = raw.split_once(':').ok_or_else(|| {
        ApiError::validation(
            "planted_at must be before:YYYY-MM-DD, after:YYYY-MM-DD, set or null".to_string(),
        )
    })?;
    let date = |s: &str| {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| ApiError::validation(format!("'{s}' is not a date in YYYY-MM-DD form")))
    };
    match prefix.trim() {
        "before" => Ok(PlantedAtFilter::Before(date(rest.trim())?)),
        "after" => Ok(PlantedAtFilter::After(date(rest.trim())?)),
        "set" | "notnull" | "not_null" => Ok(PlantedAtFilter::IsSet),
        "null" | "none" => Ok(PlantedAtFilter::IsNull),
        other => Err(ApiError::validation(format!(
            "unknown planted_at prefix '{other}'; use before, after, set or null"
        ))),
    }
}

/// A single object, for click-to-inspect.
#[utoipa::path(
    get,
    path = "/spatial/objects/{id}",
    responses(
        (status = 200, description = "One object as a GeoJSON Feature", body = crate::dto::spatial::Feature),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_spatial_object(
    state: web::Data<AppState>,
    auth: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let obj = state
        .db
        .spatial_object_repo()
        .find_by_id(agrocore_domain::TenantId(auth.0.tenant_id), path.into_inner())
        .await?;
    match obj {
        Some(obj) => {
            let feature: crate::dto::spatial::Feature = (&obj).into();
            Ok(HttpResponse::Ok().json(feature))
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}