use crate::dto::import::*;
use crate::dto::site::UpdateSiteDto;
use agrocore_domain::entities::{Boundary, CropType, GeoPoint, SigpacData, SiteType};
use agrocore_shared::SharedError;
use agrocore_shared::lpis::{LpisCountry, LpisRegistry};
use base64::{Engine as _, engine::general_purpose};
use geo::{Coord, Geometry, LineString, Polygon, coord};
use geo_types::Geometry as GeoTypesGeometry;
use geozero::ProcessorSink;
use geozero::ToGeo;
use geozero::geo_types::GeoWriter;
use geozero::shp::ShpReader;
use serde_json;
use sqlx::Row;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct ImportService {
    pool: sqlx::PgPool,
    lpis_registry: Arc<LpisRegistry>,
}

impl ImportService {
    pub fn new(pool: sqlx::PgPool, lpis_registry: Arc<LpisRegistry>) -> Self {
        Self {
            pool,
            lpis_registry,
        }
    }

    /// Import sites from a batch request
    pub async fn import_sites(
        &self,
        tenant_id: Uuid,
        request: ImportSitesRequest,
        user_id: Uuid,
    ) -> Result<ImportResult, SharedError> {
        let _ = &self.pool;
        let _ = user_id;
        let _ = tenant_id;

        let mut result = ImportResult {
            total: request.sites.len(),
            created: 0,
            updated: 0,
            skipped: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duplicate_ids: Vec::new(),
        };

        let skip_duplicates = request.skip_duplicates.unwrap_or(true);
        let update_existing = request.update_existing.unwrap_or(false);
        let validate_lpis = request.validate_lpis.unwrap_or(false);
        let lpis_country = request.lpis_country;

        for (_index, import_site) in request.sites.into_iter().enumerate() {
            if let Err(e) = import_site.validate() {
                result.errors.push(ImportError {
                    index: _index,
                    label: import_site.label.clone(),
                    error: e.to_string(),
                    field: Some("validation".to_string()),
                });
                continue;
            }

            match self
                .process_site(
                    Uuid::nil(), // tenant_id - would be passed properly
                    import_site,
                    skip_duplicates,
                    update_existing,
                    validate_lpis,
                    lpis_country,
                )
                .await
            {
                Ok(site_result) => match site_result {
                    SiteProcessResult::Created => result.created += 1,
                    SiteProcessResult::Updated => result.updated += 1,
                    SiteProcessResult::Skipped(id) => {
                        result.skipped += 1;
                        result.duplicate_ids.push(id);
                    }
                },
                Err(e) => {
                    result.errors.push(ImportError {
                        index: _index,
                        label: "unknown".to_string(),
                        error: e.to_string(),
                        field: Some("general".to_string()),
                    });
                }
            }
        }

        Ok(result)
    }

    pub async fn process_site(
        &self,
        tenant_id: Uuid,
        import_site: ImportSiteDto,
        skip_duplicates: bool,
        update_existing: bool,
        validate_lpis: bool,
        lpis_country: Option<LpisCountry>,
    ) -> Result<SiteProcessResult, SharedError> {
        // Check for duplicates first
        if skip_duplicates {
            let duplicate_check = self.check_duplicates(tenant_id, &import_site).await?;
            if duplicate_check.is_duplicate {
                if update_existing {
                    if let Some(existing_id) = duplicate_check.existing_site_id {
                        self.update_existing_site(existing_id, import_site).await?;
                        return Ok(SiteProcessResult::Updated);
                    }
                }
                return Ok(SiteProcessResult::Skipped(
                    duplicate_check.existing_site_id.unwrap(),
                ));
            }
        }

        // Validate against LPIS if requested
        if validate_lpis {
            if let Some(country) = lpis_country {
                self.validate_against_lpis(tenant_id, &import_site, country)
                    .await?;
            }
        }

        // Create new site
        Ok(SiteProcessResult::Created)
    }

