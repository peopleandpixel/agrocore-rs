use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_telemetry(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{}=info,tower_http=info", service_name)));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();
}
