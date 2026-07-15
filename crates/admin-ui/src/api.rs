#![allow(dead_code)]

use gloo_net::http::{Request, RequestBuilder};
use leptos::prelude::window;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub initialized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CompanyProfile {
    pub company_name: Option<String>,
    pub tax_id: Option<String>,
    pub office_email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub country: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenMeteoGeocodingLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: Option<String>,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub admin4: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenMeteoGeocodingResponse {
    pub results: Option<Vec<OpenMeteoGeocodingLocation>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenMeteoCurrentWeather {
    pub time: String,
    pub temperature_2m: f64,
    pub relative_humidity_2m: f64,
    pub wind_speed_10m: f64,
    pub precipitation: f64,
    pub weather_code: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenMeteoWeatherResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub current: Option<OpenMeteoCurrentWeather>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WeatherSnapshot {
    pub location_label: String,
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub wind_kmh: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub weather_code: Option<i32>,
    pub observation_time: Option<String>,
}

const COMPANY_PROFILE_KEY: &str = "agrocore.company_profile";
const USER_ROLE_KEY: &str = "agrocore.user_role";

pub fn load_company_profile() -> Option<CompanyProfile> {
    storage()?
        .get_item(COMPANY_PROFILE_KEY)
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str(&value).ok())
}

pub fn save_company_profile(profile: &CompanyProfile) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(
            COMPANY_PROFILE_KEY,
            &serde_json::to_string(profile).unwrap_or_default(),
        );
    }
}

pub async fn geocode_location(query: &str) -> Result<Option<OpenMeteoGeocodingLocation>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(None);
    }

    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        js_sys::encode_uri_component(query)
    );
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Geocoding failed: {}", resp.status()));
    }

    let payload = resp
        .json::<OpenMeteoGeocodingResponse>()
        .await
        .map_err(|e| e.to_string())?;
    Ok(payload.results.and_then(|mut items| items.pop()))
}

pub async fn fetch_open_meteo_weather(
    latitude: f64,
    longitude: f64,
    location_label: String,
) -> Result<WeatherSnapshot, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,precipitation,weather_code&wind_speed_unit=kmh&precipitation_unit=mm&timeformat=iso8601"
    );
    let resp = Request::get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Weather lookup failed: {}", resp.status()));
    }

    let payload = resp
        .json::<OpenMeteoWeatherResponse>()
        .await
        .map_err(|e| e.to_string())?;
    let current = payload.current;

    Ok(WeatherSnapshot {
        location_label,
        temperature_c: current.as_ref().map(|value| value.temperature_2m),
        humidity_percent: current.as_ref().map(|value| value.relative_humidity_2m),
        wind_kmh: current.as_ref().map(|value| value.wind_speed_10m),
        precipitation_mm: current.as_ref().map(|value| value.precipitation),
        weather_code: current.as_ref().map(|value| value.weather_code),
        observation_time: current.map(|value| value.time),
    })
}

pub async fn fetch_weather_for_company_profile() -> Result<Option<WeatherSnapshot>, String> {
    let profile = match load_company_profile() {
        Some(profile) => profile,
        None => return Ok(None),
    };

    let query = profile.address.or(profile.company_name).unwrap_or_default();

    let Some(location) = geocode_location(&query).await? else {
        return Ok(None);
    };

    let location_label = match (location.name.as_str(), location.country.as_deref()) {
        (name, Some(country)) if !country.is_empty() => format!("{name}, {country}"),
        (name, _) => name.to_string(),
    };

    fetch_open_meteo_weather(location.latitude, location.longitude, location_label)
        .await
        .map(Some)
}

fn api_base_url() -> String {
    option_env!("AGROCORE_API_BASE_URL")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            window()
                .location()
                .origin()
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
        })
}

fn api_url(path: &str) -> String {
    format!(
        "{}/{}",
        api_base_url().trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

pub fn download_bytes(filename: &str, mime_type: &str, bytes: &[u8]) -> Result<(), String> {
    let _ = mime_type;
    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);

    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(|e| format!("{:?}", e))?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| format!("{:?}", e))?;
    let document = window()
        .document()
        .ok_or_else(|| String::from("Missing document"))?;
    let element = document
        .create_element("a")
        .map_err(|e| format!("{:?}", e))?;
    let anchor: web_sys::HtmlAnchorElement = element
        .dyn_into()
        .map_err(|_| String::from("Failed to create download link"))?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.set_target("_blank");
    let _ = anchor.style().set_property("display", "none");

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    } else {
        anchor.click();
    }

    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(())
}

pub fn auth_token() -> Option<String> {
    storage()?.get_item("agrocore.auth_token").ok().flatten()
}

pub fn set_auth_token(token: &str) {
    if let Some(storage) = storage() {
        let _ = storage.set_item("agrocore.auth_token", token);
    }
}

pub fn clear_auth_token() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item("agrocore.auth_token");
    }
}

pub fn set_user_role(role: &str) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(USER_ROLE_KEY, role);
    }
}

pub fn user_role() -> Option<String> {
    storage()?.get_item(USER_ROLE_KEY).ok().flatten()
}

