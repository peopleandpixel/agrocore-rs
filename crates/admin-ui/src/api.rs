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

// --- Workforce: work logs and worker locations ---
//
// `GET /workforce/logs` and `GET /workforce/locations` had no UI caller. The
// locations endpoint is not a CRUD collection: `POST /locations` is what a
// worker's own device calls to report where it is, and the handler fills in the
// worker id from the authenticated user rather than accepting one — a client
// cannot report a location on someone else's behalf.

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WorkLogDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub date: String,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub rest_period_hours: f64,
    pub task_description: String,
    pub site_id: Option<uuid::Uuid>,
    pub is_night_shift: bool,
    pub breaks_taken: i32,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateWorkLogRequest {
    pub worker_id: uuid::Uuid,
    pub date: String,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub rest_period_hours: f64,
    pub task_description: String,
    pub site_id: Option<uuid::Uuid>,
    pub is_night_shift: bool,
    pub breaks_taken: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateWorkLogRequest {
    pub date: Option<String>,
    pub hours_worked: Option<f64>,
    pub overtime_hours: Option<f64>,
    pub rest_period_hours: Option<f64>,
    pub task_description: Option<String>,
    pub site_id: Option<uuid::Uuid>,
    pub is_night_shift: Option<bool>,
    pub breaks_taken: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WorkerLocationDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub lat: f64,
    pub lng: f64,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReportLocationRequest {
    pub lat: f64,
    pub lng: f64,
    pub current_task_id: Option<uuid::Uuid>,
}

pub async fn fetch_work_logs() -> Result<PaginatedResponse<WorkLogDto>, String> {
    get_json("/api/v1/workforce/logs", true).await
}

pub async fn create_work_log(req: CreateWorkLogRequest) -> Result<WorkLogDto, String> {
    post_json("/api/v1/workforce/logs", &req, true).await
}

pub async fn update_work_log(
    id: uuid::Uuid,
    req: UpdateWorkLogRequest,
) -> Result<WorkLogDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/workforce/logs/{}", id)));
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
    resp.json::<WorkLogDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_work_log(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/workforce/logs/{}", id), true).await
}

/// The latest reported position of every worker in the tenant.
pub async fn fetch_worker_locations() -> Result<Vec<WorkerLocationDto>, String> {
    get_json("/api/v1/workforce/locations", true).await
}

/// Report the caller's own position.
///
/// The handler takes the worker id from the authenticated user, not from the
/// body, so there is no `worker_id` field here to get wrong.
pub async fn report_own_location(req: ReportLocationRequest) -> Result<WorkerLocationDto, String> {
    post_json("/api/v1/workforce/locations", &req, true).await
}

// --- Weather: frost warnings and pest risks ---

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FrostWarningDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub station_id: uuid::Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateFrostWarningRequest {
    pub station_id: uuid::Uuid,
    pub threshold_temp_c: f64,
    pub is_active: bool,
    pub notify_email: bool,
    pub notify_sms: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateFrostWarningRequest {
    pub threshold_temp_c: Option<f64>,
    pub is_active: Option<bool>,
    pub notify_email: Option<bool>,
    pub notify_sms: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PestRiskDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub assessment_date: String,
    pub risk_level: String,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreatePestRiskRequest {
    pub site_id: uuid::Uuid,
    pub assessment_date: String,
    pub risk_level: String,
    pub pest_type: String,
    pub confidence: f64,
    pub recommended_action: String,
    pub model_version: String,
}

pub async fn fetch_frost_warnings() -> Result<PaginatedResponse<FrostWarningDto>, String> {
    get_json("/api/v1/weather/frost-warnings", true).await
}

/// Warnings currently in force, across all stations.
pub async fn fetch_active_frost_warnings() -> Result<Vec<FrostWarningDto>, String> {
    get_json("/api/v1/weather/frost-warnings/active", true).await
}

pub async fn create_frost_warning(
    req: CreateFrostWarningRequest,
) -> Result<FrostWarningDto, String> {
    post_json("/api/v1/weather/frost-warnings", &req, true).await
}

pub async fn update_frost_warning(
    id: uuid::Uuid,
    req: UpdateFrostWarningRequest,
) -> Result<FrostWarningDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/weather/frost-warnings/{}", id)));
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
    resp.json::<FrostWarningDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_frost_warning(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/weather/frost-warnings/{}", id), true).await
}

pub async fn fetch_pest_risks() -> Result<PaginatedResponse<PestRiskDto>, String> {
    get_json("/api/v1/weather/pest-risks", true).await
}

pub async fn create_pest_risk(req: CreatePestRiskRequest) -> Result<PestRiskDto, String> {
    post_json("/api/v1/weather/pest-risks", &req, true).await
}

// --- Compliance: applicator licenses ---

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ApplicatorLicenseDto {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub license_type: String,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateApplicatorLicenseRequest {
    pub user_id: uuid::Uuid,
    pub license_type: String,
    pub license_number: String,
    pub issued_by: String,
    pub valid_from: String,
    pub valid_until: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateApplicatorLicenseRequest {
    pub license_type: Option<String>,
    pub license_number: Option<String>,
    pub issued_by: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn fetch_applicator_licenses() -> Result<PaginatedResponse<ApplicatorLicenseDto>, String>
{
    get_json("/api/v1/compliance/applicator-licenses", true).await
}

pub async fn create_applicator_license(
    req: CreateApplicatorLicenseRequest,
) -> Result<ApplicatorLicenseDto, String> {
    post_json("/api/v1/compliance/applicator-licenses", &req, true).await
}

pub async fn update_applicator_license(
    id: uuid::Uuid,
    req: UpdateApplicatorLicenseRequest,
) -> Result<ApplicatorLicenseDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!(
        "/api/v1/compliance/applicator-licenses/{}",
        id
    )));
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
    resp.json::<ApplicatorLicenseDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_applicator_license(id: uuid::Uuid) -> Result<(), String> {
    delete_json(
        &format!("/api/v1/compliance/applicator-licenses/{}", id),
        true,
    )
    .await
}
// --- Agronomic calculators (`/calculate/*`) ---
//
// Twelve POST endpoints with complete request and response types in the API and
// no UI at all. Each takes a small set of numbers and returns a result, so they
// share one page with a tool picker rather than twelve pages.
//
// The bodies are `serde_json::Value` rather than typed structs: the shapes differ
// per tool, and a typed struct per tool would be thirteen more types to keep in
// step with the API. The field names are the ones
// `crates/api/src/dto/calculations.rs` deserialises, and the enum values are the
// wire renames from the domain enums — `CropType::Grape` serialises as "grape",
// not "Grape", which is the kind of detail that otherwise costs an afternoon.

async fn post_calc(path: &str, body: serde_json::Value) -> Result<serde_json::Value, String> {
    post_json(path, &body, true).await
}

pub async fn calc_nutrition_demand(
    crop_type: &str,
    expected_yield: f64,
    area_ha: f64,
    soil_nitrogen: Option<f64>,
    soil_phosphorus: Option<f64>,
    soil_potassium: Option<f64>,
    organic_matter_percent: Option<f64>,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/nutrition/demand",
        serde_json::json!({
            "crop_type": crop_type,
            "expected_yield": expected_yield,
            "area_ha": area_ha,
            "soil_nitrogen": soil_nitrogen,
            "soil_phosphorus": soil_phosphorus,
            "soil_potassium": soil_potassium,
            "organic_matter_percent": organic_matter_percent,
        }),
    )
    .await
}

pub async fn calc_fertilizer_amount(
    demand: serde_json::Value,
    fertilizer_types: Vec<serde_json::Value>,
    area_ha: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/nutrition/fertilizer-amount",
        serde_json::json!({
            "nutrition_demand": demand,
            "fertilizer_types": fertilizer_types,
            "area_ha": area_ha,
        }),
    )
    .await
}

pub async fn calc_nutrition_balance(
    demand: serde_json::Value,
    applied_amount_kg: f64,
    fertilizer: serde_json::Value,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/nutrition/balance",
        serde_json::json!({
            "demand": demand,
            "applied_amount_kg": applied_amount_kg,
            "fertilizer": fertilizer,
        }),
    )
    .await
}

pub async fn calc_water_rate(
    speed_kmh: f64,
    nozzle_flow_lmin: f64,
    lane_width: f64,
    number_of_nozzles: u32,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/water-rate",
        serde_json::json!({
            "speed_kmh": speed_kmh,
            "nozzle_flow_lmin": nozzle_flow_lmin,
            "lane_width": lane_width,
            "number_of_nozzles": number_of_nozzles,
        }),
    )
    .await
}

pub async fn calc_material(
    method: &str,
    site_id: uuid::Uuid,
    dose_per_ha: f64,
    application_date: Option<String>,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/material",
        serde_json::json!({
            "method": method,
            "site_id": site_id,
            "dose_per_ha": dose_per_ha,
            "application_date": application_date,
        }),
    )
    .await
}

