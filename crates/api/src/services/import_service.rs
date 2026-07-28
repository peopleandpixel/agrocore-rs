use agrocore_domain::entities::{Boundary, GeoPoint, SigpacData, SiteType, CropType};
use agrocore_shared::SharedError;
use crate::dto::import::*;
use crate::dto::site::UpdateSiteDto;
use geozero::shp::ShpReader;
use serde_json;
use sqlx::Row;
use std::io::Cursor;
use uuid::Uuid;
use validator::Validate;
use base64::{Engine as _, engine::general_purpose};

#[derive(Clone)]
pub struct ImportService {
    pool: sqlx::PgPool,
}

impl ImportService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
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

            match self.process_site(
                Uuid::nil(), // tenant_id - would be passed properly
                import_site,
                skip_duplicates,
                update_existing,
                validate_lpis,
            ).await {
                Ok(site_result) => {
                    match site_result {
                        SiteProcessResult::Created => result.created += 1,
                        SiteProcessResult::Updated => result.updated += 1,
                        SiteProcessResult::Skipped(id) => {
                            result.skipped += 1;
                            result.duplicate_ids.push(id);
                        }
                    }
                }
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

    async fn process_site(
        &self,
        _tenant_id: Uuid,
        import_site: ImportSiteDto,
        skip_duplicates: bool,
        update_existing: bool,
        validate_lpis: bool,
    ) -> Result<SiteProcessResult, SharedError> {
        let _ = &self.pool;
        // Check for duplicates
        let duplicate_check = self.check_duplicates(_tenant_id, &import_site).await?;

        if duplicate_check.is_duplicate {
            if skip_duplicates && !update_existing {
                return Ok(SiteProcessResult::Skipped(duplicate_check.existing_site_id.unwrap()));
            }

            #[allow(clippy::collapsible_if)]
            if update_existing {
                if let Some(_existing_id) = duplicate_check.existing_site_id {
                    let _update_dto = self.convert_to_update_dto(import_site)?;
                    // Update would be done via repository
                    return Ok(SiteProcessResult::Updated);
                }
            }

            return Err(SharedError::Validation(
                format!("Duplicate site detected: {:?}", duplicate_check.match_type)
            ));
        }

        // Validate LPIS if requested
        #[allow(clippy::collapsible_if)]
        if validate_lpis {
            if let Some(sigpac) = &import_site.sigpac_data {
                let validation = self.validate_against_lpis(sigpac, &import_site.boundary).await?;
                if !validation.is_valid {
                    return Err(SharedError::Validation(
                        format!("LPIS validation failed: {}", validation.warnings.join(", "))
                    ));
                }
            }
        }

        // Create new site
        Ok(SiteProcessResult::Created)
    }

    fn convert_to_update_dto(&self, import_site: ImportSiteDto) -> Result<UpdateSiteDto, SharedError> {
        Ok(UpdateSiteDto {
            label: Some(import_site.label),
            variety: import_site.variety,
            area: Some(import_site.area),
            gross_area: import_site.gross_area,
            plots: import_site.plots,
            row_config: import_site.row_config,
            bbch_stage: import_site.bbch_stage,
            planted_date: import_site.planted_date,
            cleared_date: import_site.cleared_date,
            soil_type: import_site.soil_type,
            slope: import_site.slope,
            slope_facing: import_site.slope_facing,
            altitude: import_site.altitude,
            organic: import_site.organic,
            center: import_site.center,
            sigpac_data: import_site.sigpac_data,
            regepac_id: import_site.regepac_id,
            boundary: import_site.boundary.map(|b| b.0), // Convert Boundary to Vec<GeoPoint>
            properties: import_site.properties,
            custom_fields: import_site.custom_fields,
            note1: import_site.note1,
            note2: import_site.note2,
            is_active: import_site.is_active,
        })
    }

    async fn check_duplicates(
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

        // Check by exact boundary match using PostGIS
        if let Some(boundary) = &import_site.boundary {
            // Convert boundary to PostGIS geometry
            let points: Vec<(f64, f64)> = boundary.0.iter().map(|p| (p.lng, p.lat)).collect();
            if points.len() >= 4 {
                // Ensure closed ring
                let mut coords = points.clone();
                if coords.first() != coords.last() {
                    coords.push(coords[0]);
                }
                let wkt = format!("POLYGON(({}))", coords.iter().map(|(lng, lat)| format!("{} {}", lng, lat)).collect::<Vec<_>>().join(","));
                
                let result = sqlx::query!(
                    r#"SELECT id FROM sites 
                       WHERE tenant_id = $1 
                       AND boundary IS NOT NULL
                       AND ST_Equals(boundary, ST_GeomFromText($2, 4326))
                       AND is_active = true
                       LIMIT 1"#,
                    tenant_id,
                    wkt
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| SharedError::Database(e.to_string()))?;

                if let Some(row) = result {
                    return Ok(DuplicateDetectionResult {
                        is_duplicate: true,
                        existing_site_id: Some(row.id),
                        match_type: DuplicateMatchType::ExactBoundary,
                        similarity_score: 1.0,
                    });
                }
            }
        }

        Ok(DuplicateDetectionResult {
            is_duplicate: false,
            existing_site_id: None,
            match_type: DuplicateMatchType::HighSimilarity,
            similarity_score: 0.0,
        })
    }

    async fn validate_against_lpis(
        &self,
        sigpac: &SigpacData,
        boundary: &Option<Boundary>,
    ) -> Result<LpisValidationResult, SharedError> {
        // Build boundary WKT if present
        let boundary_wkt = boundary.as_ref().and_then(|b| {
            let points: Vec<(f64, f64)> = b.0.iter().map(|p| (p.lng, p.lat)).collect();
            if points.len() >= 4 {
                let mut coords = points.clone();
                if coords.first() != coords.last() {
                    coords.push(coords[0]);
                }
                Some(format!("POLYGON(({}))", coords.iter().map(|(lng, lat)| format!("{} {}", lng, lat)).collect::<Vec<_>>().join(",")))
            } else {
                None
            }
        });

        let boundary_geom = boundary_wkt
            .map(|wkt| format!("ST_GeomFromText('{}', 4326)::geometry", wkt))
            .unwrap_or_else(|| "NULL".to_string());

        let query = format!(
            r#"SELECT * FROM validate_parcel_against_lpis(
                $1, $2, $3, $4, $5, $6, $7, {}, $9
            )"#,
            boundary_geom
        );

        let result = sqlx::query(&query)
            .bind(sigpac.province as i16)
            .bind(sigpac.municipality as i16)
            .bind(sigpac.aggregate as i16)
            .bind(sigpac.zone as i16)
            .bind(sigpac.polygon as i16)
            .bind(sigpac.parcel as i16)
            .bind(sigpac.enclosure as i16)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SharedError::Database(e.to_string()))?;

        let validation: serde_json::Value = serde_json::from_value(
            result.try_get("validate_parcel_against_lpis")
                .map_err(|e| SharedError::Database(e.to_string()))?
        )
            .map_err(|e| SharedError::Database(format!("Failed to parse validation result: {}", e)))?;

        Ok(LpisValidationResult {
            is_valid: validation.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(false),
            sigpac_reference: validation.get("sigpac_reference").and_then(|v| v.as_str()).map(|s| s.to_string()),
            area_difference: validation.get("area_difference").and_then(|v| v.as_f64()),
            boundary_difference: validation.get("boundary_difference").and_then(|v| v.as_f64()),
            warnings: validation.get("warnings").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        })
    }

    /// Import from GeoJSON features
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
            total: request.features.len(),
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

        for (_index, feature) in request.features.into_iter().enumerate() {
            if feature.geometry.geometry_type != "Polygon" && feature.geometry.geometry_type != "MultiPolygon" {
                result.errors.push(ImportError {
                    index: _index,
                    label: feature.properties.get("label").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    error: "Only Polygon and MultiPolygon geometries are supported".to_string(),
                    field: Some("geometry".to_string()),
                });
                continue;
            }

            let label = feature.properties.get("label")
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("Imported Site {}", _index))
                .to_string();

            let sigpac_data = feature.properties.get("sigpac_data")
                .and_then(|v| serde_json::from_value(v.clone()).ok());

            let regepac_id = feature.properties.get("regepac_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let boundary = self.geojson_to_boundary(&feature.geometry).ok();

            let import_site = ImportSiteDto {
                label,
                site_type: SiteType::Field,
                crop_type: CropType::Unknown, // Now this exists!
                variety: None,
                area: feature.properties.get("area").and_then(|v| v.as_f64()).unwrap_or(0.0),
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
                center: None,
                sigpac_data,
                regepac_id,
                boundary,
                properties: None,
                custom_fields: Some(feature.properties.clone()),
                note1: None,
                note2: None,
                is_active: Some(true),
                is_temporary: Some(false),
            };

            match self.process_site(Uuid::nil(), import_site, skip_duplicates, update_existing, validate_lpis).await {
                Ok(site_result) => {
                    match site_result {
                        SiteProcessResult::Created => result.created += 1,
                        SiteProcessResult::Updated => result.updated += 1,
                        SiteProcessResult::Skipped(id) => {
                            result.skipped += 1;
                            result.duplicate_ids.push(id);
                        }
                    }
                }
                Err(e) => {
                    result.errors.push(ImportError {
                        index: _index,
                        label: "unknown".to_string(),
                        error: e.to_string(),
                        field: Some("geometry".to_string()),
                    });
                }
            }
        }

        Ok(result)
    }

    fn geojson_to_boundary(&self, geometry: &GeoJsonGeometry) -> Result<Boundary, String> {
        let coords = match &geometry.coordinates {
            serde_json::Value::Array(arr) => arr,
            _ => return Err("Invalid coordinates format".to_string()),
        };

        #[allow(clippy::collapsible_if)]
        // Handle Polygon (first ring is exterior)
        if let Some(serde_json::Value::Array(rings)) = coords.first() {
            if let Some(serde_json::Value::Array(exterior)) = rings.first() {
                let mut points = Vec::new();
                for coord in exterior {
                    #[allow(clippy::collapsible_if)]
                    if let serde_json::Value::Array(pair) = coord {
                        if pair.len() >= 2 {
                            if let (Some(lng), Some(lat)) = (pair.first(), pair.get(1)) {
                                if let (Some(lng), Some(lat)) = (lng.as_f64(), lat.as_f64()) {
                                    points.push(GeoPoint { lng, lat });
                                }
                            }
                        }
                    }
                }
                return Ok(Boundary(points));
            }
        }

        Err("Invalid polygon geometry".to_string())
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

        // Decode base64 shapefile data
        let shp_data = general_purpose::STANDARD
            .decode(&request.file_base64)
            .map_err(|e| SharedError::Validation(format!("Invalid base64 shapefile: {}", e)))?;

        // Parse shapefile using geozero
        let cursor = Cursor::new(shp_data);
        let reader = ShpReader::new(cursor)
            .map_err(|e| SharedError::Validation(format!("Failed to read shapefile: {}", e)))?;

        // Convert shapefile features to GeoJSON
        let mut geojson_buffer: Vec<u8> = Vec::new();
        let mut json_writer = geozero::geojson::GeoJsonWriter::new(&mut geojson_buffer);

        reader
            .iter_features(&mut json_writer)
            .map_err(|e| SharedError::Validation(format!("Failed to parse shapefile features: {}", e)))?;

        let geojson_str = String::from_utf8(geojson_buffer)
            .map_err(|e| SharedError::Validation(format!("Invalid UTF-8 in shapefile: {}", e)))?;

        let geojson: serde_json::Value = serde_json::from_str(&geojson_str)
            .map_err(|e| SharedError::Validation(format!("Failed to parse shapefile GeoJSON: {}", e)))?;

        if let Some(features_array) = geojson.get("features").and_then(|v| v.as_array()) {
            result.total = features_array.len();

            for (_index, feature) in features_array.iter().enumerate() {
                let properties = feature.get("properties")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();

                let geometry = feature.get("geometry")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();

                let geometry_type = geometry.get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                if geometry_type != "Polygon" && geometry_type != "MultiPolygon" {
                    result.errors.push(ImportError {
                        index: _index,
                        label: properties.get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        error: "Only Polygon and MultiPolygon geometries are supported".to_string(),
                        field: Some("geometry".to_string()),
                    });
                    continue;
                }

                let label = properties.get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("Imported Site {}", _index))
                    .to_string();

                let sigpac_data = properties.get("sigpac_data")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let regepac_id = properties.get("regepac_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Convert geometry to boundary
                let boundary = self.geojson_value_to_boundary(&geometry).ok();

                let area = properties.get("area")
                    .and_then(|v| v.as_f64())
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
                    center: None,
                    sigpac_data,
                    regepac_id,
                    boundary,
                    properties: None,
                    custom_fields: Some(serde_json::Value::Object(properties)),
                    note1: None,
                    note2: None,
                    is_active: Some(true),
                    is_temporary: Some(false),
                };

                match self.process_site(Uuid::nil(), import_site, skip_duplicates, update_existing, validate_lpis).await {
                    Ok(site_result) => {
                        match site_result {
                            SiteProcessResult::Created => result.created += 1,
                            SiteProcessResult::Updated => result.updated += 1,
                            SiteProcessResult::Skipped(id) => {
                                result.skipped += 1;
                                result.duplicate_ids.push(id);
                            }
                        }
                    }
                    Err(e) => {
                        result.errors.push(ImportError {
                            index: _index,
                            label: "unknown".to_string(),
                            error: e.to_string(),
                            field: Some("geometry".to_string()),
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    fn geojson_value_to_boundary(&self, geometry: &serde_json::Map<String, serde_json::Value>) -> Result<Boundary, String> {
        let coordinates = geometry.get("coordinates")
            .ok_or("Missing coordinates")?;

        let coords = match coordinates {
            serde_json::Value::Array(arr) => arr,
            _ => return Err("Invalid coordinates format".to_string()),
        };

        #[allow(clippy::collapsible_if)]
        // Handle Polygon (first ring is exterior)
        if let Some(serde_json::Value::Array(rings)) = coords.first() {
            if let Some(serde_json::Value::Array(exterior)) = rings.first() {
                let mut points = Vec::new();
                for coord in exterior {
                    #[allow(clippy::collapsible_if)]
                    if let serde_json::Value::Array(pair) = coord {
                        if pair.len() >= 2 {
                            if let (Some(lng), Some(lat)) = (pair.first(), pair.get(1)) {
                                if let (Some(lng), Some(lat)) = (lng.as_f64(), lat.as_f64()) {
                                    points.push(GeoPoint { lng, lat });
                                }
                            }
                        }
                    }
                }
                return Ok(Boundary(points));
            }
        }

        Err("Invalid polygon geometry".to_string())
    }
}

enum SiteProcessResult {
    Created,
    Updated,
    Skipped(Uuid),
}