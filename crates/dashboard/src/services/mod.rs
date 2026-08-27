//! Service polling modules — each polls an external resource.
//!
//! These modules are data-only: they collect raw metrics, build statuses,
//! git info, process lists, and service health. The rendering layer
//! (views/mod.rs) handles all UI concerns using SuperLightTUI state types.

pub mod build;
pub mod git;
pub mod log;
pub mod process;
pub mod system;

pub use build::BuildStatus;
pub use git::GitStatus;
pub use log::{ApiLogEntry, LogBuffer, SqlLogEntry};
pub use process::ProcessManager;
pub use system::SystemInfo;

use serde::{Deserialize, Serialize};

/// Aggregated snapshot of all infrastructure services.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesState {
    pub postgres: ServiceStatus,
    pub nats: ServiceStatus,
    pub mqtt: ServiceStatus,
    pub redis: ServiceStatus,
    pub admin_ui: ServiceStatus,
    pub api: ServiceStatus,
}

impl ServicesState {
    /// Polling snapshot — called every refresh cycle from App::refresh.
    pub fn refresh(&mut self) {
        self.postgres = check_port("127.0.0.1:5432");
        self.nats = check_port("127.0.0.1:4222");
        self.mqtt = check_port("127.0.0.1:1883");
        self.redis = check_port("127.0.0.1:6379");
        self.api = check_port("127.0.0.1:8080");
        self.admin_ui = check_port("127.0.0.1:80");
    }

    /// All services and their display labels — used for table rendering.
    pub fn entries(&self) -> Vec<(&str, &ServiceStatus)> {
        vec![
            ("PostgreSQL", &self.postgres),
            ("NATS", &self.nats),
            ("MQTT", &self.mqtt),
            ("Redis", &self.redis),
            ("API", &self.api),
            ("Admin UI", &self.admin_ui),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum StatusLevel {
    #[default]
    Down,
    Up,
    Checking,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceStatus {
    pub level: StatusLevel,
    pub detail: String,
}

impl ServiceStatus {
    pub fn is_up(&self) -> bool {
        self.level == StatusLevel::Up
    }

    pub fn status_label(&self) -> &'static str {
        match self.level {
            StatusLevel::Up => "UP",
            StatusLevel::Down => "DOWN",
            StatusLevel::Checking => "CHECKING",
        }
    }
}

/// Check connectivity to a TCP port (e.g. database, broker, HTTP server).
fn check_port(addr: &str) -> ServiceStatus {
    use std::net::TcpStream;
    use std::time::Duration;

    let parsed = addr.parse().unwrap_or("127.0.0.1:0".parse().unwrap());
    match TcpStream::connect_timeout(&parsed, Duration::from_millis(500)) {
        Ok(_) => ServiceStatus {
            level: StatusLevel::Up,
            detail: addr.to_string(),
        },
        Err(_) => ServiceStatus {
            level: StatusLevel::Down,
            detail: addr.to_string(),
        },
    }
}