pub async fn calc_tree_crown_volume(
    crown_diameter: f64,
    tree_height: f64,
    trees_per_ha: u32,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/tree-crown-volume",
        serde_json::json!({
            "crown_diameter": crown_diameter,
            "tree_height": tree_height,
            "trees_per_ha": trees_per_ha,
        }),
    )
    .await
}

pub async fn calc_forage_demand(
    body_weight_kg: f64,
    demand_percent: f64,
    animal_count: u32,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/forage-demand",
        serde_json::json!({
            "body_weight_kg": body_weight_kg,
            "demand_percent": demand_percent,
            "animal_count": animal_count,
        }),
    )
    .await
}

pub async fn calc_nitrogen_demand(
    area_ha: f64,
    demand_per_ha: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/nitrogen-demand",
        serde_json::json!({ "area_ha": area_ha, "demand_per_ha": demand_per_ha }),
    )
    .await
}

pub async fn calc_difficulty_surcharge(
    base_rate: f64,
    is_steep: bool,
    is_heavy_soil: bool,
    is_narrow: bool,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/difficulty-surcharge",
        serde_json::json!({
            "base_rate": base_rate,
            "is_steep": is_steep,
            "is_heavy_soil": is_heavy_soil,
            "is_narrow": is_narrow,
        }),
    )
    .await
}

