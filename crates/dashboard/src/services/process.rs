//! Process manager — start/stop dev services from the dashboard.

use anyhow::Result;
use std::process::Command;

#[derive(Debug, Default)]
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager
    }

    /// Start a service via docker compose up -d.
    pub fn start_service(&self, name: &str) -> Result<()> {
        if !matches!(
            name,
            "postgres" | "nats" | "mqtt" | "redis" | "api" | "admin-ui"
        ) {
            return Err(anyhow::anyhow!("unknown service: {}", name));
        }
        Command::new("docker")
            .args(["compose", "up", "-d", name])
            .status()?;
        Ok(())
    }

    /// Restart a service via docker compose restart.
    pub fn restart_service(&self, name: &str) -> Result<()> {
        Command::new("docker")
            .args(["compose", "restart", name])
            .status()?;
        Ok(())
    }

    /// Restart all services via docker compose restart.
    pub fn restart_all(&self) -> Result<()> {
        Command::new("docker")
            .args(["compose", "restart"])
            .status()?;
        Ok(())
    }

    /// Stop all services via docker compose stop.
    pub fn stop_all(&self) -> Result<()> {
        Command::new("docker").args(["compose", "stop"]).status()?;
        Ok(())
    }
}
