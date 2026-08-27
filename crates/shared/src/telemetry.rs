//! Telemetry and logging infrastructure.
//!
//! Provides:
//! - `init_telemetry` — standard tracing setup with env-filter (stdout only)
//! - `init_telemetry_with_logs` — stdout + JSONL logging for the dashboard
//! - `append_api_log` — manually append an API call log entry to JSONL
//! - `append_sql_log` — manually append a SQL query entry to JSONL
//!
//! The JSONL files (logs/api_calls.jsonl, logs/sql_queries.jsonl)
//! are consumed by the agrocore-dashboard TUI (tabs 5 & 6).

use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Layer};

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

/// Initialize standard tracing telemetry for any service (stdout only).
pub fn init_telemetry(service_name: &str) {
    let _ = init_tracing_subscriber(service_name, false);
}

/// Initialize telemetry with JSONL logging for the dashboard.
///
/// Same as `init_telemetry` but also installs a `DashboardLogLayer`
/// that captures `tracing_actix_web::TracingLogger` events and writes
/// structured JSONL entries to logs/api_calls.jsonl.
pub fn init_telemetry_with_logs(service_name: &str) {
    let _ = init_tracing_subscriber(service_name, true);
}

fn init_tracing_subscriber(
    service_name: &str,
    with_logs: bool,
) -> Result<(), tracing_subscriber::util::TryInitError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{}=info,tower_http=info", service_name)));

    if with_logs {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(DashboardLogLayer::new("logs"))
            .try_init()
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()
    }
}

/// A tracing subscriber layer that writes structured JSONL entries to disk
/// for the agrocore-dashboard TUI to consume.
///
/// Captures events from `tracing_actix_web::TracingLogger` (which instruments
/// every HTTP request via the `tracing_log` target) and from explicit
/// `tracing::info!(event = "sql_query", ...)` calls for SQL logging.
struct DashboardLogLayer {
    log_dir: String,
}

impl DashboardLogLayer {
    fn new(log_dir: &str) -> Self {
        let _ = std::fs::create_dir_all(log_dir);
        Self {
            log_dir: log_dir.to_string(),
        }
    }

    fn append_jsonl(&self, filename: &str, value: serde_json::Value) {
        let path = format!("{}/{}", self.log_dir, filename);
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut file| writeln!(file, "{}", value));
    }
}

impl<S> Layer<S> for DashboardLogLayer
where
    S: tracing::Subscriber + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();

        // Handle TracingLogger events (target = "tracing_actix_web::middleware::tracing_log")
        if meta.target() == "tracing_actix_web::middleware::tracing_log" {
            let fields = collect_fields(event);

            let method = fields.get("request.method").cloned().unwrap_or_default();
            let path = fields.get("request.path").cloned().unwrap_or_default();
            let status = fields
                .get("response.status")
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0);
            let client = fields
                .get("request.remote")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let user_agent = fields.get("request.user_agent").cloned();

            // Try to get latency from response.elapsed (format: "Duration(...)")
            let latency_ms = fields
                .get("response.elapsed")
                .and_then(|s| {
                    s.trim_start_matches("Duration(")
                        .trim_end_matches(')')
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(0);

            if !method.is_empty() && !path.is_empty() {
                let entry = serde_json::json!({
                    "timestamp": Utc::now(),
                    "client": client,
                    "method": method,
                    "path": path,
                    "status": status,
                    "latency_ms": latency_ms,
                    "user_agent": user_agent,
                });
                self.append_jsonl("api_calls.jsonl", entry);
            }
        }

        // Handle explicit sql_query events
        let fields = collect_fields(event);
        if fields.get("event").map(|s| s.as_str()) == Some("sql_query") {
            let entry = serde_json::json!({
                "timestamp": Utc::now(),
                "query": fields.get("query").cloned().unwrap_or_default(),
                "params": fields.get("params").map(|s| vec![s.clone()]).unwrap_or_default(),
                "duration_ms": fields.get("duration_ms").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0),
                "success": fields.get("success").map(|s| s == "true").unwrap_or(true),
                "error": fields.get("error").cloned(),
                "db_label": fields.get("db_label").cloned().unwrap_or_default(),
            });
            self.append_jsonl("sql_queries.jsonl", entry);
        }
    }
}

/// Collect all field values from a tracing Event into a HashMap.
fn collect_fields(event: &tracing::Event<'_>) -> std::collections::HashMap<String, String> {
    let map = Mutex::new(std::collections::HashMap::new());
    event.record(&mut FieldCollector(&map));
    map.into_inner().unwrap()
}

/// Helper to collect field values using the Visit trait.
struct FieldCollector<'a>(&'a Mutex<std::collections::HashMap<String, String>>);

impl<'a> tracing::field::Visit for FieldCollector<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .lock()
            .unwrap()
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .lock()
            .unwrap()
            .insert(field.name().to_string(), format!("{:?}", value));
    }
}

/// Ensure the logs directory exists.
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