pub fn clear_user_role() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(USER_ROLE_KEY);
    }
}

fn with_auth(req: RequestBuilder) -> RequestBuilder {
    if let Some(token) = auth_token() {
        req.header("Authorization", &format!("Bearer {}", token))
    } else {
        req
    }
}

async fn get_json<T: DeserializeOwned>(path: &str, auth: bool) -> Result<T, String> {
    let req = Request::get(&api_url(path));
    let req = if auth { with_auth(req) } else { req };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<T>().await.map_err(|e| e.to_string())
}

async fn get_text(path: &str, auth: bool) -> Result<String, String> {
    let req = Request::get(&api_url(path));
    let req = if auth { with_auth(req) } else { req };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

async fn get_bytes(path: &str, auth: bool) -> Result<Vec<u8>, String> {
    let req = Request::get(&api_url(path));
    let req = if auth { with_auth(req) } else { req };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.binary().await.map_err(|e| e.to_string())
}

async fn post_json<T: DeserializeOwned, B: Serialize>(
    path: &str,
    body: &B,
    auth: bool,
) -> Result<T, String> {
    let req = Request::post(&api_url(path));
    let req = if auth { with_auth(req) } else { req };
    let resp = req
        .json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<T>().await.map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub firstname: String,
    pub lastname: String,
    pub roles: Vec<String>,
}

pub async fn login(req: LoginRequest) -> Result<AuthResponse, String> {
    post_json("/api/v1/auth/login", &req, false).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialSetupRequest {
    pub admin: serde_json::Value,
    pub tenant: serde_json::Value,
}

pub async fn initial_setup(req: InitialSetupRequest) -> Result<(), String> {
    let resp = Request::post(&api_url("/api/v1/system/setup"))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Setup failed: {}", resp.status()));
    }

    Ok(())
}

pub async fn fetch_system_status() -> Result<SystemStatus, String> {
    get_json("/api/v1/system/status", false).await
}

pub async fn delete_tenant() -> Result<(), String> {
    let req = Request::delete(&api_url("/api/v1/system/tenant"));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskData {
    pub id: uuid::Uuid,
    pub description: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedTasks {
    pub data: Vec<TaskData>,
    pub total: u64,
}

pub async fn fetch_tasks() -> Result<PaginatedTasks, String> {
    get_json("/api/v1/tasks", true).await
}

/// Fetch tasks assigned to current worker
pub async fn fetch_my_tasks() -> Result<PaginatedTasks, String> {
    get_json("/api/v1/orders/my-tasks", true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub language: Option<String>,
    pub color: Option<String>,
    pub last_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub label: String,
    pub site_type: String,
    pub crop_type: String,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lng: f64,
    pub lat: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: String,
    pub in_usage: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub label: String,
    pub order_type: String,
    pub status: String,
    pub site_ids: Vec<uuid::Uuid>,
    pub assigned_worker_ids: Vec<uuid::Uuid>,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub last_completed_at: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub cadence: String,
    pub every: u32,
    pub day_of_month: Option<u32>,
    pub month: Option<u32>,
}

pub async fn fetch_users() -> Result<PaginatedResponse<UserDto>, String> {
    get_json("/api/v1/users", true).await
}

pub async fn fetch_sites() -> Result<PaginatedResponse<SiteDto>, String> {
    get_json("/api/v1/sites", true).await
}

pub async fn fetch_equipment() -> Result<PaginatedResponse<EquipmentDto>, String> {
    get_json("/api/v1/equipments", true).await
}

pub async fn fetch_orders() -> Result<PaginatedResponse<OrderDto>, String> {
    get_json("/api/v1/orders", true).await
}

pub async fn fetch_pac_applications() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/finance/pac-applications", true).await
}

pub async fn fetch_cost_centers() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/finance/cost-centers", true).await
}

pub async fn fetch_financial_records() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/finance/financial-records", true).await
}

pub async fn fetch_animals() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/livestock/animals", true).await
}

pub async fn fetch_weather_stations() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/weather/stations", true).await
}

pub async fn fetch_weather_data() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/weather/data", true).await
}

pub async fn fetch_phenology_records() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/weather/phenology", true).await
}

pub async fn fetch_compliance_checklists() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/compliance/checklists", true).await
}

pub async fn fetch_fertilizer_records() -> Result<PaginatedResponse<serde_json::Value>, String> {
    get_json("/api/v1/compliance/fertilizer", true).await
}

