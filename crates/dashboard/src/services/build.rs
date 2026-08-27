//! Build status — runs `cargo check` on demand and reports results.

use serde::{Deserialize, Serialize};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum BuildStatus {
    #[default]
    Unknown,
    Ok,
    Running,
    Failing(String),
}

impl BuildStatus {
    /// Mark the build as "Running" without actually blocking.
    /// The real cargo check is triggered separately via App::run_build.
    pub fn set_running(&mut self) {
        *self = BuildStatus::Running;
    }

    /// Run `cargo check --workspace --quiet` in a blocking thread.
    /// Returns the resulting status. Called by App::run_build_async.
    pub fn run_check() -> BuildStatus {
        let handle = thread::spawn(|| {
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
        handle
            .join()
            .unwrap_or(BuildStatus::Failing("task panicked".into()))
    }
}