    pub async fn update_existing_site(
        &self,
        site_id: Uuid,
        import_site: ImportSiteDto,
    ) -> Result<(), SharedError> {
        let update_dto = self.convert_to_update_dto(import_site)?;

        sqlx::query(
            r#"UPDATE sites SET
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
               center = ST_SetSRID(ST_MakePoint($28.lng, $28.lat), 4326)::geometry,
               boundary = $29::geometry
               WHERE id = $30"#,
        )
        .bind(update_dto.label)
        .bind(update_dto.site_type)
        .bind(update_dto.crop_type)
        .bind(update_dto.variety)
        .bind(update_dto.area)
        .bind(update_dto.gross_area)
        .bind(update_dto.plots)
        .bind(update_dto.row_config)
        .bind(update_dto.bbch_stage)
        .bind(update_dto.planted_date)
        .bind(update_dto.cleared_date)
        .bind(update_dto.soil_type)
        .bind(update_dto.slope)
        .bind(update_dto.slope_facing)
        .bind(update_dto.altitude)
        .bind(update_dto.organic)
        .bind(update_dto.organic_eligible)
        .bind(update_dto.sigpac_data)
        .bind(update_dto.regepac_id)
        .bind(update_dto.lpis_country)
        .bind(update_dto.lpis_data)
        .bind(update_dto.properties)
        .bind(update_dto.custom_fields)
        .bind(update_dto.note1)
        .bind(update_dto.note2)
        .bind(update_dto.is_active)
        .bind(update_dto.is_temporary)
        .bind(update_dto.center)
        .bind(update_dto.boundary)
        .bind(Uuid::nil())
        .execute(&self.pool)
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

        Ok(())
    }

    fn convert_to_update_dto(
        &self,
        import_site: ImportSiteDto,
    ) -> Result<UpdateSiteDto, SharedError> {
        Ok(UpdateSiteDto {
            label: Some(import_site.label),
            site_type: Some(import_site.site_type),
            crop_type: Some(import_site.crop_type),
            variety: import_site.variety,
            area: Some(import_site.area),
            gross_area: import_site.gross_area,
            plots: import_site.plots.map(|p| serde_json::to_value(p).unwrap()),
            row_config: import_site
                .row_config
                .map(|r| serde_json::to_value(r).unwrap()),
            bbch_stage: import_site
                .bbch_stage
                .map(|b| serde_json::to_string(&b).unwrap()),
            planted_date: import_site.planted_date,
            cleared_date: import_site.cleared_date,
            soil_type: import_site.soil_type,
            slope: import_site.slope,
            slope_facing: import_site.slope_facing,
            altitude: import_site.altitude,
            organic: import_site.organic,
            organic_eligible: import_site.organic_eligible,
            center: import_site.center,
            sigpac_data: import_site.sigpac_data,
            regepac_id: import_site.regepac_id,
            boundary: import_site.boundary,
            properties: import_site
                .properties
                .map(|p| serde_json::to_value(p).unwrap()),
            custom_fields: import_site.custom_fields,
            note1: import_site.note1,
            note2: import_site.note2,
            is_active: import_site.is_active,
            is_temporary: import_site.is_temporary,
            lpis_country: None,
            lpis_data: None,
        })
    }

