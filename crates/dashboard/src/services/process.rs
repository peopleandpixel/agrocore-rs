//! Process manager — start/stop dev services from the dashboard.

use anyhow::Result;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager
    }

    /// Start a service via docker compose up -d (timeout 30s).
    pub fn start_service(&self, name: &str) -> Result<()> {
        if !matches!(
            name,
            "postgres" | "nats" | "mqtt" | "redis" | "api" | "admin-ui"
        ) {
            return Err(anyhow::anyhow!("unknown service: {}", name));
        }
        let mut child = Command::new("docker")
            .args(["compose", "up", "-d", name])
            .spawn()
            .map_err(|e| anyhow::anyhow!("service {} start spawn failed: {}", name, e))?;
        // Timeout 30s via loop with kill (tokio-timer-kompatibel)
        let timeout = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        return Err(anyhow::anyhow!(
                            "service {} start failed: exit code {:?}",
                            name,
                            status.code()
                        ));
                    }
                    return Ok(());
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        return Err(anyhow::anyhow!(
                            "service {} start timed out after 30s",
                            name
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(anyhow::anyhow!("service {} wait error: {}", name, e)),
            }
        }
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
