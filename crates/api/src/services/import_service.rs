use crate::AppState;
use crate::dto::{
    CreateSiteDto, ErrorResponse, ImportError, ImportResult, ImportSitesRequest, ImportSiteDto,
    ImportSource, ImportWarning, PaginatedResponseDto, PaginatedSiteResponse, SiteDto,
    UpdateSiteDto,
};
use crate::error::ApiError;
use crate::middleware::AuthExtractor as AuthUser;
use actix_web::{HttpResponse, web};
use agrocore_domain::entities::site::{Boundary, GeoPoint, Plot, RowConfig, SigpacData, Site, SiteProperty, SiteType};
use agrocore_domain::entities::tenant::TenantId;
use agrocore_domain::entities::{CropType, SiteType};
use agrocore_shared::SharedError;
use validator::Validate;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: GeoJsonGeometry,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeoJsonGeometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShapefileImportRequest {
    pub file_base64: String,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
    pub encoding: Option<String>,
}

impl ImportSiteDto {
    fn to_update_dto(&self) -> UpdateSiteDto {
        UpdateSiteDto {
            label: Some(self.label.clone()),
            variety: self.variety.clone(),
            area: Some(self.area),
            gross_area: self.gross_area,
            plots: self.plots.clone(),
            row_config: self.row_config.clone(),
            bbch_stage: self.bbch_stage.clone(),
            planted_date: self.planted_date,
            cleared_date: self.cleared_date,
            soil_type: self.soil_type.clone(),
            slope: self.slope,
            slope_facing: self.slope_facing.clone(),
            altitude: self.altitude,
            organic: self.organic,
            center: self.center.clone(),
            sigpac_data: self.sigpac_data.clone(),
            regepac_id: self.regepac_id.clone(),
            boundary: self.boundary.clone(),
            properties: self.properties.clone(),
            custom_fields: self.custom_fields.clone(),
            note1: self.note1.clone(),
            note2: self.note2.clone(),
            is_active: self.is_active,
        }
    }
}