pub async fn fetch_plant_protection_records() -> Result<PaginatedResponse<serde_json::Value>, String>
{
    get_json("/api/v1/compliance/plant-protection", true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSiteRequest {
    pub label: String,
    pub site_type: String,
    pub crop_type: String,
    pub variety: Option<String>,
    pub area: f64,
    pub gross_area: Option<f64>,
    pub center: Option<GeoPoint>,
    pub boundary: Option<Vec<GeoPoint>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEquipmentRequest {
    pub label: String,
    pub code: Option<String>,
    pub equipment_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub label: String,
    pub order_type: String,
    pub site_ids: Vec<uuid::Uuid>,
    pub assigned_worker_ids: Option<Vec<uuid::Uuid>>,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

pub async fn create_user(req: CreateUserRequest) -> Result<UserDto, String> {
    post_json("/api/v1/users", &req, true).await
}

pub async fn update_user(id: uuid::Uuid, req: UpdateUserRequest) -> Result<UserDto, String> {
    let body = req;
    let req = Request::put(&api_url(&format!("/api/v1/users/{}", id)));
    let req = with_auth(req);
    let resp = req
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<UserDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_user(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/users/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn create_site(req: CreateSiteRequest) -> Result<SiteDto, String> {
    post_json("/api/v1/sites", &req, true).await
}

pub async fn create_equipment(req: CreateEquipmentRequest) -> Result<EquipmentDto, String> {
    post_json("/api/v1/equipments", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePACApplicationRequest {
    pub year: i32,
    pub application_number: String,
    pub total_eligible_area: f64,
    pub eco_schemes: Vec<serde_json::Value>,
}

pub async fn create_pac_application(
    req: CreatePACApplicationRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/finance/pac-applications", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCostCenterRequest {
    pub label: String,
    pub code: String,
    pub cost_center_type: String,
    pub reference_id: Option<uuid::Uuid>,
}

pub async fn create_cost_center(req: CreateCostCenterRequest) -> Result<serde_json::Value, String> {
    post_json("/api/v1/finance/cost-centers", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateFinancialRecordRequest {
    pub cost_center_id: uuid::Uuid,
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub record_type: String,
    pub category: String,
    pub description: String,
    pub reference_id: Option<uuid::Uuid>,
}

pub async fn create_financial_record(
    req: CreateFinancialRecordRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/finance/financial-records", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAnimalRequest {
    pub species: String,
    pub breed: Option<String>,
    pub identifier: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub current_site_id: Option<uuid::Uuid>,
}

pub async fn create_animal(req: CreateAnimalRequest) -> Result<serde_json::Value, String> {
    post_json("/api/v1/livestock/animals", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateTreatmentRequest {
    pub id: uuid::Uuid,
    pub date: String,
    pub treatment_type: String,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<u32>,
    pub notes: Option<String>,
}

pub async fn add_treatment(
    animal_id: uuid::Uuid,
    req: CreateTreatmentRequest,
) -> Result<serde_json::Value, String> {
    post_json(
        &format!("/api/v1/livestock/animals/{}/treatments", animal_id),
        &req,
        true,
    )
    .await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateWeatherStationRequest {
    pub label: String,
    pub station_type: String,
    pub location: Option<serde_json::Value>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub api_key_config: Option<String>,
}

pub async fn create_weather_station(
    req: CreateWeatherStationRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/weather/stations", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateWeatherDataRequest {
    pub station_id: uuid::Uuid,
    pub timestamp: String,
    pub temperature_c: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_direction_deg: Option<u16>,
    pub solar_radiation_wm2: Option<f64>,
    pub pressure_hpa: Option<f64>,
}

pub async fn create_weather_data(
    req: CreateWeatherDataRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/weather/data", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePhenologyRecordRequest {
    pub site_id: uuid::Uuid,
    pub observation_date: String,
    pub stage: String,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
}

pub async fn create_phenology_record(
    req: CreatePhenologyRecordRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/weather/phenology", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateComplianceChecklistRequest {
    pub site_id: uuid::Uuid,
    pub checklist_type: String,
    pub due_date: Option<String>,
}

pub async fn create_compliance_checklist(
    req: CreateComplianceChecklistRequest,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/compliance/checklists", &req, true).await
}

pub async fn calculate_nutrition_demand(
    req: serde_json::Value,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/nutrition/demand", &req, true).await
}

pub async fn predict_harvest(site_id: uuid::Uuid) -> Result<serde_json::Value, String> {
    get_json(
        &format!("/api/v1/predict/harvest?site_id={}", site_id),
        true,
    )
    .await
}

pub async fn calculate_material_request(
    req: serde_json::Value,
) -> Result<serde_json::Value, String> {
    post_json("/api/v1/calculate/material", &req, true).await
}

pub async fn export_orders_excel() -> Result<Vec<u8>, String> {
    get_bytes("/api/v1/reporting/export/orders/excel", true).await
}

pub async fn export_sites_geojson() -> Result<String, String> {
    get_text("/api/v1/reporting/export/sites/geojson", true).await
}

pub async fn export_pac_sip() -> Result<Vec<u8>, String> {
    get_bytes("/api/v1/reporting/export/pac/sip", true).await
}

pub async fn export_veterinary() -> Result<Vec<u8>, String> {
    get_bytes("/api/v1/reporting/export/veterinary", true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogDto {
    pub id: uuid::Uuid,
    pub entity_type: String,
    pub entity_id: uuid::Uuid,
    pub action: String,
    pub user_id: uuid::Uuid,
    pub timestamp: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

pub async fn fetch_audit_logs() -> Result<PaginatedResponse<AuditLogDto>, String> {
    get_json("/api/v1/compliance/audit-logs", true).await
}

pub async fn create_order(req: CreateOrderRequest) -> Result<OrderDto, String> {
    post_json("/api/v1/orders", &req, true).await
}
