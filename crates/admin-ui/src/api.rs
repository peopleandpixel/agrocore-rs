#![allow(dead_code)]

use agrocore_shared::lpis::LpisCountry;
use gloo_net::http::{Request, RequestBuilder};
use leptos::prelude::window;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::JsCast;

/// Structured error type for API failures.
/// Instead of raw strings, this gives the UI actionable error info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    /// Network error — server unreachable, CORS issue, etc.
    Network(String),
    /// HTTP status error (4xx, 5xx)
    Http { status: u16, message: String },
    /// JSON deserialization failed — response shape changed
    JsonParse(String),
    /// Auth expired or invalid
    Auth { message: String },
}

impl ApiError {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, ApiError::Auth { .. })
    }

    pub fn is_network_error(&self) -> bool {
        matches!(self, ApiError::Network { .. })
    }

    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            ApiError::Http { status, .. } if *status >= 500
        )
    }

    pub fn user_message(&self) -> String {
        match self {
            ApiError::Network(msg) => format!("Verbindungsproblem: {}", msg),
            ApiError::Http { status, message } => {
                if *status >= 500 {
                    format!("Server-Fehler ({}): {}", status, message)
                } else if *status == 401 {
                    "Sitzung abgelaufen. Bitte neu anmelden.".to_string()
                } else if *status == 403 {
                    "Zugriff verweigert.".to_string()
                } else {
                    format!("Fehler ({}): {}", status, message)
                }
            }
            ApiError::JsonParse(msg) => format!("Datenformat ungültig: {}", msg),
            ApiError::Auth { message } => message.clone(),
        }
    }
}

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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TaskData {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub order_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub description: String,
    pub label: String,
    pub order_type: String,
    pub status: String,
    pub planned_date: Option<String>,
    pub deadline_date: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub paused_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub area_covered: Option<f64>,
    pub observations: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
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

/// Fetch a single task by ID — used by task detail pages
pub async fn fetch_task(id: uuid::Uuid) -> Result<TaskData, String> {
    get_json(&format!("/api/v1/tasks/{}", id), true).await
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
    pub is_active: bool,
    pub maintenance_intervals: Option<Vec<MaintenanceIntervalDto>>,
    pub next_maintenance_date: Option<String>,
    pub last_maintenance_hours: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

/// Maintenance interval configuration from the backend.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MaintenanceIntervalDto {
    pub label: String,
    pub interval_hours: Option<f64>,
    pub interval_days: Option<u32>,
}

/// Equipment filter parameters for the search/filter endpoint.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EquipmentFilter {
    pub search: Option<String>,
    pub equipment_type: Option<String>,
    pub in_usage: Option<bool>,
    pub needs_maintenance: Option<bool>,
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

/// Fetch a single equipment by ID — used by equipment detail pages
pub async fn fetch_equipment_by_id(id: uuid::Uuid) -> Result<EquipmentDto, String> {
    get_json(&format!("/api/v1/equipments/{}", id), true).await
}

/// Fetch equipment with search and filter query parameters.
pub async fn fetch_equipment_filtered(
    filter: &EquipmentFilter,
) -> Result<PaginatedResponse<EquipmentDto>, String> {
    let mut params: Vec<String> = Vec::new();
    if let Some(ref search) = filter.search {
        params.push(format!("search={}", js_sys::encode_uri_component(search)));
    }
    if let Some(ref eq_type) = filter.equipment_type {
        params.push(format!("equipment_type={}", js_sys::encode_uri_component(eq_type)));
    }
    if let Some(in_usage) = filter.in_usage {
        params.push(format!("in_usage={}", in_usage));
    }
    if let Some(needs_maint) = filter.needs_maintenance {
        params.push(format!("needs_maintenance={}", needs_maint));
    }
    let query_string = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    get_json::<PaginatedResponse<EquipmentDto>>(&format!("/api/v1/equipments/search{}", query_string), true)
        .await
}

/// Fetch equipment that needs maintenance (next_maintenance_date <= now).
pub async fn fetch_maintenance_due() -> Result<PaginatedResponse<EquipmentDto>, String> {
    get_json("/api/v1/equipments/maintenance", true).await
}

/// Record a maintenance event for equipment.
pub async fn record_maintenance(
    equipment_id: uuid::Uuid,
    hours: f64,
    note: Option<&str>,
) -> Result<EquipmentDto, String> {
    let body = serde_json::json!({
        "hours": hours,
        "note": note,
    });
    let url = api_url(&format!("/api/v1/equipments/{}/maintenance", equipment_id));
    let req = Request::post(&url);
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
    resp.json::<EquipmentDto>().await.map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceLogDto {
    pub id: uuid::Uuid,
    pub equipment_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub hours: f64,
    pub note: Option<String>,
    pub performed_at: String,
    pub created_at: String,
}

/// Fetch maintenance history for a specific equipment.
pub async fn fetch_equipment_maintenance_log(
    equipment_id: uuid::Uuid,
) -> Result<Vec<MaintenanceLogDto>, String> {
    get_json(&format!("/api/v1/equipments/{}/maintenance", equipment_id), true).await
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

pub async fn delete_order(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/orders/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn delete_site(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/sites/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn impersonate_user(id: uuid::Uuid) -> Result<AuthResponse, String> {
    post_json(
        &format!("/api/v1/auth/impersonate/{}", id),
        &serde_json::Value::Null,
        true,
    )
    .await
}

pub async fn stop_impersonation() -> Result<AuthResponse, String> {
    post_json(
        "/api/v1/auth/impersonate/stop",
        &serde_json::Value::Null,
        true,
    )
    .await
}

pub async fn delete_equipment(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/equipments/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn fetch_worker_tasks() -> Result<Vec<OrderDto>, String> {
    get_json("/api/v1/orders/my-tasks", true).await
}

// ===========================================================================
// Data Import API Types & Functions
// ===========================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoJsonImportRequest {
    pub features: Vec<serde_json::Value>,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
    pub lpis_country: Option<LpisCountry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapefileImportRequest {
    pub file_base64: String,
    pub skip_duplicates: Option<bool>,
    pub update_existing: Option<bool>,
    pub validate_lpis: Option<bool>,
    pub encoding: Option<String>,
    pub lpis_country: Option<LpisCountry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub label: String,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportWarning {
    pub label: String,
    pub warning: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub total: u64,
    pub created: u64,
    pub updated: u64,
    pub skipped: u64,
    pub errors: Vec<ImportError>,
    pub warnings: Vec<ImportWarning>,
}

pub async fn import_geojson(req: GeoJsonImportRequest) -> Result<ImportResult, String> {
    post_json("/api/v1/sites/import/geojson", &req, true).await
}

pub async fn import_shapefile(req: ShapefileImportRequest) -> Result<ImportResult, String> {
    post_json("/api/v1/sites/import/shapefile", &req, true).await
}

// =============================================================================
// Worker Task Status API - Multi-Worker Task Status Management
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerTaskStatusTypeDto {
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerTaskStatusDto {
    pub task_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub status: String,
    pub started_at: Option<String>,
    pub paused_at: Option<String>,
    pub resumed_at: Option<String>,
    pub stopped_at: Option<String>,
    pub done_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedWorkerTaskStatusResponse {
    pub data: Vec<WorkerTaskStatusDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateWorkerTaskStatusRequest {
    pub task_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateWorkerTaskStatusRequest {
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerTaskStatusAggregateDto {
    pub task_id: uuid::Uuid,
    pub aggregated_status: String,
    pub worker_statuses: Vec<WorkerTaskStatusDto>,
}

pub async fn fetch_task_worker_statuses(
    task_id: uuid::Uuid,
) -> Result<PaginatedWorkerTaskStatusResponse, String> {
    get_json(&format!("/api/v1/workforce/tasks/{}/status", task_id), true).await
}

pub async fn fetch_worker_task_status(
    task_id: uuid::Uuid,
    worker_id: uuid::Uuid,
) -> Result<WorkerTaskStatusDto, String> {
    get_json(
        &format!("/api/v1/workforce/tasks/{}/status/{}", task_id, worker_id),
        true,
    )
    .await
}

pub async fn create_worker_task_status(
    task_id: uuid::Uuid,
    req: CreateWorkerTaskStatusRequest,
) -> Result<WorkerTaskStatusDto, String> {
    post_json(
        &format!("/api/v1/workforce/tasks/{}/status", task_id),
        &req,
        true,
    )
    .await
}

pub async fn update_worker_task_status(
    task_id: uuid::Uuid,
    worker_id: uuid::Uuid,
    req: UpdateWorkerTaskStatusRequest,
) -> Result<WorkerTaskStatusDto, String> {
    let body = req;
    let req = Request::put(&api_url(&format!(
        "/api/v1/workforce/tasks/{}/status/{}",
        task_id, worker_id
    )));
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
    resp.json::<WorkerTaskStatusDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_aggregated_task_status(
    task_id: uuid::Uuid,
) -> Result<WorkerTaskStatusAggregateDto, String> {
    get_json(
        &format!("/api/v1/workforce/tasks/{}/status/aggregate", task_id),
        true,
    )
    .await
}

// ===========================================================================
// SIGPAC Parcel API Types & Functions
// ===========================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigpacParcelDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedSigpacParcelResponse {
    pub data: Vec<SigpacParcelDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NearPointQuery {
    pub lng: f64,
    pub lat: f64,
    pub radius_m: Option<f64>,
}

pub async fn list_sigpac_parcels(
    query: SigpacParcelQuery,
) -> Result<PaginatedSigpacParcelResponse, String> {
    let mut params = Vec::new();
    if let Some(v) = query.province {
        params.push(format!("province={}", v));
    }
    if let Some(v) = query.municipality {
        params.push(format!("municipality={}", v));
    }
    if let Some(v) = query.aggregate {
        params.push(format!("aggregate={}", v));
    }
    if let Some(v) = query.zone {
        params.push(format!("zone={}", v));
    }
    if let Some(v) = query.polygon {
        params.push(format!("polygon={}", v));
    }
    if let Some(v) = query.parcel {
        params.push(format!("parcel={}", v));
    }
    if let Some(v) = query.enclosure {
        params.push(format!("enclosure={}", v));
    }
    if let Some(v) = query.sigpac_reference {
        params.push(format!(
            "sigpac_reference={}",
            js_sys::encode_uri_component(&v)
        ));
    }
    if let Some(v) = query.page {
        params.push(format!("page={}", v));
    }
    if let Some(v) = query.per_page {
        params.push(format!("per_page={}", v));
    }

    let query_string = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    let url = format!("/api/v1/sigpac/parcels?{}", query_string);
    get_json(&url, true).await
}

pub async fn get_sigpac_parcel(id: uuid::Uuid) -> Result<SigpacParcelDto, String> {
    get_json(&format!("/api/v1/sigpac/parcels/{}", id), true).await
}

pub async fn search_parcels_near_point(
    query: NearPointQuery,
) -> Result<PaginatedSigpacParcelResponse, String> {
    let mut params = Vec::new();
    params.push(format!("lng={}", query.lng));
    params.push(format!("lat={}", query.lat));
    if let Some(r) = query.radius_m {
        params.push(format!("radius_m={}", r));
    }

    let url = format!(
        "/api/v1/sigpac/parcels/search/near-point?{}",
        params.join("&")
    );
    get_json(&url, true).await
}

// --- Inventory API ---

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InventoryItemDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PaginatedInventoryResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InventoryLocationDto {
    pub id: uuid::Uuid,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryBalanceDto {
    pub item_id: uuid::Uuid,
    pub item_name: String,
    pub category: String,
    pub unit: String,
    pub total_quantity: f64,
    pub available_quantity: f64,
    pub reserved_quantity: f64,
    pub average_unit_cost: Option<f64>,
    pub total_value: Option<f64>,
    pub minimum_stock: f64,
    pub is_below_minimum: bool,
    pub inventory_method: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryTransactionDto {
    pub id: uuid::Uuid,
    pub item_id: uuid::Uuid,
    pub transaction_type: String,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<uuid::Uuid>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateInventoryItemRequest {
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockInRequest {
    pub item_id: uuid::Uuid,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub batch_number: Option<String>,
    pub expiration_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockOutRequest {
    pub item_id: uuid::Uuid,
    pub quantity: f64,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    pub item_id: uuid::Uuid,
    pub quantity: f64,
    pub from_location: String,
    pub to_location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdjustRequest {
    pub item_id: uuid::Uuid,
    pub quantity: f64,
    pub notes: String,
}

pub async fn fetch_inventory_items() -> Result<PaginatedInventoryResponse<InventoryItemDto>, String>
{
    get_json("/api/v1/inventory/items", true).await
}

pub async fn fetch_inventory_balances() -> Result<Vec<InventoryBalanceDto>, String> {
    get_json("/api/v1/inventory/balances", true).await
}

pub async fn fetch_below_minimum() -> Result<Vec<InventoryBalanceDto>, String> {
    get_json("/api/v1/inventory/balances/below-minimum", true).await
}

pub async fn fetch_inventory_locations()
-> Result<PaginatedInventoryResponse<InventoryLocationDto>, String> {
    get_json("/api/v1/inventory/locations", true).await
}

pub async fn fetch_item_transactions(
    item_id: uuid::Uuid,
) -> Result<PaginatedInventoryResponse<InventoryTransactionDto>, String> {
    get_json(
        &format!("/api/v1/inventory/items/{}/transactions", item_id),
        true,
    )
    .await
}

pub async fn create_inventory_item(
    req: CreateInventoryItemRequest,
) -> Result<InventoryItemDto, String> {
    post_json("/api/v1/inventory/items", &req, true).await
}

pub async fn delete_inventory_item(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/inventory/items/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn stock_in(req: StockInRequest) -> Result<InventoryTransactionDto, String> {
    post_json("/api/v1/inventory/stock-in", &req, true).await
}

pub async fn stock_out(req: StockOutRequest) -> Result<InventoryTransactionDto, String> {
    post_json("/api/v1/inventory/stock-out", &req, true).await
}

pub async fn transfer_inventory(req: TransferRequest) -> Result<InventoryTransactionDto, String> {
    post_json("/api/v1/inventory/transfer", &req, true).await
}

pub async fn adjust_inventory(req: AdjustRequest) -> Result<InventoryTransactionDto, String> {
    post_json("/api/v1/inventory/adjust", &req, true).await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateInventoryItemRequest {
    pub category: String,
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub unit: String,
    pub minimum_stock: f64,
    pub inventory_method: String,
}

pub async fn update_inventory_item(
    id: uuid::Uuid,
    req: UpdateInventoryItemRequest,
) -> Result<InventoryItemDto, String> {
    let url = format!("/api/v1/inventory/items/{}", id);
    let request = Request::put(&api_url(&url));
    let request = with_auth(request);
    let resp = request
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<InventoryItemDto>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use crate::api;

    #[test]
    fn test_paginated_inventory_response_default() {
        let resp = api::PaginatedInventoryResponse::<api::InventoryItemDto>::default();
        assert_eq!(resp.total, 0);
    }
}

// =============================================================================
// Worker & Job Arbeitskräfte-Management API
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub contract_type: String,
    pub language: Option<String>,
    pub skills: Vec<String>,
    pub certifications: Vec<serde_json::Value>,
    pub emergency_contact: Option<String>,
    pub nationality: Option<String>,
    pub hourly_rate: Option<f64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateWorkerRequest {
    pub user_id: uuid::Uuid,
    pub contract_type: String,
    pub language: Option<String>,
    pub skills: Option<Vec<String>>,
    pub emergency_contact: Option<String>,
    pub nationality: Option<String>,
    pub hourly_rate: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateWorkerRequest {
    pub contract_type: Option<String>,
    pub language: Option<String>,
    pub skills: Option<Vec<String>>,
    pub certifications: Option<Vec<serde_json::Value>>,
    pub emergency_contact: Option<String>,
    pub nationality: Option<String>,
    pub hourly_rate: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedWorkerResponse {
    pub data: Vec<WorkerDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

pub async fn fetch_workers() -> Result<PaginatedWorkerResponse, String> {
    get_json("/api/v1/workers", true).await
}

pub async fn fetch_worker(id: uuid::Uuid) -> Result<WorkerDto, String> {
    get_json(&format!("/api/v1/workers/{}", id), true).await
}

pub async fn create_worker(req: &CreateWorkerRequest) -> Result<WorkerDto, String> {
    post_json("/api/v1/workers", req, true).await
}

pub async fn update_worker(id: uuid::Uuid, req: &UpdateWorkerRequest) -> Result<WorkerDto, String> {
    let request = Request::put(&api_url(&format!("/api/v1/workers/{}", id)));
    let request = with_auth(request);
    let resp = request
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<WorkerDto>().await.map_err(|e| e.to_string())
}

// --- Clock Entry DTOs (Arbeitszeiterfassung) ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockEntryDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub entry_type: String,
    pub timestamp: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub task_id: Option<uuid::Uuid>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateClockEntryRequest {
    pub worker_id: uuid::Uuid,
    pub entry_type: String,
    pub timestamp: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub task_id: Option<uuid::Uuid>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedClockEntryResponse {
    pub data: Vec<ClockEntryDto>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockSessionDto {
    pub worker_id: uuid::Uuid,
    pub clock_in: ClockEntryDto,
    pub clock_out: Option<ClockEntryDto>,
    pub duration_hours: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HoursWorkedResponse {
    pub worker_id: uuid::Uuid,
    pub total_hours: f64,
}

pub async fn fetch_clock_entries(
    worker_id: Option<uuid::Uuid>,
) -> Result<PaginatedClockEntryResponse, String> {
    if let Some(wid) = worker_id {
        get_json(&format!("/api/v1/workers/{}/clock-entries", wid), true).await
    } else {
        get_json("/api/v1/workforce/clock-entries", true).await
    }
}

pub async fn fetch_active_session(worker_id: uuid::Uuid) -> Result<Option<ClockEntryDto>, String> {
    get_json(&format!("/api/v1/workers/{}/clock-active", worker_id), true).await
}

pub async fn clock_in(req: &CreateClockEntryRequest) -> Result<ClockEntryDto, String> {
    let mut req = req.clone();
    req.entry_type = "ClockIn".to_string();
    post_json("/api/v1/workforce/clock-entries", &req, true).await
}

pub async fn clock_out(req: &CreateClockEntryRequest) -> Result<ClockEntryDto, String> {
    let mut req = req.clone();
    req.entry_type = "ClockOut".to_string();
    post_json("/api/v1/workforce/clock-entries", &req, true).await
}

pub async fn delete_clock_entry(id: uuid::Uuid) -> Result<(), String> {
    let req = Request::delete(&api_url(&format!("/api/v1/workforce/clock-entries/{}", id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

pub async fn fetch_worker_sessions(
    worker_id: uuid::Uuid,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<ClockSessionDto>, String> {
    let mut query = String::new();
    if let Some(f) = &from {
        query.push_str(&format!("from={}&", js_sys::encode_uri_component(f)));
    }
    if let Some(t) = &to {
        query.push_str(&format!("to={}&", js_sys::encode_uri_component(t)));
    }
    let path = if query.is_empty() {
        format!("/api/v1/workers/{}/clock-sessions", worker_id)
    } else {
        format!(
            "/api/v1/workers/{}/clock-sessions?{}",
            worker_id,
            query.trim_end_matches('&')
        )
    };
    get_json(&path, true).await
}

pub async fn fetch_total_hours(
    worker_id: uuid::Uuid,
    from: Option<String>,
    to: Option<String>,
) -> Result<HoursWorkedResponse, String> {
    let mut query = String::new();
    if let Some(f) = &from {
        query.push_str(&format!("from={}&", js_sys::encode_uri_component(f)));
    }
    if let Some(t) = &to {
        query.push_str(&format!("to={}&", js_sys::encode_uri_component(t)));
    }
    let path = if query.is_empty() {
        format!("/api/v1/workers/{}/hours-worked", worker_id)
    } else {
        format!(
            "/api/v1/workers/{}/hours-worked?{}",
            worker_id,
            query.trim_end_matches('&')
        )
    };
    get_json(&path, true).await
}

// ============================================================
// Orphan API Route Consumers
// These functions consume backend API routes that were missing
// UI fetch helpers, causing "Orphaned API routes" test warnings.
// ============================================================

/// POST /api/v1/auth/logout — clear auth token
pub async fn logout() -> Result<(), String> {
    let req = Request::post(&api_url("/api/v1/auth/logout"));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Logout failed: {}", resp.status()));
    }
    clear_auth_token();
    Ok(())
}

/// POST /api/v1/auth/refresh — refresh access token using stored refresh token
pub async fn refresh_token() -> Result<String, String> {
    let storage = storage().ok_or_else(|| String::from("Storage not available"))?;
    let refresh = storage
        .get_item("agrocore.refresh_token")
        .ok()
        .flatten()
        .ok_or_else(|| String::from("No refresh token stored"))?;

    let resp = Request::post(&api_url("/api/v1/auth/refresh"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "refresh_token": refresh }))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        return Err(format!("Token refresh failed: {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if let Some(token) = body.get("access_token").and_then(|v| v.as_str()) {
        set_auth_token(token);
        Ok(token.to_string())
    } else {
        Err(String::from("Refresh response missing access_token"))
    }
}

/// GET /api/v1/health — health check endpoint
pub async fn fetch_health() -> Result<SystemStatus, String> {
    get_json("/api/v1/health", false).await
}

/// POST /api/v1/orders/{id}/start — start an order
pub async fn start_order(order_id: uuid::Uuid) -> Result<(), String> {
    let req = Request::post(&api_url(&format!("/api/v1/orders/{}", order_id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Start order failed: {}", resp.status()));
    }
    Ok(())
}

/// POST /api/v1/orders/{id}/complete — complete an order
pub async fn complete_order(order_id: uuid::Uuid) -> Result<(), String> {
    let req = Request::post(&api_url(&format!("/api/v1/orders/{}/complete", order_id)));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Complete order failed: {}", resp.status()));
    }
    Ok(())
}

/// POST /api/v1/tasks/{id}/start-for-worker — start task for worker
pub async fn start_task_for_worker(task_id: uuid::Uuid) -> Result<(), String> {
    let req = Request::post(&api_url(&format!(
        "/api/v1/tasks/{}/start-for-worker",
        task_id
    )));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Start task failed: {}", resp.status()));
    }
    Ok(())
}

/// POST /api/v1/tasks/{id}/stop-for-worker — stop task for worker
pub async fn stop_task_for_worker(task_id: uuid::Uuid) -> Result<(), String> {
    let req = Request::post(&api_url(&format!(
        "/api/v1/tasks/{}/stop-for-worker",
        task_id
    )));
    let req = with_auth(req);
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Stop task failed: {}", resp.status()));
    }
    Ok(())
}

/// GET /api/v1/parcels — list all parcels
pub async fn list_parcels() -> Result<Vec<ParcelDto>, String> {
    get_json("/api/v1/parcels", true).await
}

/// GET /api/v1/parcels/{id} — get single parcel
pub async fn get_parcel(id: uuid::Uuid) -> Result<ParcelDto, String> {
    get_json(&format!("/api/v1/parcels/{}", id), true).await
}

/// GET /api/v1/specialized/sites — specialized site list
pub async fn fetch_specialized_sites() -> Result<serde_json::Value, String> {
    get_json("/api/v1/specialized/sites", true).await
}

/// GET /api/v1/lpis/providers — list LPIS providers
pub async fn fetch_lpis_providers() -> Result<Vec<LpisProviderDto>, String> {
    get_json("/api/v1/lpis/providers", true).await
}

/// GET /api/v1/calculate/water-rate — calculate water rate
pub async fn calculate_water_rate(site_id: uuid::Uuid, days: u32) -> Result<f64, String> {
    let path = format!(
        "/api/v1/calculate/water-rate?site_id={}&days={}",
        site_id, days
    );
    let result: serde_json::Value = get_json(&path, true).await?;
    result
        .get("rate")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| String::from("Missing rate in response"))
}

/// GET /api/v1/gdd/accumulated — accumulated growing degree days
pub async fn fetch_gdd_accumulated(
    site_id: uuid::Uuid,
    start: String,
    end: String,
) -> Result<f64, String> {
    let path = format!(
        "/api/v1/gdd/accumulated?site_id={}&start={}&end={}",
        site_id,
        js_sys::encode_uri_component(&start),
        js_sys::encode_uri_component(&end)
    );
    let result: serde_json::Value = get_json(&path, true).await?;
    result
        .get("accumulated_gdd")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| String::from("Missing accumulated_gdd in response"))
}

/// GET /api/v1/inventory/transactions — list all inventory transactions
pub async fn fetch_all_inventory_transactions() -> Result<Vec<InventoryTransactionDto>, String> {
    get_json("/api/v1/inventory/transactions", true).await
}

/// GET /api/v1/customers/number/{number} — lookup customer by order number
pub async fn fetch_customer_by_number(number: &str) -> Result<CustomerDto, String> {
    get_json(
        &format!(
            "/api/v1/customers/number/{}",
            js_sys::encode_uri_component(number)
        ),
        true,
    )
    .await
}

/// GET /api/v1/customers/search/{query} — search customers
pub async fn search_customers(query: &str) -> Result<Vec<CustomerDto>, String> {
    get_json(
        &format!(
            "/api/v1/customers/search/{}",
            js_sys::encode_uri_component(query)
        ),
        true,
    )
    .await
}

/// GET /api/v1/customers/{id}/orders — orders for a specific customer
pub async fn fetch_customer_orders(customer_id: uuid::Uuid) -> Result<Vec<OrderDto>, String> {
    get_json(&format!("/api/v1/customers/{}/orders", customer_id), true).await
}

/// GET /api/v1/livestock/animals/{id} — get single animal
pub async fn fetch_animal(id: uuid::Uuid) -> Result<AnimalDto, String> {
    get_json(&format!("/api/v1/livestock/animals/{}", id), true).await
}

/// GET /api/v1/livestock/animals/{id}/grazing — grazing records for an animal
pub async fn fetch_animal_grazing(
    animal_id: uuid::Uuid,
    from: Option<String>,
    to: Option<String>,
) -> Result<Vec<GrazingRecordDto>, String> {
    let mut query = String::new();
    if let Some(f) = &from {
        query.push_str(&format!("from={}&", js_sys::encode_uri_component(f)));
    }
    if let Some(t) = &to {
        query.push_str(&format!("to={}&", js_sys::encode_uri_component(t)));
    }
    let path = if query.is_empty() {
        format!("/api/v1/livestock/animals/{}/grazing", animal_id)
    } else {
        format!(
            "/api/v1/livestock/animals/{}/grazing?{}",
            animal_id,
            query.trim_end_matches('&')
        )
    };
    get_json(&path, true).await
}

/// POST /api/v1/devices/{device_id}/command — send command to a device
pub async fn send_device_command(
    device_id: uuid::Uuid,
    command: serde_json::Value,
) -> Result<serde_json::Value, String> {
    post_json(
        &format!("/api/v1/devices/{}/command", device_id),
        &command,
        true,
    )
    .await
}

// DTOs for the new API callers above
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ParcelDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub label: String,
    pub area: f64,
    pub geom: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LpisProviderDto {
    pub country: String,
    pub name: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CustomerDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub order_number_prefix: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AnimalDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: Option<String>,
    pub species: String,
    pub breed: Option<String>,
    pub birth_date: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GrazingRecordDto {
    pub id: uuid::Uuid,
    pub animal_id: uuid::Uuid,
    pub parcel_id: Option<uuid::Uuid>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub notes: Option<String>,
}
