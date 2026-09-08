//! Telemetry and logging infrastructure.
//!
//! Provides:
//! - `init_telemetry` — standard logging setup via agrocore_logging (stdout only)
//! - `init_telemetry_with_logs` — stdout + JSONL logging for the dashboard
//! - `append_api_log` — manually append an API call log entry to JSONL
//! - `append_sql_log` — manually append a SQL query entry to JSONL
//!
//! The JSONL files (logs/api_calls.jsonl, logs/sql_queries.jsonl)
//! are consumed by the agrocore-dashboard TUI (tabs 5 & 6).

use agrocore_logging::{EnvironmentType, LoggingConfig, init_logging};
use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

/// Simple API call log entry for JSONL output.
#[derive(Serialize)]
struct ApiLogEntry {
    timestamp: chrono::DateTime<Utc>,
    client: String,
    method: String,
    path: String,
    status: u16,
    latency_ms: u64,
    user_agent: Option<String>,
}

/// Simple SQL query log entry for JSONL output.
#[derive(Serialize)]
struct SqlLogEntry {
    timestamp: chrono::DateTime<Utc>,
    query: String,
    params: Vec<String>,
    duration_ms: u64,
    success: bool,
    error: Option<String>,
    db_label: String,
}

/// Initialize standard logging telemetry for any service (stdout only).
pub fn init_telemetry(service_name: &str) {
    let config = LoggingConfig {
        level: "info".to_string(),
        service_name: service_name.to_string(),
        environment: EnvironmentType::Development,
        console_enabled: true,
        console_pretty: true,
        console_thread_ids: true,
        console_thread_names: true,
        ..Default::default()
    };
    let _ = init_logging(config);
}

/// Initialize telemetry with JSONL logging for the dashboard.
///
/// Same as `init_telemetry` but also ensures the logs directory exists.
/// The actual JSONL writing is done manually via `append_api_log`/`append_sql_log`
/// since agrocore_logging doesn't have a built-in JSONL layer.
pub fn init_telemetry_with_logs(service_name: &str) {
    init_telemetry(service_name);
    ensure_log_dir();
}

fn ensure_log_dir() {
    let _ = std::fs::create_dir_all("logs");
}

/// Append a single API call log entry to logs/api_calls.jsonl.
/// Called manually from handlers when needed.
pub fn append_api_log(
    client: &str,
    method: &str,
    path: &str,
    status: u16,
    latency_ms: u64,
    user_agent: Option<&str>,
) {
    ensure_log_dir();
    let entry = ApiLogEntry {
        timestamp: Utc::now(),
        client: client.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        latency_ms,
        user_agent: user_agent.map(|s| s.to_string()),
    };
    write_jsonl("logs/api_calls.jsonl", &entry);
}

/// Append a single SQL query log entry to logs/sql_queries.jsonl.
pub fn append_sql_log(
    query: &str,
    params: &[String],
    duration_ms: u64,
    success: bool,
    error: Option<&str>,
    db_label: &str,
) {
    ensure_log_dir();
    let entry = SqlLogEntry {
        timestamp: Utc::now(),
        query: query.to_string(),
        params: params.to_vec(),
        duration_ms,
        success,
        error: error.map(|s| s.to_string()),
        db_label: db_label.to_string(),
    };
    write_jsonl("logs/sql_queries.jsonl", &entry);
}

fn write_jsonl<T: Serialize>(path: &str, value: &T) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let line = serde_json::to_string(value).unwrap_or_default();
        let _ = writeln!(file, "{}", line);
    }
}