pub async fn calc_profitability(
    yield_amount: f64,
    price_per_unit: f64,
    material_costs: f64,
    labor_costs: f64,
    machinery_costs: f64,
    area_ha: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/profitability",
        serde_json::json!({
            "yield_amount": yield_amount,
            "price_per_unit": price_per_unit,
            "material_costs": material_costs,
            "labor_costs": labor_costs,
            "machinery_costs": machinery_costs,
            "area_ha": area_ha,
        }),
    )
    .await
}

pub async fn calc_harvest_estimation(
    current_bbch: u32,
    target_bbch: u32,
    avg_temp: f64,
    base_temp: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/harvest-estimation",
        serde_json::json!({
            "current_bbch": current_bbch,
            "target_bbch": target_bbch,
            "avg_temp": avg_temp,
            "base_temp": base_temp,
        }),
    )
    .await
}

pub async fn calc_workflow_follow_up(
    order_id: uuid::Uuid,
    next_status: &str,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/workflow/follow-ups",
        serde_json::json!({ "order_id": order_id, "next_status": next_status }),
    )
    .await
}

pub async fn calc_weather_fetch(
    latitude: f64,
    longitude: f64,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/calculate/weather/fetch",
        serde_json::json!({
            "latitude": latitude,
            "longitude": longitude,
            "provider": provider,
            "api_key": api_key,
        }),
    )
    .await
}

