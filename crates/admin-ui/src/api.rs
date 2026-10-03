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

pub(crate) fn api_base_url() -> String {
    option_env!("AGROCORE_API_BASE_URL")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_default()
}

pub(crate) fn api_url(path: &str) -> String {
    // Absolute URLs pass through untouched so external APIs
    // (e.g. Open-Meteo) are not rewritten onto the local proxy.
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let base = api_base_url();
    if base.is_empty() {
        // Use relative path for nginx proxy
        format!("/{}", path.trim_start_matches('/'))
    } else {
        format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceLogDto {
    pub id: uuid::Uuid,
    pub equipment_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub hours: f64,
    pub note: Option<String>,
    pub performed_at: String,
    pub parts_cost: f64,
    pub labor_hours: f64,
    pub downtime_hours: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceCostSummaryDto {
    pub equipment_id: uuid::Uuid,
    pub total_parts_cost: f64,
    pub total_labor_hours: f64,
    pub total_downtime_hours: f64,
    pub total_maintenance_count: i64,
    pub total_cost: f64,
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
    pub fuel_capacity_liters: Option<f64>,
    pub fuel_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Fuel consumption entry — tracks liters, cost, operation context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuelConsumptionDto {
    pub id: uuid::Uuid,
    pub equipment_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub liters: f64,
    pub cost_per_liter: Option<f64>,
    pub total_cost: Option<f64>,
    pub operation_type: Option<String>,
    pub field_id: Option<uuid::Uuid>,
    pub hours_operated: Option<f64>,
    pub consumed_at: String,
    pub notes: Option<String>,
    pub created_at: String,
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
        params.push(format!(
            "equipment_type={}",
            js_sys::encode_uri_component(eq_type)
        ));
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
    get_json::<PaginatedResponse<EquipmentDto>>(
        &format!("/api/v1/equipments/search{}", query_string),
        true,
    )
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

/// Fetch maintenance history for a specific equipment.
pub async fn fetch_equipment_maintenance_log(
    equipment_id: uuid::Uuid,
) -> Result<Vec<MaintenanceLogDto>, String> {
    get_json(
        &format!("/api/v1/equipments/{}/maintenance", equipment_id),
        true,
    )
    .await
}

pub async fn fetch_maintenance_cost_summary(
    equipment_id: uuid::Uuid,
) -> Result<MaintenanceCostSummaryDto, String> {
    get_json(
        &format!(
            "/api/v1/equipments/{}/maintenance-cost-summary",
            equipment_id
        ),
        true,
    )
    .await
}

pub async fn fetch_fuel_consumption(
    equipment_id: uuid::Uuid,
) -> Result<Vec<FuelConsumptionDto>, String> {
    get_json(
        &format!("/api/v1/equipments/{}/fuel-consumption", equipment_id),
        true,
    )
    .await
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateFuelConsumptionRequest {
    pub liters: f64,
    pub cost_per_liter: Option<f64>,
    pub operation_type: Option<String>,
    pub field_id: Option<uuid::Uuid>,
    pub hours_operated: Option<f64>,
    pub notes: Option<String>,
}

pub async fn record_fuel_consumption(
    equipment_id: uuid::Uuid,
    req: &CreateFuelConsumptionRequest,
) -> Result<FuelConsumptionDto, String> {
    post_json(
        &format!("/api/v1/equipments/{}/fuel-consumption", equipment_id),
        req,
        true,
    )
    .await
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UsageLogDto {
    pub id: uuid::Uuid,
    pub equipment_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub worker_id: Option<uuid::Uuid>,
    pub task_id: Option<uuid::Uuid>,
    pub operation_type: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub hours_operated: f64,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UsageSummaryDto {
    pub equipment_id: uuid::Uuid,
    pub total_hours: f64,
    pub total_sessions: i64,
    pub avg_hours_per_session: f64,
    pub first_used: Option<String>,
    pub last_used: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateUsageLogRequest {
    pub worker_id: Option<uuid::Uuid>,
    pub task_id: Option<uuid::Uuid>,
    pub operation_type: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub hours_operated: Option<f64>,
    pub note: Option<String>,
}

pub async fn fetch_usage_log(equipment_id: uuid::Uuid) -> Result<Vec<UsageLogDto>, String> {
    get_json(&format!("/api/v1/equipments/{}/usage", equipment_id), true).await
}

pub async fn fetch_usage_summary(
    equipment_id: uuid::Uuid,
) -> Result<Option<UsageSummaryDto>, String> {
    get_json(
        &format!("/api/v1/equipments/{}/usage-summary", equipment_id),
        true,
    )
    .await
}

pub async fn record_usage(
    equipment_id: uuid::Uuid,
    req: &CreateUsageLogRequest,
) -> Result<UsageLogDto, String> {
    post_json(
        &format!("/api/v1/equipments/{}/usage", equipment_id),
        req,
        true,
    )
    .await
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum DepreciationMethod {
    #[default]
    StraightLine,
    DoubleDeclining,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EquipmentDepreciationDto {
    pub equipment_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub original_cost: f64,
    pub salvage_value: f64,
    pub purchase_date: Option<String>,
    pub depreciation_method: DepreciationMethod,
    pub useful_life_years: u32,
    pub accumulated_depreciation: f64,
    pub net_book_value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DepreciationScheduleEntry {
    pub year: i32,
    pub depreciation_amount: f64,
    pub accumulated_depreciation: f64,
    pub net_book_value: f64,
}

/// Fetch depreciation summary for an equipment.
pub async fn fetch_depreciation(
    id: uuid::Uuid,
) -> Result<Option<EquipmentDepreciationDto>, String> {
    get_json(&format!("/api/v1/equipments/{}/depreciation", id), true).await
}

/// Fetch depreciation schedule for an equipment.
pub async fn fetch_depreciation_schedule(
    id: uuid::Uuid,
) -> Result<Vec<DepreciationScheduleEntry>, String> {
    get_json(
        &format!("/api/v1/equipments/{}/depreciation-schedule", id),
        true,
    )
    .await
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateAnimalRequest {
    pub species: Option<String>,
    pub breed: Option<String>,
    pub identifier: Option<String>,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub status: Option<String>,
    pub current_site_id: Option<uuid::Uuid>,
    pub group_id: Option<uuid::Uuid>,
    pub weight_kg: Option<f64>,
    pub livestock_type: Option<String>,
    pub plot_id: Option<uuid::Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TreatmentRecordDto {
    pub id: uuid::Uuid,
    pub animal_id: uuid::Uuid,
    pub date: String,
    pub treatment_type: String,
    pub medication: String,
    pub dosage: Option<String>,
    pub veterinarian: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateGrazingRequest {
    pub plot_id: uuid::Uuid,
    pub site_id: Option<uuid::Uuid>,
    pub animal_id: Option<uuid::Uuid>,
    pub start_time: String,
    pub start_date: Option<String>,
}

pub async fn fetch_animals() -> Result<PaginatedResponse<AnimalDto>, String> {
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
    // `CreateAnimalDto` requires these two; they have no default on the Rust
    // side, so omitting them deserialised to nothing and the request was
    // rejected with a 422 the page never surfaced.
    pub livestock_type: String,
    pub status: String,
    pub current_site_id: Option<uuid::Uuid>,
    pub mother_id: Option<uuid::Uuid>,
    pub father_id: Option<uuid::Uuid>,
    pub plot_id: Option<uuid::Uuid>,
}

pub async fn create_animal(req: CreateAnimalRequest) -> Result<AnimalDto, String> {
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
    get_json("/api/v1/workforce/workers", true).await
}

pub async fn fetch_worker(id: uuid::Uuid) -> Result<WorkerDto, String> {
    get_json(&format!("/api/v1/workforce/workers/{}", id), true).await
}

pub async fn create_worker(req: &CreateWorkerRequest) -> Result<WorkerDto, String> {
    post_json("/api/v1/workforce/workers", req, true).await
}

pub async fn update_worker(id: uuid::Uuid, req: &UpdateWorkerRequest) -> Result<WorkerDto, String> {
    let request = Request::put(&api_url(&format!("/api/v1/workforce/workers/{}", id)));
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
        get_json(
            &format!("/api/v1/workforce/workers/{}/clock-entries", wid),
            true,
        )
        .await
    } else {
        get_json("/api/v1/workforce/clock-entries", true).await
    }
}

pub async fn fetch_active_session(worker_id: uuid::Uuid) -> Result<Option<ClockEntryDto>, String> {
    get_json(
        &format!("/api/v1/workforce/workers/{}/clock-active", worker_id),
        true,
    )
    .await
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
        format!("/api/v1/workforce/workers/{}/clock-sessions", worker_id)
    } else {
        format!(
            "/api/v1/workforce/workers/{}/clock-sessions?{}",
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
        format!("/api/v1/workforce/workers/{}/hours-worked", worker_id)
    } else {
        format!(
            "/api/v1/workforce/workers/{}/hours-worked?{}",
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
    get_json("/api/v1/sigpac/parcels", true).await
}

/// GET /api/v1/parcels/{id} — get single parcel
pub async fn get_parcel(id: uuid::Uuid) -> Result<ParcelDto, String> {
    get_json(&format!("/api/v1/sigpac/parcels/{}", id), true).await
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
        "/api/v1/weather/gdd/accumulated?site_id={}&start={}&end={}",
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
        &format!("/api/v1/iot/devices/{}/command", device_id),
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
    pub species: String,
    pub breed: Option<String>,
    pub identifier: String,
    pub birth_date: Option<String>,
    pub gender: Option<String>,
    pub status: String,
    pub current_site_id: Option<uuid::Uuid>,
    pub mother_id: Option<uuid::Uuid>,
    pub father_id: Option<uuid::Uuid>,
    pub created_at: String,
    pub updated_at: String,
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

/// A setting as the API reports it.
///
/// `is_default` distinguishes a value this tenant chose from one inherited from
/// the system defaults, which is what lets the UI mark it as inherited and offer
/// a reset.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingDto {
    pub key: String,
    pub value: serde_json::Value,
    pub value_type: String,
    pub is_default: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub updated_at: String,
    pub updated_by: Option<uuid::Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsListResponse {
    pub settings: Vec<SettingDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingKeyDto {
    pub key: String,
    pub value_type: String,
    pub description: Option<String>,
    pub is_sensitive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreDefaultsResponse {
    pub restored_keys: Vec<String>,
}

async fn put_json<T: DeserializeOwned, B: Serialize>(
    path: &str,
    body: &B,
    auth: bool,
) -> Result<T, String> {
    let req = Request::put(&api_url(path));
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

async fn delete_json(path: &str, auth: bool) -> Result<(), String> {
    let req = Request::delete(&api_url(path));
    let req = if auth { with_auth(req) } else { req };
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    Ok(())
}

/// All effective settings for the current tenant.
pub async fn fetch_settings() -> Result<SettingsListResponse, String> {
    get_json("/api/v1/settings", true).await
}

/// Every known key, including those this tenant has no value for.
pub async fn fetch_setting_keys() -> Result<Vec<SettingKeyDto>, String> {
    get_json("/api/v1/settings/keys", true).await
}

/// Write several settings in one request.
///
/// A `Json::Null` value clears the tenant's override instead of storing a null,
/// matching what the handler does.
pub async fn save_settings(
    values: serde_json::Map<String, serde_json::Value>,
) -> Result<SettingsListResponse, String> {
    let body = serde_json::json!({ "settings": values });
    put_json("/api/v1/settings", &body, true).await
}

/// Write one setting.
pub async fn save_setting(
    key: &str,
    value: serde_json::Value,
) -> Result<SettingsListResponse, String> {
    let body = serde_json::json!({ "value": value });
    put_json(
        &format!("/api/v1/settings/{}", encode_path_segment(key)),
        &body,
        true,
    )
    .await
}

/// Drop this tenant's override for one key.
pub async fn reset_setting(key: &str) -> Result<RestoreDefaultsResponse, String> {
    let req = Request::delete(&api_url(&format!(
        "/api/v1/settings/{}",
        encode_path_segment(key)
    )));
    let resp = with_auth(req).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<RestoreDefaultsResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Remove every tenant override in the system.
pub async fn restore_default_settings() -> Result<RestoreDefaultsResponse, String> {
    let req = Request::post(&api_url("/api/v1/settings/restore-defaults"));
    let resp = with_auth(req).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<RestoreDefaultsResponse>()
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Settings groups (tasks.md F3)
// ---------------------------------------------------------------------------

/// The available groups and their declared fields.
///
/// Used to render the settings page from the server's own field list rather
/// than a copy in the client, which is what let the two drift apart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroupInfo {
    pub namespace: String,
    pub fields: Vec<SettingsGroupField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroupField {
    pub name: String,
    pub key: String,
    /// Human-readable expected type, e.g. "a string".
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroupsResponse {
    pub groups: Vec<SettingsGroupInfo>,
}

pub async fn fetch_setting_groups() -> Result<SettingsGroupsResponse, String> {
    get_json("/api/v1/settings/groups", true).await
}

/// Read one settings group.
///
/// Returns the raw object rather than a typed struct: the group shapes differ,
/// and the editor renders from the declared field list.
pub async fn fetch_setting_group(namespace: &str) -> Result<serde_json::Value, String> {
    get_json(&format!("/api/v1/settings/{namespace}"), true).await
}

/// Write a settings group.
///
/// Sends only the fields present in `values`, so the endpoint's partial-update
/// behaviour keeps the fields this call did not mention.
pub async fn save_setting_group(
    namespace: &str,
    values: serde_json::Map<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    put_json(&format!("/api/v1/settings/{namespace}"), &values, true).await
}

// ---------------------------------------------------------------------------
// Backups
// ---------------------------------------------------------------------------

/// The stored backup configuration.
///
/// Returned by both the GET and the PUT: the handler answers a write with the
/// configuration as saved, so the UI renders what the server has rather than
/// what was typed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfigDto {
    pub enabled: bool,
    pub schedule_db: String,
    pub schedule_config: String,
    pub timezone: String,
    pub targets_count: usize,
    pub retention_daily: u32,
    pub retention_weekly: u32,
    pub retention_monthly: u32,
    pub retention_yearly: u32,
    pub verification_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSummaryDto {
    pub id: String,
    pub backup_type: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_size_bytes: u64,
    pub target_count: usize,
    /// False when no manifest was found and the id and type were inferred from
    /// the object name.
    #[serde(default)]
    pub manifest_backed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRunDto {
    pub id: String,
    pub backup_type: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub target_ids: Vec<String>,
    pub total_size_bytes: u64,
    pub error: Option<String>,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResultDto {
    pub success: bool,
    pub message: String,
}

pub async fn fetch_backup_config() -> Result<BackupConfigDto, String> {
    get_json("/api/v1/backup/config", true).await
}

/// Save the backup configuration.
///
/// Sends only the fields that were filled in, because the endpoint is a partial
/// update: sending an unchanged field as an empty string would overwrite a
/// stored schedule with nothing.
pub async fn save_backup_config(config: &BackupConfigDto) -> Result<BackupConfigDto, String> {
    put_json("/api/v1/backup/config", config, true).await
}

/// Backups found in storage, newest first.
pub async fn fetch_backups() -> Result<Vec<BackupSummaryDto>, String> {
    get_json("/api/v1/backup/backups", true).await
}

/// Start a backup. `backup_type` is "database", "config" or "full".
pub async fn create_backup(backup_type: &str) -> Result<BackupRunDto, String> {
    post_json(
        "/api/v1/backup/backups",
        &serde_json::json!({ "backup_type": backup_type }),
        true,
    )
    .await
}

pub async fn fetch_backup(id: &str) -> Result<BackupRunDto, String> {
    get_json(&format!("/api/v1/backup/backups/{id}"), true).await
}

pub async fn fetch_backup_status(id: &str) -> Result<serde_json::Value, String> {
    get_json(&format!("/api/v1/backup/backups/{id}/status"), true).await
}

/// Restore a backup. `dry_run` checks readability without writing.
pub async fn restore_backup(id: &str, dry_run: bool) -> Result<RestoreResultDto, String> {
    post_json(
        &format!("/api/v1/backup/backups/{id}/restore"),
        &serde_json::json!({
            "backup_id": id,
            "dry_run": dry_run,
        }),
        true,
    )
    .await
}

pub async fn delete_backup(id: &str) -> Result<serde_json::Value, String> {
    let req = Request::delete(&api_url(&format!("/api/v1/backup/backups/{id}")));
    let resp = with_auth(req).send().await.map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())
}

/// Percent-encode one path segment.
///
/// Setting keys contain a dot, which is legal in a path segment, but encoding
/// keeps an arbitrary key from breaking out of the segment.
fn encode_path_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

// --- Plot sub-entities: trees, groups, buildings ---
//
// The `/trees`, `/groups` and `/buildings` routes existed in the backend with
// list, create, update and delete handlers, and the Admin UI pages for them
// collected form input and discarded it. These calls are what makes those pages
// functional.

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TreeDto {
    pub id: uuid::Uuid,
    pub plot_id: uuid::Uuid,
    pub group_id: Option<String>,
    pub tree_type: String,
    pub count: i32,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateTreeRequest {
    pub plot_id: uuid::Uuid,
    pub group_id: Option<String>,
    pub tree_type: String,
    pub count: u32,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateTreeRequest {
    pub group_id: Option<String>,
    pub tree_type: Option<String>,
    pub count: Option<u32>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GroupDto {
    pub id: uuid::Uuid,
    pub plot_id: uuid::Uuid,
    pub parent_group_id: Option<uuid::Uuid>,
    pub group_type: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateGroupRequest {
    pub plot_id: uuid::Uuid,
    pub parent_group_id: Option<uuid::Uuid>,
    pub group_type: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateGroupRequest {
    pub parent_group_id: Option<uuid::Uuid>,
    pub group_type: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BuildingDto {
    pub id: uuid::Uuid,
    pub plot_id: uuid::Uuid,
    pub building_type: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateBuildingRequest {
    pub plot_id: uuid::Uuid,
    pub building_type: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateBuildingRequest {
    pub building_type: Option<String>,
    pub label: Option<String>,
}

pub async fn fetch_trees() -> Result<PaginatedInventoryResponse<TreeDto>, String> {
    get_json("/api/v1/trees", true).await
}

pub async fn fetch_trees_by_plot(
    plot_id: uuid::Uuid,
) -> Result<PaginatedInventoryResponse<TreeDto>, String> {
    get_json(&format!("/api/v1/trees/by-plot/{}", plot_id), true).await
}

pub async fn create_tree(req: CreateTreeRequest) -> Result<TreeDto, String> {
    post_json("/api/v1/trees", &req, true).await
}

pub async fn update_tree(id: uuid::Uuid, req: UpdateTreeRequest) -> Result<TreeDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/trees/{}", id)));
    let request = with_auth(request);
    let resp = request
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<TreeDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_tree(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/trees/{}", id), true).await
}

pub async fn fetch_groups() -> Result<PaginatedInventoryResponse<GroupDto>, String> {
    get_json("/api/v1/groups", true).await
}

pub async fn fetch_groups_by_plot(
    plot_id: uuid::Uuid,
) -> Result<PaginatedInventoryResponse<GroupDto>, String> {
    get_json(&format!("/api/v1/groups/by-plot/{}", plot_id), true).await
}

pub async fn fetch_group_children(id: uuid::Uuid) -> Result<Vec<GroupDto>, String> {
    get_json(&format!("/api/v1/groups/{}/children", id), true).await
}

pub async fn create_group(req: CreateGroupRequest) -> Result<GroupDto, String> {
    post_json("/api/v1/groups", &req, true).await
}

pub async fn update_group(id: uuid::Uuid, req: UpdateGroupRequest) -> Result<GroupDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/groups/{}", id)));
    let request = with_auth(request);
    let resp = request
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<GroupDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_group(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/groups/{}", id), true).await
}

pub async fn fetch_buildings() -> Result<PaginatedInventoryResponse<BuildingDto>, String> {
    get_json("/api/v1/buildings", true).await
}

pub async fn fetch_buildings_by_plot(
    plot_id: uuid::Uuid,
) -> Result<PaginatedInventoryResponse<BuildingDto>, String> {
    get_json(&format!("/api/v1/buildings/by-plot/{}", plot_id), true).await
}

pub async fn create_building(req: CreateBuildingRequest) -> Result<BuildingDto, String> {
    post_json("/api/v1/buildings", &req, true).await
}

pub async fn update_building(
    id: uuid::Uuid,
    req: UpdateBuildingRequest,
) -> Result<BuildingDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/buildings/{}", id)));
    let request = with_auth(request);
    let resp = request
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<BuildingDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_building(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/buildings/{}", id), true).await
}

// --- Livestock ---
//
// The `/livestock` page collected a label and a count and discarded them. The
// animal routes underneath it — list, create, update, delete, treatments,
// grazing — had no caller at all.

pub async fn update_animal(id: uuid::Uuid, req: UpdateAnimalRequest) -> Result<AnimalDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/livestock/animals/{}", id)));
    let request = with_auth(request);
    let resp = request
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    resp.json::<AnimalDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_animal(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/livestock/animals/{}", id), true).await
}

pub async fn fetch_animal_treatments(id: uuid::Uuid) -> Result<Vec<TreatmentRecordDto>, String> {
    get_json(
        &format!("/api/v1/livestock/animals/{}/treatments", id),
        true,
    )
    .await
}

pub async fn add_animal_treatment(
    id: uuid::Uuid,
    req: CreateTreatmentRequest,
) -> Result<TreatmentRecordDto, String> {
    post_json(
        &format!("/api/v1/livestock/animals/{}/treatments", id),
        &req,
        true,
    )
    .await
}

pub async fn add_animal_grazing(
    id: uuid::Uuid,
    req: CreateGrazingRequest,
) -> Result<GrazingRecordDto, String> {
    post_json(
        &format!("/api/v1/livestock/animals/{}/grazing", id),
        &req,
        true,
    )
    .await
}
