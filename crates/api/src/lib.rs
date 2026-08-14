use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{App, HttpServer, web};
use actix_web_prometheus::PrometheusMetricsBuilder;
use tracing_actix_web::TracingLogger;
use utoipa_swagger_ui::SwaggerUi;

pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod openapi;
pub mod services;

#[cfg(test)]
mod dto_validation_tests;

use agrocore_infrastructure::Database;
use agrocore_lpis_providers::create_default_registry;
use agrocore_messaging::MessagingClient;
use agrocore_shared::lpis::LpisRegistry;
use std::sync::Arc;

// Re-export for admin-ui
pub use agrocore_shared::lpis::LpisProviderConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub messaging: Arc<MessagingClient>,
    pub lpis_registry: Arc<LpisRegistry>,
}

/// Builds a CORS configuration from the `CORS_ALLOWED_ORIGINS` environment variable.
/// If the variable is set, only the specified origins are allowed.
/// If unset, falls back to permissive mode (all origins) for development.
///
/// The env var accepts a comma-separated list of origins, e.g.:
/// `CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com`
fn build_cors() -> Cors {
    match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins_str) => {
            let mut cors = Cors::default()
                .max_age(3600)
                .allow_any_method()
                .allow_any_header()
                .supports_credentials();
            for origin in origins_str.split(',') {
                let origin = origin.trim();
                if !origin.is_empty() {
                    cors = cors.allowed_origin(origin);
                }
            }
            cors
        }
        Err(_) => {
            tracing::warn!(
                "CORS_ALLOWED_ORIGINS not set - using permissive CORS policy (all origins allowed). \
                 Set CORS_ALLOWED_ORIGINS in production for security."
            );
            Cors::permissive().max_age(3600)
        }
    }
}

pub async fn run_server(
    db: Database,
    messaging: MessagingClient,
    bind_addr: &str,
) -> std::io::Result<()> {
    // Initialize LPIS Registry with all providers
    let lpis_registry = Arc::new(create_default_registry());

    let state = web::Data::new(AppState {
        db: Arc::new(db),
        messaging: Arc::new(messaging),
        lpis_registry,
    });

    let prometheus = PrometheusMetricsBuilder::new("agrocore")
        .endpoint("/metrics")
        .build()
        .unwrap();

    // Governor Rate Limiting: 120 req/min (2 req/sec) per IP
    let gov_conf = GovernorConfigBuilder::default()
        .seconds_per_request(1)
        .burst_size(120)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        let cors = build_cors();
        let security_headers = actix_web::middleware::DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("X-Frame-Options", "DENY"))
            .add(("X-XSS-Protection", "1; mode=block"))
            .add((
                "Strict-Transport-Security",
                "max-age=31536000; includeSubDomains",
            ))
            .add(("Content-Security-Policy", "default-src 'self'"));

        App::new()
            .app_data(state.clone())
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .wrap(security_headers)
            .wrap(Governor::new(&gov_conf))
            .wrap(cors)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url(
                "/api-docs/openapi.json",
                openapi::ApiDoc::openapi_with_security(),
            ))
            .configure(handlers::configure)
            .service(
                fs::Files::new("/admin", "/var/lib/agrocore/admin-ui")
                    .index_file("index.html")
                    .default_handler(web::to(|| async {
                        fs::NamedFile::open("/var/lib/agrocore/admin-ui/index.html")
                    })),
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}
