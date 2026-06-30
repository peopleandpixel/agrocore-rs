use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use leptos::prelude::window;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub initialized: bool,
}

pub async fn fetch_system_status() -> Result<SystemStatus, String> {
    let base_url = window().location().origin().unwrap_or_else(|_| "http://localhost:3000".to_string());
    let url = format!("{}/api/v1/system/status", base_url);

    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    
    resp.json::<SystemStatus>()
        .await
        .map_err(|e| e.to_string())
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
    let resp = Request::get("/api/v1/tasks")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        return Err(format!("Error: {}", resp.status()));
    }
    
    resp.json::<PaginatedTasks>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialSetupRequest {
    pub admin: serde_json::Value,
    pub tenant: serde_json::Value,
}

pub async fn initial_setup(req: InitialSetupRequest) -> Result<(), String> {
    let base_url = window().location().origin().unwrap_or_else(|_| "http://localhost:3000".to_string());
    let url = format!("{}/api/v1/system/setup", base_url);
    
    let resp = Request::post(&url)
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
