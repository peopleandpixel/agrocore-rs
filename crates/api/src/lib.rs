use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{App, HttpServer, web};
use actix_web_prometheus::PrometheusMetricsBuilder;
use prometheus::Registry;
use tracing_actix_web::TracingLogger;
use utoipa_swagger_ui::SwaggerUi;

pub mod dto;
pub mod error;
pub mod handlers;
pub mod metrics;
pub mod middleware;
pub mod openapi;
pub mod services;

#[cfg(test)]
mod dto_validation_tests;

pub use crate::metrics::{BusinessMetrics, DbMetrics};
use crate::middleware::TokenRevocationList;
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
    pub token_revocation: Arc<TokenRevocationList>,
    pub db_metrics: DbMetrics,
    pub business_metrics: BusinessMetrics,
    pub metrics_registry: Arc<Registry>,
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

/// Initialize the token revocation list from centralized configuration.
/// If `REDIS_URL` is set in `AgroCoreConfig`, uses Redis; otherwise falls back to in-memory store.
fn init_token_revocation() -> TokenRevocationList {
    let config = agrocore_shared::config::AgroCoreConfig::global();
    match &config.redis_url {
        Some(url) => match TokenRevocationList::from_redis(url) {
            Ok(trl) => trl,
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize Redis-backed token revocation list: {}. Falling back to in-memory store.",
                    e
                );
                TokenRevocationList::new()
            }
        },
        None => TokenRevocationList::new(),
    }
}

pub async fn run_server(
    db: Database,
    messaging: MessagingClient,
    bind_addr: &str,
) -> std::io::Result<()> {
    // Initialize LPIS Registry with all providers
    let lpis_registry = Arc::new(create_default_registry());

    // Initialize DB metrics registry and metrics instance
    let metrics_registry = Arc::new(Registry::new());
    let db_metrics = DbMetrics::new(&metrics_registry);
    let business_metrics = BusinessMetrics::new(&metrics_registry);

    let state = web::Data::new(AppState {
        db: Arc::new(db),
        messaging: Arc::new(messaging),
        lpis_registry,
        token_revocation: Arc::new(init_token_revocation()),
        db_metrics,
        business_metrics,
        metrics_registry: metrics_registry.clone(),
    });

    let prometheus = PrometheusMetricsBuilder::new("agrocore")
        .endpoint("/metrics")
        .build()
        .unwrap();

    // Default rate limit: 120 req/min (2 req/sec) per IP
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
            .route("/metrics/db", actix_web::web::get().to(db_metrics_handler))
            .route(
                "/metrics/business",
                actix_web::web::get().to(business_metrics_handler),
            )
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

/// HTTP handler that exposes DB metrics in Prometheus text format at /metrics/db.
async fn db_metrics_handler(state: web::Data<AppState>) -> actix_web::HttpResponse {
    let encoder = prometheus::TextEncoder::new();
    let mf = state.metrics_registry.gather();
    match encoder.encode_to_string(&mf) {
        Ok(output) => actix_web::HttpResponse::Ok().body(output),
        Err(e) => {
            tracing::error!("Failed to encode DB metrics: {}", e);
            actix_web::HttpResponse::InternalServerError().body("metrics encode error")
        }
    }
}

/// HTTP handler that exposes ALL metrics (DB + business) at /metrics/business.
async fn business_metrics_handler(state: web::Data<AppState>) -> actix_web::HttpResponse {
    let encoder = prometheus::TextEncoder::new();
    let mf = state.metrics_registry.gather();
    match encoder.encode_to_string(&mf) {
        Ok(output) => actix_web::HttpResponse::Ok().body(output),
        Err(e) => {
            tracing::error!("Failed to encode business metrics: {}", e);
            actix_web::HttpResponse::InternalServerError().body("metrics encode error")
        }
    }
}