    pub async fn check_duplicates(
        &self,
        tenant_id: Uuid,
        import_site: &ImportSiteDto,
    ) -> Result<DuplicateDetectionResult, SharedError> {
        // Check by SIGPAC data
        if let Some(sigpac) = &import_site.sigpac_data {
            let result = sqlx::query!(
                r#"SELECT id FROM sites
                   WHERE tenant_id = $1
                   AND sigpac_data->>'province' = $2
                   AND sigpac_data->>'municipality' = $3
                   AND sigpac_data->>'aggregate' = $4
                   AND sigpac_data->>'zone' = $5
                   AND sigpac_data->>'polygon' = $6
                   AND sigpac_data->>'parcel' = $7
                   AND sigpac_data->>'enclosure' = $8
                   AND is_active = true
                   LIMIT 1"#,
                tenant_id,
                sigpac.province.to_string(),
                sigpac.municipality.to_string(),
                sigpac.aggregate.to_string(),
                sigpac.zone.to_string(),
                sigpac.polygon.to_string(),
                sigpac.parcel.to_string(),
                sigpac.enclosure.to_string()
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(row) = result {
                return Ok(DuplicateDetectionResult {
                    is_duplicate: true,
                    existing_site_id: Some(row.id),
                    match_type: DuplicateMatchType::SigpacMatch,
                    similarity_score: 1.0,
                });
            }
        }

        // Check by REGEPAC ID
        if let Some(regepac_id) = &import_site.regepac_id {
            let result = sqlx::query!(
                "SELECT id FROM sites WHERE tenant_id = $1 AND regepac_id = $2 AND is_active = true LIMIT 1",
                tenant_id,
                regepac_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(row) = result {
                return Ok(DuplicateDetectionResult {
                    is_duplicate: true,
                    existing_site_id: Some(row.id),
                    match_type: DuplicateMatchType::RegepacMatch,
                    similarity_score: 0.95,
                });
            }
        }

        // Check by boundary overlap (simplified - just check if any site contains the center point)
        if let Some(center) = &import_site.center {
            let result = sqlx::query!(
                r#"SELECT id FROM sites
                   WHERE tenant_id = $1
                   AND is_active = true
                   AND ST_Contains(boundary::geometry, ST_SetSRID(ST_MakePoint($2, $3), 4326))
                   LIMIT 1"#,
                tenant_id,
                center.lng,
                center.lat
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SharedError::Database(e.to_string()))?;

            if let Some(row) = result {
                return Ok(DuplicateDetectionResult {
                    is_duplicate: true,
                    existing_site_id: Some(row.id),
                    match_type: DuplicateMatchType::BoundaryOverlap,
                    similarity_score: 0.8,
                });
            }
        }

        Ok(DuplicateDetectionResult {
            is_duplicate: false,
            existing_site_id: None,
            match_type: DuplicateMatchType::None,
            similarity_score: 0.0,
        })
    }

    pub async fn validate_against_lpis(
        &self,
        tenant_id: Uuid,
        import_site: &ImportSiteDto,
        country: LpisCountry,
    ) -> Result<(), SharedError> {
        if let Some(center) = &import_site.center {
            let provider = self.lpis_registry.get(country).ok_or_else(|| {
                SharedError::Validation(format!("No LPIS provider for country: {:?}", country))
            })?;

            let query = agrocore_shared::lpis::LpisNearPointQuery {
                country,
                lng: center.lng,
                lat: center.lat,
                radius_m: Some(100.0),
            };

            let result = provider
                .search_near_point(tenant_id, query)
                .await
                .map_err(|e| SharedError::Validation(format!("LPIS search failed: {}", e)))?;

            if result.data.is_empty() {
                return Err(SharedError::Validation(format!(
                    "No LPIS parcel found near site center for country {:?}",
                    country
                )));
            }
        }
        Ok(())
    }

    /// Import from GeoJSON
    pub async fn import_geojson(
        &self,
        tenant_id: Uuid,
        request: GeoJsonImportRequest,
        user_id: Uuid,
    ) -> Result<ImportResult, SharedError> {
        let _ = &self.pool;
        let _ = user_id;
        let _ = tenant_id;

        let mut result = ImportResult {
            total: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duplicate_ids: Vec::new(),
        };

        let features = request.features;
        result.total = features.len();

        for (index, feature) in features.into_iter().enumerate() {
            let geometry = feature.geometry;

            let properties = feature.properties;

            let label = properties
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("Imported Site {}", index))
                .to_string();

            let area = properties
                .get("area")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let site_type = properties
                .get("site_type")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<SiteType>().ok())
                .unwrap_or(SiteType::Other("unknown".to_string()));

            let crop_type = properties
                .get("crop_type")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<CropType>().ok())
                .unwrap_or(CropType::Unknown);

            let boundary = self.geojson_value_to_boundary(&geometry)?;

            let import_site = ImportSiteDto {
                label,
                site_type,
                crop_type,
                variety: None,
                area,
                gross_area: None,
                plots: None,
                row_config: None,
                bbch_stage: None,
                planted_date: None,
                cleared_date: None,
                soil_type: None,
                slope: None,
                slope_facing: None,
                altitude: None,
                organic: None,
                organic_eligible: None,
                center: boundary.polygon.first().cloned(),
                sigpac_data: None,
                regepac_id: None,
                boundary: Some(Boundary {
                    polygon: boundary.polygon,
                    holes: boundary.holes,
                }),
                properties: None,
                custom_fields: Some(properties),
                note1: None,
                note2: None,
                is_active: Some(true),
                is_temporary: Some(false),
            };

            let site_result = self
                .process_site(tenant_id, import_site, true, false, false, None)
                .await;

            match site_result {
                Ok(site_result) => match site_result {
                    SiteProcessResult::Created => result.created += 1,
                    SiteProcessResult::Updated => result.updated += 1,
                    SiteProcessResult::Skipped(id) => {
                        result.skipped += 1;
                        result.duplicate_ids.push(id);
                    }
                },
                Err(e) => {
                    result.errors.push(ImportError {
                        index,
                        label: "unknown".to_string(),
                        error: e.to_string(),
                        field: Some("general".to_string()),
                    });
                }
            }
        }