pub async fn calc_weather_providers() -> Result<serde_json::Value, String> {
    get_json("/api/v1/calculate/weather/providers", true).await
}
// --- Water: sources, usage, quotas ---

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WaterSourceDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub name: String,
    pub source_type: String,
    pub capacity_m3: Option<f64>,
    pub current_usage_m3: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateWaterSourceRequest {
    pub site_id: uuid::Uuid,
    pub name: String,
    pub source_type: String,
    pub capacity_m3: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateWaterSourceRequest {
    pub name: Option<String>,
    pub source_type: Option<String>,
    pub capacity_m3: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WaterUsageDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub usage_date: String,
    pub volume_m3: f64,
    pub irrigation_method: String,
    pub efficiency_pct: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateWaterUsageRequest {
    pub site_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub usage_date: String,
    pub volume_m3: f64,
    pub irrigation_method: String,
    pub efficiency_pct: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateWaterUsageRequest {
    pub site_id: Option<uuid::Uuid>,
    pub source_id: Option<uuid::Uuid>,
    pub usage_date: Option<String>,
    pub volume_m3: Option<f64>,
    pub irrigation_method: Option<String>,
    pub efficiency_pct: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WaterQuotaDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub source_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub year: i32,
    pub allocated_m3: f64,
    pub used_m3: f64,
    pub remaining_m3: f64,
    pub comunidad_id: Option<uuid::Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateWaterQuotaRequest {
    pub source_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub year: i32,
    pub allocated_m3: f64,
    pub comunidad_id: Option<uuid::Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateWaterQuotaRequest {
    pub allocated_m3: Option<f64>,
    pub used_m3: Option<f64>,
    pub comunidad_id: Option<uuid::Uuid>,
}

pub async fn fetch_water_sources() -> Result<PaginatedInventoryResponse<WaterSourceDto>, String> {
    get_json("/api/v1/water/sources", true).await
}

pub async fn create_water_source(req: CreateWaterSourceRequest) -> Result<WaterSourceDto, String> {
    post_json("/api/v1/water/sources", &req, true).await
}

pub async fn update_water_source(
    id: uuid::Uuid,
    req: UpdateWaterSourceRequest,
) -> Result<WaterSourceDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/water/sources/{}", id)));
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
    resp.json::<WaterSourceDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_water_source(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/water/sources/{}", id), true).await
}

pub async fn fetch_water_usage() -> Result<PaginatedInventoryResponse<WaterUsageDto>, String> {
    get_json("/api/v1/water/usage", true).await
}

pub async fn create_water_usage(req: CreateWaterUsageRequest) -> Result<WaterUsageDto, String> {
    post_json("/api/v1/water/usage", &req, true).await
}

pub async fn update_water_usage(
    id: uuid::Uuid,
    req: UpdateWaterUsageRequest,
) -> Result<WaterUsageDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/water/usage/{}", id)));
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
    resp.json::<WaterUsageDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_water_usage(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/water/usage/{}", id), true).await
}

pub async fn fetch_water_quotas() -> Result<PaginatedInventoryResponse<WaterQuotaDto>, String> {
    get_json("/api/v1/water/quotas", true).await
}

pub async fn create_water_quota(req: CreateWaterQuotaRequest) -> Result<WaterQuotaDto, String> {
    post_json("/api/v1/water/quotas", &req, true).await
}

pub async fn update_water_quota(
    id: uuid::Uuid,
    req: UpdateWaterQuotaRequest,
) -> Result<WaterQuotaDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/water/quotas/{}", id)));
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
    resp.json::<WaterQuotaDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_water_quota(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/water/quotas/{}", id), true).await
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HarvestSeasonDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub year: i32,
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateHarvestSeasonRequest {
    pub year: i32,
    pub label: String,
    pub start_date: String,
    pub end_date: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateHarvestSeasonRequest {
    pub label: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HarvestLotDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub season_id: uuid::Uuid,
    pub lot_number: String,
    pub crop_type: String,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
    pub total_weight_kg: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateHarvestLotRequest {
    pub season_id: uuid::Uuid,
    pub lot_number: String,
    pub site_ids: Vec<uuid::Uuid>,
    pub crop_type: String,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateHarvestLotRequest {
    pub lot_number: Option<String>,
    pub site_ids: Option<Vec<uuid::Uuid>>,
    pub crop_type: Option<String>,
    pub variety: Option<String>,
    pub quality_target: Option<String>,
    pub total_weight_kg: Option<f64>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HarvestDeliveryDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub lot_id: uuid::Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateHarvestDeliveryRequest {
    pub lot_id: uuid::Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub tare_weight_kg: f64,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateHarvestDeliveryRequest {
    pub delivery_date: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub tare_weight_kg: Option<f64>,
    pub carrier_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub quality_notes: Option<String>,
    pub temperature_at_delivery: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ColdChainLogDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub lot_id: uuid::Uuid,
    pub sensor_id: String,
    pub recorded_at: String,
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateColdChainLogRequest {
    pub lot_id: uuid::Uuid,
    pub sensor_id: String,
    pub recorded_at: String,
    pub temperature_c: f64,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateColdChainLogRequest {
    pub lot_id: Option<uuid::Uuid>,
    pub sensor_id: Option<String>,
    pub recorded_at: Option<String>,
    pub temperature_c: Option<f64>,
    pub humidity_pct: Option<f64>,
    pub location: Option<String>,
}

pub async fn fetch_harvest_seasons() -> Result<PaginatedInventoryResponse<HarvestSeasonDto>, String>
{
    get_json("/api/v1/harvest/seasons", true).await
}

pub async fn create_harvest_season(
    req: CreateHarvestSeasonRequest,
) -> Result<HarvestSeasonDto, String> {
    post_json("/api/v1/harvest/seasons", &req, true).await
}

pub async fn update_harvest_season(
    id: uuid::Uuid,
    req: UpdateHarvestSeasonRequest,
) -> Result<HarvestSeasonDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/harvest/seasons/{}", id)));
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
    resp.json::<HarvestSeasonDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_harvest_season(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/harvest/seasons/{}", id), true).await
}

pub async fn fetch_harvest_lots() -> Result<PaginatedInventoryResponse<HarvestLotDto>, String> {
    get_json("/api/v1/harvest/lots", true).await
}

pub async fn create_harvest_lot(req: CreateHarvestLotRequest) -> Result<HarvestLotDto, String> {
    post_json("/api/v1/harvest/lots", &req, true).await
}

pub async fn update_harvest_lot(
    id: uuid::Uuid,
    req: UpdateHarvestLotRequest,
) -> Result<HarvestLotDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/harvest/lots/{}", id)));
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
    resp.json::<HarvestLotDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_harvest_lot(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/harvest/lots/{}", id), true).await
}

pub async fn fetch_harvest_deliveries()
-> Result<PaginatedInventoryResponse<HarvestDeliveryDto>, String> {
    get_json("/api/v1/harvest/deliveries", true).await
}

pub async fn create_harvest_delivery(
    req: CreateHarvestDeliveryRequest,
) -> Result<HarvestDeliveryDto, String> {
    post_json("/api/v1/harvest/deliveries", &req, true).await
}

pub async fn update_harvest_delivery(
    id: uuid::Uuid,
    req: UpdateHarvestDeliveryRequest,
) -> Result<HarvestDeliveryDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/harvest/deliveries/{}", id)));
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
    resp.json::<HarvestDeliveryDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_harvest_delivery(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/harvest/deliveries/{}", id), true).await
}

pub async fn fetch_cold_chain_logs() -> Result<PaginatedInventoryResponse<ColdChainLogDto>, String>
{
    get_json("/api/v1/harvest/cold-chain", true).await
}

pub async fn create_cold_chain_log(
    req: CreateColdChainLogRequest,
) -> Result<ColdChainLogDto, String> {
    post_json("/api/v1/harvest/cold-chain", &req, true).await
}

pub async fn update_cold_chain_log(
    id: uuid::Uuid,
    req: UpdateColdChainLogRequest,
) -> Result<ColdChainLogDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/harvest/cold-chain/{}", id)));
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
    resp.json::<ColdChainLogDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_cold_chain_log(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/harvest/cold-chain/{}", id), true).await
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OliveGroveDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub label: String,
    pub variety: String,
    pub planting_year: Option<i32>,
    pub area_ha: f64,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateOliveGroveRequest {
    pub site_id: uuid::Uuid,
    pub label: String,
    pub variety: String,
    pub planting_year: Option<i32>,
    pub area_ha: f64,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateOliveGroveRequest {
    pub label: Option<String>,
    pub variety: Option<String>,
    pub planting_year: Option<i32>,
    pub area_ha: Option<f64>,
    pub tree_count: Option<i32>,
    pub spacing_m: Option<f64>,
    pub irrigation_type: Option<String>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OliveOilRecordDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub olive_grove_id: uuid::Uuid,
    pub harvest_date: String,
    pub quantity_kg: f64,
    pub oil_yield_kg: f64,
    pub oil_yield_percent: f64,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateOliveOilRecordRequest {
    pub olive_grove_id: uuid::Uuid,
    pub harvest_date: String,
    pub quantity_kg: f64,
    pub oil_yield_kg: f64,
    pub oil_yield_percent: f64,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateOliveOilRecordRequest {
    pub olive_grove_id: Option<uuid::Uuid>,
    pub harvest_date: Option<String>,
    pub quantity_kg: Option<f64>,
    pub oil_yield_kg: Option<f64>,
    pub oil_yield_percent: Option<f64>,
    pub acidity_percent: Option<f64>,
    pub peroxide_value: Option<f64>,
    pub k232: Option<f64>,
    pub k270: Option<f64>,
    pub quality_grade: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VineyardDto {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<String>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateVineyardRequest {
    pub site_id: uuid::Uuid,
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<String>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: bool,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateVineyardRequest {
    pub doc_area: Option<String>,
    pub vintage: Option<i32>,
    pub grape_variety: Option<String>,
    pub brix_at_harvest: Option<f64>,
    pub ph_at_harvest: Option<f64>,
    pub acidity: Option<f64>,
    pub yield_tons: Option<f64>,
    pub quality_grade: Option<String>,
    pub slope_percent: Option<f64>,
    pub altitude_m: Option<f64>,
    pub is_organic: Option<bool>,
    pub certification_body: Option<String>,
    pub certification_number: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct KelterDeliveryDto {
    pub id: uuid::Uuid,
    pub vineyard_id: uuid::Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub lot_number: String,
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CreateKelterDeliveryRequest {
    pub vineyard_id: uuid::Uuid,
    pub delivery_date: String,
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub lot_number: String,
    pub kelter_name: String,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpdateKelterDeliveryRequest {
    pub vineyard_id: Option<uuid::Uuid>,
    pub delivery_date: Option<String>,
    pub gross_weight_kg: Option<f64>,
    pub net_weight_kg: Option<f64>,
    pub lot_number: Option<String>,
    pub kelter_name: Option<String>,
    pub transport_company: Option<String>,
    pub temperature_c: Option<f64>,
    pub notes: Option<String>,
}

pub async fn fetch_olive_groves() -> Result<PaginatedInventoryResponse<OliveGroveDto>, String> {
    get_json("/api/v1/specialized/olive-groves", true).await
}

pub async fn create_olive_grove(req: CreateOliveGroveRequest) -> Result<OliveGroveDto, String> {
    post_json("/api/v1/specialized/olive-groves", &req, true).await
}

pub async fn update_olive_grove(
    id: uuid::Uuid,
    req: UpdateOliveGroveRequest,
) -> Result<OliveGroveDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!(
        "/api/v1/specialized/olive-groves/{}",
        id
    )));
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
    resp.json::<OliveGroveDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_olive_grove(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/specialized/olive-groves/{}", id), true).await
}

pub async fn fetch_olive_oil_records()
-> Result<PaginatedInventoryResponse<OliveOilRecordDto>, String> {
    get_json("/api/v1/specialized/olive-oil-records", true).await
}

pub async fn create_olive_oil_record(
    req: CreateOliveOilRecordRequest,
) -> Result<OliveOilRecordDto, String> {
    post_json("/api/v1/specialized/olive-oil-records", &req, true).await
}

pub async fn update_olive_oil_record(
    id: uuid::Uuid,
    req: UpdateOliveOilRecordRequest,
) -> Result<OliveOilRecordDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!(
        "/api/v1/specialized/olive-oil-records/{}",
        id
    )));
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
    resp.json::<OliveOilRecordDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_olive_oil_record(id: uuid::Uuid) -> Result<(), String> {
    delete_json(
        &format!("/api/v1/specialized/olive-oil-records/{}", id),
        true,
    )
    .await
}

pub async fn fetch_vineyards() -> Result<PaginatedInventoryResponse<VineyardDto>, String> {
    get_json("/api/v1/specialized/vineyards", true).await
}

pub async fn create_vineyard(req: CreateVineyardRequest) -> Result<VineyardDto, String> {
    post_json("/api/v1/specialized/vineyards", &req, true).await
}

pub async fn update_vineyard(
    id: uuid::Uuid,
    req: UpdateVineyardRequest,
) -> Result<VineyardDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!("/api/v1/specialized/vineyards/{}", id)));
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
    resp.json::<VineyardDto>().await.map_err(|e| e.to_string())
}

pub async fn delete_vineyard(id: uuid::Uuid) -> Result<(), String> {
    delete_json(&format!("/api/v1/specialized/vineyards/{}", id), true).await
}

pub async fn fetch_kelter_deliveries()
-> Result<PaginatedInventoryResponse<KelterDeliveryDto>, String> {
    get_json("/api/v1/specialized/kelter-deliveries", true).await
}

pub async fn create_kelter_delivery(
    req: CreateKelterDeliveryRequest,
) -> Result<KelterDeliveryDto, String> {
    post_json("/api/v1/specialized/kelter-deliveries", &req, true).await
}

pub async fn update_kelter_delivery(
    id: uuid::Uuid,
    req: UpdateKelterDeliveryRequest,
) -> Result<KelterDeliveryDto, String> {
    let body = req;
    let request = Request::put(&api_url(&format!(
        "/api/v1/specialized/kelter-deliveries/{}",
        id
    )));
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
    resp.json::<KelterDeliveryDto>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_kelter_delivery(id: uuid::Uuid) -> Result<(), String> {
    delete_json(
        &format!("/api/v1/specialized/kelter-deliveries/{}", id),
        true,
    )
    .await
}

/// The legacy alias for the fertiliser calculator.
///
/// `nutrition.rs` and `calculation.rs` each register a fertiliser-amount
/// endpoint — `/nutrition/fertilizer-amount` and
/// `/calculate/nutrition/fertilizer-amount` — with two near-identical handler
/// bodies. Both are live, and the UI is expected to reach both, so the alias is
/// bound explicitly rather than left as the one path with no caller.
pub async fn calc_fertilizer_amount_legacy(
    demand: serde_json::Value,
    fertilizer_types: Vec<serde_json::Value>,
    area_ha: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/nutrition/fertilizer-amount",
        serde_json::json!({
            "nutrition_demand": demand,
            "fertilizer_types": fertilizer_types,
            "area_ha": area_ha,
        }),
    )
    .await
}

/// Profitability for a specialised crop.
///
/// `specialized.rs` registers this at `/specialized/profitability` while
/// `calculation.rs` registers the same calculation at `/calculate/profitability`.
/// Both are live.
pub async fn calc_specialized_profitability(
    yield_amount: f64,
    price_per_unit: f64,
    material_costs: f64,
    labor_costs: f64,
    machinery_costs: f64,
    area_ha: f64,
) -> Result<serde_json::Value, String> {
    post_calc(
        "/api/v1/specialized/profitability",
        serde_json::json!({
            "yield_amount": yield_amount,
            "price_per_unit": price_per_unit,
            "material_costs": material_costs,
            "labor_costs": labor_costs,
            "machinery_costs": machinery_costs,
            "area_ha": area_ha,
        }),
    )
    .await
}
