//! Build status — runs `cargo check` periodically and reports results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BuildStatus {
    #[default]
    Unknown,
    Ok,
    Running,
    Failing(String),
}

impl BuildStatus {
    #[allow(dead_code)]
    pub fn refresh(&mut self) {
        *self = BuildStatus::Running;

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

        *self = result
            .join()
            .unwrap_or(BuildStatus::Failing("task panicked".into()));
    }
}