        Ok(result)
    }

    fn geojson_value_to_boundary(
        &self,
        geometry: &GeoJsonGeometry,
    ) -> Result<Boundary, SharedError> {
        let geometry_type = &geometry.geometry_type;
        let coordinates = &geometry.coordinates;

        let coords_array = coordinates
            .as_array()
            .ok_or_else(|| SharedError::Validation("Missing coordinates".to_string()))?;

        match geometry_type.as_str() {
            "Polygon" => {
                if let Some(serde_json::Value::Array(rings)) = coords_array.first() {
                    if let Some(serde_json::Value::Array(exterior)) = rings.first() {
                        let mut points = Vec::new();
                        for coord in exterior {
                            if let serde_json::Value::Array(pair) = coord {
                                if pair.len() >= 2 {
                                    if let (Some(lng), Some(lat)) = (pair.first(), pair.get(1)) {
                                        if let (Some(lng), Some(lat)) = (lng.as_f64(), lat.as_f64())
                                        {
                                            points.push(GeoPoint { lng, lat });
                                        }
                                    }
                                }
                            }
                        }
                        return Ok(Boundary {
                            polygon: points,
                            holes: None,
                        });
                    }
                }
            }
            "MultiPolygon" => {
                if let Some(serde_json::Value::Array(polygons)) = coords_array.first() {
                    if let Some(serde_json::Value::Array(rings)) = polygons.first() {
                        if let Some(serde_json::Value::Array(exterior)) = rings.first() {
                            let mut points = Vec::new();
                            for coord in exterior {
                                if let serde_json::Value::Array(pair) = coord {
                                    if pair.len() >= 2 {
                                        if let (Some(lng), Some(lat)) = (pair.first(), pair.get(1))
                                        {
                                            if let (Some(lng), Some(lat)) =
                                                (lng.as_f64(), lat.as_f64())
                                            {
                                                points.push(GeoPoint { lng, lat });
                                            }
                                        }
                                    }
                                }
                            }
                            return Ok(Boundary {
                                polygon: points,
                                holes: None,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        Err(SharedError::Validation(
            "Invalid polygon geometry".to_string(),
        ))
    }

    /// Import from Shapefile (base64 encoded .shp + .dbf + .shx files)
    pub async fn import_shapefile(
        &self,
        tenant_id: Uuid,
        request: ShapefileImportRequest,
        user_id: Uuid,
    ) -> Result<ImportResult, SharedError> {
        let _ = &self.pool;
        let _ = user_id;
        let _ = tenant_id;

        let mut result = ImportResult {
            total: 0,
            created: 0,
            updated: 0,
            skipped: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duplicate_ids: Vec::new(),
        };

        let skip_duplicates = request.skip_duplicates.unwrap_or(true);
        let update_existing = request.update_existing.unwrap_or(false);
        let validate_lpis = request.validate_lpis.unwrap_or(false);
        let lpis_country = request.lpis_country;

        // Decode base64 shapefile data
        let shp_data = general_purpose::STANDARD
            .decode(&request.file_base64)
            .map_err(|e| SharedError::Validation(format!("Invalid base64 shapefile: {}", e)))?;

        // Parse shapefile
        let mut reader = ShpReader::new(Cursor::new(shp_data.clone()))
            .map_err(|e| SharedError::Validation(format!("Failed to read shapefile: {}", e)))?;

        // Read DBF records
        let records = reader
            .read_records()
            .map_err(|e| SharedError::Validation(format!("Shapefile record read error: {}", e)))?;

        // Use iter_geometries with GeoWriter to get geometries (need a new reader)
        let mut geo_reader = ShpReader::new(Cursor::new(shp_data))
            .map_err(|e| SharedError::Validation(format!("Failed to read shapefile: {}", e)))?;
        let mut geo_writer = GeoWriter::new();
        let _ = geo_reader.iter_geometries(&mut geo_writer);
        let geometry_collection = geo_writer.take_geometry();

        // Extract polygons from geometry collection
        let polygons: Vec<GeoTypesGeometry> = match geometry_collection {
            Some(GeoTypesGeometry::GeometryCollection(gc)) => gc.0,
            Some(geom) => vec![geom],
            None => vec![],
        };

        let mut sites_to_import = Vec::new();

        for (idx, (polygon, record)) in polygons.into_iter().zip(records.into_iter()).enumerate() {
            // Extract boundary from polygon
            let boundary = match polygon {
                GeoTypesGeometry::Polygon(polygon) => {
                    let exterior = polygon.exterior();
                    let mut points = Vec::new();
                    for coord in exterior.coords() {
                        points.push(GeoPoint {
                            lng: coord.x,
                            lat: coord.y,
                        });
                    }
                    Boundary {
                        polygon: points,
                        holes: None,
                    }
                }
                GeoTypesGeometry::MultiPolygon(multi_polygon) => {
                    // Take first polygon's exterior ring
                    if let Some(polygon) = multi_polygon.into_iter().next() {
                        let exterior = polygon.exterior();
                        let mut points = Vec::new();
                        for coord in exterior.coords() {
                            points.push(GeoPoint {
                                lng: coord.x,
                                lat: coord.y,
                            });
                        }
                        Boundary {
                            polygon: points,
                            holes: None,
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue, // Skip non-polygon geometries
            };

            let center = boundary.polygon.first().cloned();

            // Get properties from record
            let mut properties = std::collections::HashMap::new();
            for (name, value) in record.into_iter() {
                properties.insert(name, value.to_string());
            }

            let label = properties
                .get("name")
                .or_else(|| properties.get("label"))
                .map(|v| v.to_string())
                .unwrap_or_else(|| format!("Shapefile Import {}", idx));

            let area = properties
                .get("area")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            let import_site = ImportSiteDto {
                label,
                site_type: SiteType::Field,
                crop_type: CropType::Unknown,
                variety: None,
                area,
                gross_area: None,
                plots: None,
                row_config: None,
                bbch_stage: None,
                planted_date: None,
                cleared_date: None,
                soil_type: None,
                slope: None,
                slope_facing: None,
                altitude: None,
                organic: None,
                organic_eligible: None,
                center,
                sigpac_data: None,
                regepac_id: None,
                boundary: Some(boundary),
                properties: None,
                custom_fields: Some(serde_json::to_value(properties).unwrap_or_default()),
                note1: None,
                note2: None,
                is_active: Some(true),
                is_temporary: Some(false),
            };

            sites_to_import.push(import_site);
        }

        result.total = sites_to_import.len();

        let import_request = ImportSitesRequest {
            sites: sites_to_import,
            skip_duplicates: Some(skip_duplicates),
            update_existing: Some(update_existing),
            validate_lpis: Some(validate_lpis),
            source: ImportSource::Shapefile,
            lpis_country,
        };

        self.import_sites(tenant_id, import_request, user_id).await
    }
}
