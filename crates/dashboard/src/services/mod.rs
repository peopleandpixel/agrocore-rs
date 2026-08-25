//! Service polling modules — each polls an external resource.

pub mod build;
pub mod git;
pub mod process;
pub mod system;

pub use process::ProcessManager;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesState {
    pub postgres: ServiceStatus,
    pub nats: ServiceStatus,
    pub mqtt: ServiceStatus,
}

impl ServicesState {
    pub fn refresh_blocking(&mut self) {
        self.postgres = check_port("127.0.0.1:5432");
        self.nats = check_port("127.0.0.1:4222");
        self.mqtt = check_port("127.0.0.1:1883");
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
    pub fn status(&self) -> String {
        match self.level {
            StatusLevel::Up => format!("✅ UP  ({})", self.detail),
            StatusLevel::Down => format!("❌ DOWN ({})", self.detail),
            StatusLevel::Checking => "🔄 checking...".to_string(),
        }
    }
}

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