impl ImportSitesRequest {
    pub async fn process(
        self,
        state: web::Data<AppState>,
        auth: AuthUser,
    ) -> Result<HttpResponse, ApiError> {
        auth.require_manager()?;

        let tenant_id = TenantId(auth.0.tenant_id);
        let user_id = auth.0.user_id;

        let skip_duplicates = self.skip_duplicates.unwrap_or(true);
        let update_existing = self.update_existing.unwrap_or(false);
        let validate_lpis = self.validate_lpis.unwrap_or(false);

        let mut result = ImportResult {
            total: self.sites.len(),
            created: 0,
            updated: 0,
            skipped: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duplicate_ids: Vec::new(),
        };

        for (index, import_site) in self.sites.into_iter().enumerate() {
            if let Err(e) = import_site.validate() {
                result.errors.push(ImportError {
                    index,
                    label: "unknown".to_string(),
                    error: e.to_string(),
                    field: Some("validation".to_string()),
                });
                continue;
            }

            match process_site(
                &state,
                tenant_id,
                user_id,
                import_site,
                index,
                skip_duplicates,
                update_existing,
                validate_lpis,
            ).await {
                Ok(SiteProcessResult::Created) => result.created += 1,
                Ok(SiteProcessResult::Updated) => result.updated += 1,
                Ok(SiteProcessResult::Skipped(id)) => {
                    result.skipped += 1;
                    result.duplicate_ids.push(id);
                }
                Ok(SiteProcessResult::Warning(warning)) => {
                    result.warnings.push(warning);
                    result.created += 1;
                }
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

        Ok(HttpResponse::Ok().json(result))
    }
}

async fn process_site(
    state: &web::Data<AppState>,
    tenant_id: TenantId,
    user_id: Uuid,
    import_site: ImportSiteDto,
    index: usize,
    skip_duplicates: bool,
    update_existing: bool,
    validate_lpis: bool,
) -> Result<SiteProcessResult, SharedError> {
    // Check for duplicates
    let duplicate_result = check_duplicates(&state.db.site_repo(), tenant_id, &import_site).await?;

    if duplicate_result.is_duplicate {
        if skip_duplicates {
            return Ok(SiteProcessResult::Skipped(duplicate_result.existing_site_id.unwrap()));
        } else if update_existing {
            let update_dto = import_site.to_update_dto();
            let existing_id = duplicate_result.existing_site_id.unwrap();

            let updated = state
                .db
                .site_repo()
                .update(tenant_id, existing_id, update_dto, user_id)
                .await?;

            if let Some(site) = updated {
                let event = Event::new("api".into(), GlobalEvent::SiteUpdated(site.clone()));
                let _ = state.messaging.publish("events.sites", &event).await;
                return Ok(SiteProcessResult::Updated);
            }
        }

        return Ok(SiteProcessResult::Skipped(duplicate_result.existing_site_id.unwrap()));
    }

    // Validate against LPIS if requested
    if validate_lpis {
        if let Some(sigpac) = &import_site.sigpac_data {
            let validation = validate_against_lpis(sigpac, &import_site).await;
            if !validation.is_valid {
                return Err(SharedError::Validation(
                    format!("LPIS validation failed: {}", validation.warnings.join(", "))
                ));
            }
        }
    }

    // Create the site
    let create_dto: CreateSiteDto = import_site.into();
    let site = state
        .db
        .site_repo()
        .create(tenant_id, create_dto, user_id)
        .await?;

    let event = Event::new("api".into(), GlobalEvent::SiteCreated(site.clone()));
    let _ = state.messaging.publish("events.sites", &event).await;

    Ok(SiteProcessResult::Created)
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportSiteDto {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub site_type: SiteType,
    pub crop_type: CropType,
    pub variety: Option<String>,
    #[validate(range(min = 0.0))]
    pub area: f64,
    #[validate(range(min = 0.0))]
    pub gross_area: Option<f64>,
    pub plots: Option<Vec<Plot>>,
    pub row_config: Option<RowConfig>,
    pub bbch_stage: Option<crate::entities::BbchStage>,
    pub planted_date: Option<chrono::DateTime<chrono::Utc>>,
    pub cleared_date: Option<chrono::DateTime<chrono::Utc>>,
    pub soil_type: Option<String>,
    pub slope: Option<f64>,
    pub slope_facing: Option<String>,
    pub altitude: Option<f64>,
    pub organic: Option<bool>,
    pub organic_eligible: Option<bool>,
    pub center: Option<GeoPoint>,
    pub sigpac_data: Option<SigpacData>,
    pub regepac_id: Option<String>,
    pub boundary: Option<Boundary>,
    pub properties: Option<Vec<SiteProperty>>,
    pub custom_fields: Option<serde_json::Value>,
    pub note1: Option<String>,
    pub note2: Option<String>,
    pub is_active: Option<bool>,
    pub is_temporary: Option<bool>,
}

impl From<ImportSiteDto> for CreateSiteDto {
    fn from(import: ImportSiteDto) -> Self {
        CreateSiteDto {
            label: import.label,
            site_type: import.site_type,
            crop_type: import.crop_type,
            variety: import.variety,
            area: import.area,
            gross_area: import.gross_area,
            plots: import.plots,
            row_config: import.row_config,
            bbch_stage: import.bbch_stage,
            planted_date: import.planted_date,
            cleared_date: import.cleared_date,
            soil_type: import.soil_type,
            slope: import.slope,
            slope_facing: import.slope_facing,
            altitude: import.altitude,
            organic: import.organic,
            organic_eligible: import.organic_eligible,
            center: import.center,
            sigpac_data: import.sigpac_data,
            regepac_id: import.regepac_id,
            boundary: import.boundary,
            properties: import.properties,
            custom_fields: import.custom_fields,
            note1: import.note1,
            note2: import.note2,
            is_active: import.is_active,
            is_temporary: import.is_temporary,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DuplicateMatchType {
    ExactBoundary,
    SigpacMatch,
    RegepacMatch,
    HighSimilarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DuplicateDetectionResult {
    pub is_duplicate: bool,
    pub existing_site_id: Option<Uuid>,
    pub match_type: DuplicateMatchType,
    pub similarity_score: f64,
}

async fn check_duplicates(
    repo: &dyn agrocore_domain::repositories::SiteRepository,
    tenant_id: TenantId,
    import_site: &ImportSiteDto,
) -> Result<DuplicateDetectionResult, SharedError> {
    // Check by SigpacData
    if let Some(sigpac) = &import_site.sigpac_data {
        let existing = repo.find_by_id(tenant_id, Uuid::nil()).await?;
        // In a real implementation, you'd query by sigpac_data fields
        // For now, return no duplicate found
    }

    // Check by regepac_id
    if let Some(regepac_id) = &import_site.regepac_id {
        // Query by regepac_id
    }

    // Check by boundary similarity
    if let Some(boundary) = &import_site.boundary {
        // Use PostGIS ST_Area and ST_SymmetricDifference for similarity
    }

    Ok(DuplicateDetectionResult {
        is_duplicate: false,
        existing_site_id: None,
        match_type: DuplicateMatchType::HighSimilarity,
        similarity_score: 0.0,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LpisValidationResult {
    pub is_valid: bool,
    pub sigpac_reference: Option<String>,
    pub area_difference: Option<f64>,
    pub boundary_difference: Option<f64>,
    pub warnings: Vec<String>,
}

async fn validate_against_lpis(
    sigpac: &SigpacData,
    import_site: &ImportSiteDto,
) -> LpisValidationResult {
    let warnings = Vec::new();
    
    // In a real implementation, this would query the LPIS database
    // For now, return a mock result
    LpisValidationResult {
        is_valid: true,
        sigpac_reference: Some(format!(
            "{:02}{:03}{:03}{:03}{:03}{:03}{:03}",
            sigpac.province, sigpac.municipality, sigpac.aggregate,
            sigpac.zone, sigpac.polygon, sigpac.parcel, sigpac.enclosure
        )),
        area_difference: None,
        boundary_difference: None,
        warnings,
    }
}

enum SiteProcessResult {
    Created,
    Updated,
    Skipped(Uuid),
    Warning(ImportWarning),
}

use crate::dto::ImportError;
use crate::dto::ImportWarning;
use agrocore_messaging::{Event, GlobalEvent};