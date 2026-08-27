//! Git status — shows current branch, dirty/clean state, and changed files.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
    pub changed_files: Vec<String>,
}

impl GitStatus {
    pub fn refresh(&mut self) {
        // Branch name
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".into());
        self.branch = branch;

        // Dirty check
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        self.changed_files = status
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        self.dirty = !self.changed_files.is_empty();
    }
}
