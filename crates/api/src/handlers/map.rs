use crate::AppState;
use crate::dto::spatial::{MapSiteData, MapTaskData};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::TenantId;
use agrocore_domain::entities::spatial::types::boundary_to_geojson;
use agrocore_domain::repositories::SpatialObjectFilter;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MapSitesQuery {
    pub site_id: Option<Uuid>,
    pub object_type: Option<String>,
    pub bbox: Option<String>,
    pub limit: Option<i64>,
}

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

#[utoipa::path(
    get,
    path = "/api/v1/map/sites",
    params(MapSitesQuery),
    responses(
        (status = 200, description = "Sites as GeoJSON features for map", body = Vec<MapSiteData>),
        (status = 400, description = "Invalid filter"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "map"
)]
pub async fn list_map_sites(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<MapSitesQuery>,
) -> Result<HttpResponse, ApiError> {
    let bbox = match &query.bbox {
        Some(raw) => Some(parse_bbox(raw)?),
        None => None,
    };
    let limit = query.limit.unwrap_or(5_000).clamp(1, 50_000);

    let filter = SpatialObjectFilter {
        site_id: query.site_id,
        object_type: query.object_type.clone(),
        parent_id: None,
        bbox,
        planted_at: None,
        include_inactive: false,
        limit: Some(limit),
    };

    let objects = state
        .db
        .spatial_object_repo()
        .find_by_filter(TenantId(auth.0.tenant_id), filter)
        .await?;

    let sites: Vec<MapSiteData> = objects
        .into_iter()
        .map(|obj| MapSiteData {
            id: obj.id,
            label: obj.label,
            geometry: obj.geometry.to_geojson(),
            color: None,
        })
        .collect();

    Ok(HttpResponse::Ok().json(sites))
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MapTasksQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_m: Option<f64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/map/tasks",
    params(MapTasksQuery),
    responses(
        (status = 200, description = "Tasks as GeoJSON features for map", body = Vec<MapTaskData>),
        (status = 400, description = "Invalid filter"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = [])),
    tag = "map"
)]
pub async fn list_map_tasks(
    state: web::Data<AppState>,
    auth: AuthUser,
    query: web::Query<MapTasksQuery>,
) -> Result<HttpResponse, ApiError> {
    let limit = query.limit.unwrap_or(5_000).clamp(1, 50_000);
    let radius = query.radius_m.unwrap_or(50_000.0).clamp(1.0, 100_000.0);
    let lat = query.lat.unwrap_or(38.7223); // Default to Lisbon
    let lng = query.lng.unwrap_or(-9.1393);

    let tid = TenantId(auth.0.tenant_id);

    // Get nearby sub-tasks with site boundaries
    let nearby = state
        .db
        .task_sub_task_repo()
        .find_nearby(tid, lat, lng, radius, limit)
        .await?;

    let mut tasks = Vec::new();
    for st in nearby {
        let Some(site_id) = st.site_id else { continue };
        let Ok(Some(site)) = state.db.site_repo().find_by_id(tid, site_id).await else {
            continue;
        };
        let Some(boundary) = site.boundary else {
            continue;
        };
        let geojson_str = boundary_to_geojson(&boundary);
        let geometry: serde_json::Value =
            serde_json::from_str(&geojson_str).unwrap_or(serde_json::json!({}));
        tasks.push(MapTaskData {
            id: st.sub_task_id,
            label: st.label,
            status: st.status,
            geometry,
            site_id: Some(site_id),
        });
    }

    Ok(HttpResponse::Ok().json(tasks))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/map")
            .service(web::resource("/sites").route(web::get().to(list_map_sites)))
            .service(web::resource("/tasks").route(web::get().to(list_map_tasks))),
    );
}
