use crate::error::LoggingResult;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
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
    pub file_path: PathBuf,
    pub file_rotation: RotationConfig,
    pub file_max_files: usize,

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentType {
    Development,
    Staging,
    Production,
}

impl Default for EnvironmentType {
    fn default() -> Self {
        EnvironmentType::Development
    }
}

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
        Self {
            level: "info".to_string(),
            service_name: "agrocore".to_string(),
            environment: EnvironmentType::Development,

            console_enabled: true,
            console_pretty: true,
            console_thread_ids: true,
            console_thread_names: true,

            file_enabled: false,
            file_path: PathBuf::from("logs/agrocore.json"),
            file_rotation: RotationConfig::default(),
            file_max_files: 30,

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
}
