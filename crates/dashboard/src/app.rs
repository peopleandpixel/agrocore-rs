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
        }
    }

    /// Blocking update — runs all refresh operations synchronously.
    pub fn update_blocking(&mut self) {
        self.services.refresh_blocking();
        self.git.refresh();
        self.system.refresh();
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
}
