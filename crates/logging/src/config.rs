use crate::error::LoggingResult;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub service_name: String,
    pub environment: EnvironmentType,

    // Console output
    pub console_enabled: bool,
    pub console_pretty: bool,
    pub console_thread_ids: bool,
    pub console_thread_names: bool,

    // File output (JSON)
    pub file_enabled: bool,
    pub log_dir: Option<PathBuf>,
    pub file_path: PathBuf,
    pub file_rotation: RotationConfig,
    pub file_max_files: usize,

    // Per-crate log files and levels
    pub crate_logs: HashMap<String, CrateLogConfig>,

    // OTLP (distributed tracing)
    pub otlp_enabled: bool,
    pub otlp_endpoint: String,
    pub otlp_service_name: String,
    pub otlp_batch_timeout_ms: u64,
    pub otlp_max_export_batch_size: usize,

    // Prometheus metrics
    pub prometheus_enabled: bool,
    pub prometheus_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateLogConfig {
    pub level: String,
    pub file_name: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentType {
    #[default]
    Development,
    Staging,
    Production,
}

// Default is now derived via #[derive(Default)] and #[default] on Development variant

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    pub rotation: RotationType,
    pub max_size_mb: Option<u64>,
    pub max_age_days: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RotationType {
    Daily,
    Hourly,
    Size,
    Never,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut crate_logs = HashMap::new();
        // Default per-crate log configs - can be overridden via config
        crate_logs.insert(
            "agrocore_api".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_api.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "agrocore_domain".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_domain.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "agrocore_infrastructure".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_infrastructure.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "agrocore_messaging".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_messaging.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "agrocore_scheduler".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_scheduler.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "agrocore_worker".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("agrocore_worker.jsonl".to_string()),
                enabled: true,
            },
        );
        // Common framework crates
        crate_logs.insert(
            "actix_web".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("actix_web.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "actix_server".to_string(),
            CrateLogConfig {
                level: "info".to_string(),
                file_name: Some("actix_server.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "sqlx".to_string(),
            CrateLogConfig {
                level: "warn".to_string(),
                file_name: Some("sqlx.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "tower".to_string(),
            CrateLogConfig {
                level: "warn".to_string(),
                file_name: Some("tower.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "hyper".to_string(),
            CrateLogConfig {
                level: "warn".to_string(),
                file_name: Some("hyper.jsonl".to_string()),
                enabled: true,
            },
        );
        crate_logs.insert(
            "reqwest".to_string(),
            CrateLogConfig {
                level: "warn".to_string(),
                file_name: Some("reqwest.jsonl".to_string()),
                enabled: true,
            },
        );

        Self {
            level: "info".to_string(),
            service_name: "agrocore".to_string(),
            environment: EnvironmentType::Development,

            console_enabled: true,
            console_pretty: true,
            console_thread_ids: true,
            console_thread_names: true,

            file_enabled: true,
            log_dir: Some(PathBuf::from("logs")),
            file_path: PathBuf::from("logs/agrocore.json"),
            file_rotation: RotationConfig::default(),
            file_max_files: 30,

            crate_logs,

            otlp_enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            otlp_service_name: "agrocore".to_string(),
            otlp_batch_timeout_ms: 5000,
            otlp_max_export_batch_size: 512,

            prometheus_enabled: false,
            prometheus_endpoint: "0.0.0.0:9090".to_string(),
        }
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            rotation: RotationType::Daily,
            max_size_mb: Some(100),
            max_age_days: Some(30),
        }
    }
}

impl LoggingConfig {
    pub fn from_env() -> LoggingResult<Self> {
        let config = Config::builder()
            .add_source(File::with_name("logging").required(false))
            .add_source(Environment::with_prefix("AGROCORE_LOG").separator("__"))
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }

    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_log_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.log_dir = Some(dir.into());
        self
    }

    pub fn crate_log_level(mut self, crate_name: &str, level: impl Into<String>) -> Self {
        if let Some(crate_config) = self.crate_logs.get_mut(crate_name) {
            crate_config.level = level.into();
        } else {
            self.crate_logs.insert(
                crate_name.to_string(),
                CrateLogConfig {
                    level: level.into(),
                    file_name: Some(format!("{}.jsonl", crate_name.replace('_', "-"))),
                    enabled: true,
                },
            );
        }
        self
    }

    pub fn enable_crate_log(mut self, crate_name: &str, enabled: bool) -> Self {
        if let Some(crate_config) = self.crate_logs.get_mut(crate_name) {
            crate_config.enabled = enabled;
        }
        self
    }

    pub fn get_crate_log_config(&self, crate_name: &str) -> Option<&CrateLogConfig> {
        self.crate_logs.get(crate_name)
    }
}
