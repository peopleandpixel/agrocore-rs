//! Application state — holds all data for the TUI panels.

use crate::services::{
    ProcessManager, ServicesState, build::BuildStatus, git::GitStatus, system::SystemInfo,
};
use std::time::Instant;

#[derive(Debug)]
#[allow(dead_code)]
pub struct App {
    pub start_time: Instant,
    pub current_tab: usize,
    pub tabs: Vec<&'static str>,
    pub services: ServicesState,
    pub build: BuildStatus,
    pub git: GitStatus,
    pub system: SystemInfo,
    pub process_mgr: ProcessManager,
    pub error: Option<String>,
    /// History of CPU percentages for sparkline rendering (last 30 samples).
    pub cpu_history: Vec<f32>,
    /// History of memory percentages for sparkline rendering (last 30 samples).
    pub mem_history: Vec<f32>,
    /// Whether the service control menu is open.
    pub show_service_menu: bool,
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
            tabs: vec!["Services", "Build", "Git", "System"],
            services: ServicesState::default(),
            build: BuildStatus::Unknown,
            git: GitStatus::default(),
            system: SystemInfo::default(),
            process_mgr: ProcessManager::new(),
            error: None,
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            show_service_menu: false,
        }
    }

    /// Blocking update — runs all refresh operations synchronously.
    pub fn update_blocking(&mut self) {
        self.services.refresh_blocking();
        self.git.refresh();
        self.system.refresh();

        // Update history (keep last 30 samples for sparkline)
        self.cpu_history.push(self.system.cpu_percent);
        self.mem_history.push(self.system.mem_percent);
        if self.cpu_history.len() > 30 {
            self.cpu_history.remove(0);
        }
        if self.mem_history.len() > 30 {
            self.mem_history.remove(0);
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    #[allow(dead_code)]
    pub fn run_build_async(&mut self) {
        self.build = BuildStatus::Running;
        let result = std::thread::spawn(|| {
            use std::process::Command;
            let out = Command::new("cargo")
                .args(["check", "--workspace", "--quiet"])
                .output();
            match out {
                Ok(o) if o.status.success() => BuildStatus::Ok,
                Ok(_) => BuildStatus::Failing("check failed".into()),
                Err(_) => BuildStatus::Failing("cargo not found".into()),
            }
        });

        match result.join() {
            Ok(status) => self.build = status,
            Err(_) => self.build = BuildStatus::Failing("task panicked".into()),
        }
    }

    /// Restart a specific service by name via docker compose.
    #[allow(dead_code)]
    pub fn restart_service(&mut self, service: &str) {
        let service_name = service.to_string();
        let result = std::thread::spawn(move || {
            use std::process::Command;
            let _ = Command::new("docker")
                .args([
                    "compose",
                    "-f",
                    "docker-compose.dev.yml",
                    "restart",
                    &service_name,
                ])
                .status();
        });
        match result.join() {
            Ok(_) => self.error = Some(format!("{} restarted", service)),
            Err(_) => self.error = Some(format!("Failed to restart {}", service)),
        }
    }

    /// Restart all services via docker compose.
    #[allow(dead_code)]
    pub fn restart_all(&mut self) {
        let result = std::thread::spawn(|| {
            use std::process::Command;
            let _ = Command::new("docker")
                .args(["compose", "-f", "docker-compose.dev.yml", "restart"])
                .status();
        });
        match result.join() {
            Ok(_) => self.error = Some("All services restarted".to_string()),
            Err(_) => self.error = Some("Failed to restart services".to_string()),
        }
    }

    /// Stop all services via docker compose.
    #[allow(dead_code)]
    pub fn stop_all(&mut self) {
        let result = std::thread::spawn(|| {
            use std::process::Command;
            let _ = Command::new("docker")
                .args(["compose", "-f", "docker-compose.dev.yml", "stop"])
                .status();
        });
        match result.join() {
            Ok(_) => self.error = Some("All services stopped".to_string()),
            Err(_) => self.error = Some("Failed to stop services".to_string()),
        }
    }
}
