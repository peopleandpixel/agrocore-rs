//! Application state — holds all data for the TUI dashboard.
//!
//! Following SuperLightTUI patterns: App state is a plain Rust struct.
//! UI-local state (scroll position, table cursor, toast queue) lives
//! here as SLT state types so they persist across frames.

use crate::services::{
    ProcessManager, ServicesState, build::BuildStatus, git::GitStatus, log::LogBuffer,
    system::SystemInfo,
};
use slt::{ScrollState, SpinnerState, TableState, ToastState};
use std::time::Instant;

#[derive(Debug)]
pub struct App {
    /// When the dashboard was launched.
    pub start_time: Instant,

    /// Tab index: 0=Services, 1=Build, 2=Git, 3=System, 4=API Log, 5=SQL Log.
    pub current_tab: usize,
    pub tabs: Vec<&'static str>,

    // Data snapshots — refreshed every cycle
    pub services: ServicesState,
    pub build: BuildStatus,
    pub git: GitStatus,
    pub system: SystemInfo,
    pub process_mgr: ProcessManager,

    // UI-local SLT state — persists across frames
    pub spinner: SpinnerState,
    pub toast_state: ToastState,
    pub table_cursor: TableState,
    pub log_scroll: ScrollState,
    pub api_log_scroll: ScrollState,
    pub sql_log_scroll: ScrollState,

    /// CPU/memory trend data for sparklines (last 30 samples).
    pub cpu_history: Vec<f32>,
    pub mem_history: Vec<f32>,

    /// Whether a build check was triggered (to be run in main loop).
    pub build_pending: bool,

    /// Whether the service control menu is open.
    pub show_service_menu: bool,

    // Log buffers — recent API calls and SQL queries
    pub api_log: LogBuffer<crate::services::log::ApiLogEntry>,
    pub sql_log: LogBuffer<crate::services::log::SqlLogEntry>,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            start_time: Instant::now(),
            current_tab: 0,
            tabs: vec!["Services", "Build", "Git", "System", "API Log", "SQL Log"],
            services: ServicesState::default(),
            build: BuildStatus::default(),
            git: GitStatus::default(),
            system: SystemInfo::default(),
            process_mgr: ProcessManager::new(),
            spinner: SpinnerState::dots(),
            toast_state: ToastState::new(),
            table_cursor: TableState::new(
                vec!["Service", "Status", "Detail"],
                vec![
                    vec![
                        "PostgreSQL".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:5432".to_string(),
                    ],
                    vec![
                        "NATS".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:4222".to_string(),
                    ],
                    vec![
                        "MQTT".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:1883".to_string(),
                    ],
                    vec![
                        "Redis".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:6379".to_string(),
                    ],
                    vec![
                        "API".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:8080".to_string(),
                    ],
                    vec![
                        "Admin UI".to_string(),
                        "DOWN".to_string(),
                        "127.0.0.1:80".to_string(),
                    ],
                ],
            ),
            log_scroll: ScrollState::new(),
            api_log_scroll: ScrollState::new(),
            sql_log_scroll: ScrollState::new(),
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            build_pending: false,
            show_service_menu: false,
            api_log: LogBuffer::new(100),
            sql_log: LogBuffer::new(100),
        }
    }

    /// Refresh all data snapshots — called every frame.
    pub fn refresh(&mut self) {
        self.services.refresh();
        self.git.refresh();
        self.system.refresh();

        // Update history (keep last 30 samples for charts)
        self.cpu_history.push(self.system.cpu_percent);
        self.mem_history.push(self.system.mem_percent);
        if self.cpu_history.len() > 30 {
            self.cpu_history.remove(0);
        }
        if self.mem_history.len() > 30 {
            self.mem_history.remove(0);
        }

        // Update the table view with current service statuses
        let rows: Vec<Vec<String>> = self
            .services
            .entries()
            .iter()
            .map(|(name, status)| {
                vec![
                    (*name).to_string(),
                    status.status_label().to_string(),
                    status.detail.clone(),
                ]
            })
            .collect();
        self.table_cursor.set_rows(rows);

        // Refresh log buffers from JSONL files
        self.refresh_logs();
    }

    /// Read log files from disk — called on every refresh cycle.
    fn refresh_logs(&mut self) {
        self.api_log = read_api_log("logs/api_calls.jsonl", 100);
        self.sql_log = read_sql_log("logs/sql_queries.jsonl", 100);
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Restart a specific service by name via docker compose.
    pub fn restart_service(&mut self, service: &str) {
        match self.process_mgr.restart_service(service) {
            Ok(_) => {
                self.toast_state
                    .success(format!("{} restarted", service), 0);
            }
            Err(e) => {
                self.toast_state.error(format!("Failed: {}", e), 0);
            }
        }
    }

    /// Restart all services via docker compose.
    pub fn restart_all(&mut self) {
        match self.process_mgr.restart_all() {
            Ok(_) => {
                self.toast_state.success("All services restarted", 0);
            }
            Err(e) => {
                self.toast_state.error(format!("Failed: {}", e), 0);
            }
        }
    }

    /// Stop all services via docker compose.
    pub fn stop_all(&mut self) {
        match self.process_mgr.stop_all() {
            Ok(_) => {
                self.toast_state.success("All services stopped", 0);
            }
            Err(e) => {
                self.toast_state.error(format!("Failed: {}", e), 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// JSONL file readers — parse log entries from disk.
// ---------------------------------------------------------------------------

fn read_api_log(path: &str, capacity: usize) -> LogBuffer<crate::services::log::ApiLogEntry> {
    let mut buffer = LogBuffer::new(capacity);
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<crate::services::log::ApiLogEntry>(line) {
                buffer.push(entry);
            }
        }
    }
    buffer
}

fn read_sql_log(path: &str, capacity: usize) -> LogBuffer<crate::services::log::SqlLogEntry> {
    let mut buffer = LogBuffer::new(capacity);
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(entry) = serde_json::from_str::<crate::services::log::SqlLogEntry>(line) {
                buffer.push(entry);
            }
        }
    }
    buffer
}
