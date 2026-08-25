//! Process manager — start/stop dev services from the dashboard.

#[derive(Debug, Default)]
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager
    }

    #[allow(dead_code)]
    pub fn start_service(&self, name: &str) -> anyhow::Result<u32> {
        let cmd = match name {
            "postgres" => ("docker", vec!["compose", "up", "-d", "postgres"]),
            "nats" => ("docker", vec!["compose", "up", "-d", "nats"]),
            _ => return Err(anyhow::anyhow!("unknown service: {}", name)),
        };

        let status = std::process::Command::new(cmd.0).args(&cmd.1).status()?;

        if status.success() {
            Ok(std::process::id())
        } else {
            Err(anyhow::anyhow!("failed to start {}", name))
        }
    }
}
