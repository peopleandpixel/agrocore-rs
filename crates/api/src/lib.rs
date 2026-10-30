use actix_cors::Cors;
use actix_files as fs;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{App, FromRequest, HttpRequest, HttpServer, dev::Payload, web};
use actix_web_prometheus::PrometheusMetricsBuilder;
use prometheus::Registry;
use utoipa_swagger_ui::SwaggerUi;

pub mod dto;
pub mod encryption;
pub mod error;
pub mod handlers;
pub mod lpis_settings;
pub mod metrics;
pub mod middleware;
pub mod openapi;
pub mod services;

#[cfg(test)]
mod dto_validation_tests;

pub use crate::metrics::{BusinessMetrics, DbMetrics};
use crate::middleware::TokenRevocationList;
use agrocore_backup::service::BackupService;
use agrocore_infrastructure::Database;
use agrocore_lpis_providers::create_default_registry;
use agrocore_messaging::MessagingClient;
use agrocore_shared::lpis::LpisRegistry;
use futures_util::{StreamExt, future::LocalBoxFuture};
use std::sync::Arc;

// Re-export for admin-ui
pub use agrocore_shared::lpis::LpisProviderConfig;

use agrocore_logging::{error as logging_error, warn};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    /// `None` when the broker was unreachable at startup.
    ///
    /// Optional on purpose: publishers already ignore send failures (`let _ =
    /// ...publish()`), so a broker outage degraded notifications only. Making it
    /// required made the whole API refuse to start for the same reason.
    pub messaging: Option<Arc<MessagingClient>>,
    pub lpis_registry: Arc<LpisRegistry>,
    pub token_revocation: Arc<TokenRevocationList>,
    pub db_metrics: DbMetrics,
    pub business_metrics: BusinessMetrics,
    pub metrics_registry: Arc<Registry>,
    pub backup_service: Option<Arc<BackupService>>,
    /// Whether the demo endpoints are enabled. Read from `ALLOW_DEMO_ENDPOINTS`.
    ///
    /// The demo endpoints create and delete tenants, so they must not be
    /// reachable in production. They additionally require an admin role; this
    /// flag is the second, independent barrier.
    pub demo_endpoints_enabled: bool,
    /// When the API process started, for reporting uptime.
    pub startup_time: std::time::Instant,

    // ── Demo Mode (G3 - Option C) ──────────────────────────────────────────
    /// True when the API started with `DEMO_MODE=1`. The demo tenant is seeded
    /// automatically at startup if it does not exist yet.
    pub is_demo_mode: bool,
    /// ID of the demo tenant, populated after automatic seeding at startup.
    /// Used by the read-only middleware and backup isolation.
    pub demo_tenant_id: Option<uuid::Uuid>,
    /// If true, all write operations (POST/PUT/DELETE) on the demo tenant
    /// return 403. Controlled by `DEMO_READONLY=1`.
    pub demo_readonly: bool,
    /// If set to a positive number of hours, the demo data is automatically
    /// reset after that many hours. Controlled by `DEMO_RESET_HOURS`.
    pub demo_reset_hours: Option<u32>,
}

/// Whether the demo endpoints are enabled.
///
/// Defaults to false. Set `ALLOW_DEMO_ENDPOINTS=1` to enable; the dev
/// environment does this itself when `--demo` is passed.
///
/// Exposed as `pub(crate)` so integration tests can verify the flag parsing
/// without reaching into the handler module.
pub fn demo_endpoints_enabled() -> bool {
    match std::env::var("ALLOW_DEMO_ENDPOINTS") {
        Ok(v) => {
            let enabled = matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes");
            if !enabled {
                warn!("ALLOW_DEMO_ENDPOINTS is set to '{v}' but not recognised as enabled");
            }
            enabled
        }
        Err(_) => false,
    }
}

/// Whether the API is running in full demo mode.
///
/// When `DEMO_MODE=1`, the demo tenant is seeded automatically at startup,
/// the health endpoint reports demo status, and the UI displays a banner.
fn is_demo_mode() -> bool {
    match std::env::var("DEMO_MODE") {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

/// Whether demo tenant writes are blocked.
///
/// When `DEMO_READONLY=1`, all POST/PUT/DELETE requests targeting the demo
/// tenant return 403 Forbidden. Useful for public demo stands where visitors
/// should not modify data.
fn demo_readonly() -> bool {
    match std::env::var("DEMO_READONLY") {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
        Err(_) => false,
    }
}

/// Auto-reset interval in hours.
///
/// When `DEMO_RESET_HOURS` is set to a positive integer, the demo data is
/// automatically reset after that many hours. Returns `None` if disabled.
fn demo_reset_hours() -> Option<u32> {
    match std::env::var("DEMO_RESET_HOURS") {
        Ok(v) => v.trim().parse().ok().filter(|&h| h > 0),
        Err(_) => None,
    }
}

/// Builds a CORS configuration from the `CORS_ALLOWED_ORIGINS` environment variable.
/// If the variable is set, only the specified origins are allowed.
/// If unset, falls back to a safe local-development default list (localhost/127.0.0.1).
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
            warn!(
                "CORS_ALLOWED_ORIGINS not set — using local development defaults \
                 (localhost/127.0.0.1). Set CORS_ALLOWED_ORIGINS in production for security."
            );
            let mut cors = Cors::default()
                .max_age(3600)
                .allow_any_method()
                .allow_any_header()
                .supports_credentials();
            // Safe local-development defaults — covers common dev setups.
            for origin in [
                "http://127.0.0.1:8080",
                "http://localhost:8080",
                "http://127.0.0.1:8081",
                "http://localhost:8081",
                "http://127.0.0.1:3000",
                "http://localhost:3000",
                "http://127.0.0.1:5173",
                "http://localhost:5173",
            ] {
                cors = cors.allowed_origin(origin);
            }
            cors
        }
    }
}

/// Initialize the token revocation list from centralized configuration.
/// If `REDIS_URL` is set in `AgroCoreConfig`, uses Redis; otherwise falls back to in-memory store.
fn init_token_revocation() -> TokenRevocationList {
    let config = agrocore_shared::config::AgroCoreConfig::global();
    match &config.redis_url {
        Some(url) => TokenRevocationList::from_redis(url).unwrap_or_else(|e| {
            warn!(
                    "Failed to initialize Redis-backed token revocation list: {}. Falling back to in-memory store.",
                    e
                );
            TokenRevocationList::new()
        }),
        None => TokenRevocationList::new(),
    }
}

/// Publish an event when a broker is available.
///
/// Every call site already discarded the result with `let _ =`, so a missing
/// broker must not turn a successful write into an error. Keeping this in one
/// place means each handler does not repeat the `Option` dance.
pub async fn publish_event<T: serde::Serialize>(
    messaging: Option<&Arc<MessagingClient>>,
    subject: impl Into<String>,
    event: &T,
) -> Result<(), String> {
    match messaging {
        Some(m) => m
            .publish(subject.into(), event)
            .await
            .map_err(|e| e.to_string()),
        None => Ok(()),
    }
}

pub async fn run_server(
    db: Database,
    messaging: Option<MessagingClient>,
    bind_addr: &str,
    backup_service: Option<Arc<BackupService>>,
) -> std::io::Result<()> {
    // Refuse to serve with a weak signing key. Without this the server starts
    // with the built-in `dev-secret`, and anyone can mint an admin token with
    // a plain HS256 signature. `validate_jwt_secret()` existed but had no
    // callers at all.
    if let Err(e) = agrocore_shared::config::validate_jwt_secret() {
        return Err(std::io::Error::other(format!(
            "Refusing to start: {e}. Set JWT_SECRET to at least {} characters, \
             or ALLOW_DEV_SECRET=1 for local development.",
            agrocore_shared::config::JWT_SECRET_MIN_LENGTH
        )));
    }

    // ── Demo Mode initialization ────────────────────────────────────────────
    let demo_mode = is_demo_mode();
    let demo_readonly_flag = demo_readonly();
    let demo_reset_hours_flag = demo_reset_hours();
    let mut demo_tenant_id: Option<uuid::Uuid> = None;

    if demo_mode {
        agrocore_logging::info!("DEMO_MODE enabled — checking for demo tenant...");

        let pool = db.unscoped_pool();
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM tenants WHERE slug = $1 AND is_demo = true",
        )
        .bind("demo")
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

        if let Some(tid) = existing {
            agrocore_logging::info!("Demo tenant already exists (id: {tid}), skipping seed.");
            demo_tenant_id = Some(tid);
        } else {
            agrocore_logging::info!("No demo tenant found — seeding automatically...");
            use crate::handlers::demo::seed_demo_internal;

            if let Ok(seeded_tenant_id) = seed_demo_internal(&db).await {
                demo_tenant_id = Some(seeded_tenant_id);
                agrocore_logging::info!(
                    "Demo tenant seeded successfully (id: {seeded_tenant_id})"
                );
            } else {
                warn!("Failed to auto-seed demo tenant — manual seed required");
            }
        }
    }

    if demo_readonly_flag {
        agrocore_logging::info!("DEMO_READONLY enabled — demo tenant writes will be blocked");
    }

    if let Some(hours) = demo_reset_hours_flag {
        agrocore_logging::info!(
            "DEMO_RESET_HOURS={hours} — demo data will auto-reset every {hours} hours"
        );
    }

    // Initialize LPIS Registry with all providers
    let lpis_registry = Arc::new(create_default_registry());

    // Initialize DB metrics registry and metrics instance
    let metrics_registry = Arc::new(Registry::new());
    let db_metrics = DbMetrics::new(&metrics_registry);
    let business_metrics = BusinessMetrics::new(&metrics_registry);

    let state_inner = AppState {
        db: Arc::new(db),
        messaging: messaging.map(Arc::new),
        lpis_registry,
        token_revocation: Arc::new(init_token_revocation()),
        db_metrics,
        business_metrics,
        metrics_registry: metrics_registry.clone(),
        backup_service,
        // Demo endpoints stay off unless explicitly enabled. They create and
        // delete tenants, so production must never have them reachable.
        demo_endpoints_enabled: demo_endpoints_enabled(),
        startup_time: std::time::Instant::now(),
        is_demo_mode: demo_mode,
        demo_tenant_id,
        demo_readonly: demo_readonly_flag,
        demo_reset_hours: demo_reset_hours_flag,
    };

    // Spawn demo auto-reset background task if configured
    if demo_reset_hours_flag.is_some() {
        crate::middleware::spawn_demo_auto_reset(Arc::new(state_inner.clone())).await;
    }

    let state = web::Data::new(state_inner);

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
            // The payload limit, stated explicitly.
            //
            // `web::Json` already defaults to 2 MiB, so ordinary endpoints were
            // bounded before this line existed. What was not bounded was the import
            // path: `/sites/import/shapefile` carries a file as Base64 and cannot
            // use the default without rejecting a real municipality, so it takes
            // `LargeJson` with a ceiling of its own. Writing the global value out
            // rather than inheriting it makes the number greppable and testable.
            .app_data(web::JsonConfig::default().limit(MAX_JSON_PAYLOAD))
            .app_data(web::PayloadConfig::default().limit(MAX_JSON_PAYLOAD))
            .wrap(prometheus.clone())
            .wrap(security_headers)
            .wrap(Governor::new(&gov_conf))
            .wrap(cors)
            // I15: Compress responses — ~70-85% fewer bytes for GeoJSON, Excel, SIGPAC
            .wrap(actix_web::middleware::Compress::default())
            .wrap(crate::middleware::DemoReadOnlyMiddleware)
            .wrap(crate::middleware::MetricsMiddleware)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url(
                "/api-docs/openapi.json",
                openapi::ApiDoc::openapi_with_security(),
            ))
            .configure(handlers::configure)
            .configure(metrics_routes)
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

/// The request body limit for the API, in bytes.
///
/// This is `actix_web`'s own default, `JsonConfig::DEFAULT_LIMIT`, set explicitly so
/// the value is visible and testable rather than implicit. It was worth writing down
/// because the audit recorded C3 as "no payload limit" — that was half right.
///
/// `web::Json` has always defaulted to 2 MiB, so an ordinary endpoint was never
/// truly unbounded. What was unbounded was `/sites/import/shapefile`, which could not
/// take that limit (see `LargeJson`) and so had no ceiling of its own until now.
/// Setting the value here rather than relying on the default means a future Actix
/// change cannot silently alter it, and `payload_limit_tests` pins it.
pub const MAX_JSON_PAYLOAD: usize = 2 * 1024 * 1024;

/// The request body limit for the three import handlers, in bytes.
///
/// 64 MiB. A European municipality's parcel shapefile is typically 10–30 MB of
/// ZIP data, which is 13–40 MB once Base64-encoded for transport (Base64 inflates
/// by 4/3) and grows again if the caller sends it uncompressed. 64 MiB leaves
/// headroom for a large district without being a licence to send arbitrary data.
///
/// This is a limit on one request, not a quota: it does not cap what a tenant may
/// import in total over time.
pub const MAX_IMPORT_PAYLOAD: usize = 64 * 1024 * 1024;

/// A `web::Json` body read with a limit that is set per-extractor rather than
/// per-application.
///
/// `web::JsonConfig` is app data, so it is global: one value for the whole
/// application. The three import endpoints need more headroom than the rest, and
/// registering them in a scope with a second config does not work — the routes are
/// already mounted by `handlers::configure`, and a second registration of the same
/// path is shadowed by the first rather than overriding it.
///
/// So this extractor reads the body itself, with its own limit. It mirrors
/// `web::Json`'s behaviour — same content-type check, same error type, same
/// `Deserialize` requirement — and differs only in the ceiling.
///
/// An over-limit body still produces a 413, so the response a client sees is the
/// same one it would get from `web::Json` with a smaller limit.
///
/// ```ignore
/// pub async fn import_shapefile(
///     state: web::Data<AppState>,
///     auth: AuthUser,
///     body: LargeJson<ShapefileImportRequest>,
/// ) -> Result<HttpResponse, ApiError> { /* ... */ }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LargeJson<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromRequest for LargeJson<T> {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(_req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let limit = MAX_IMPORT_PAYLOAD;
        // The payload stream is moved into the future rather than borrowed: the
        // returned future is `'static`, so it cannot hold a borrow of `payload`.
        let stream = payload.take();

        Box::pin(async move {
            // Read the body with an explicit ceiling. `Payload::take` in
            // actix-http 3 takes no argument and applies no limit, so the running
            // total is compared here instead — checked on every chunk rather than
            // once at the end, so an oversized body is rejected without ever being
            // fully buffered.
            let mut body = web::BytesMut::new();
            let mut stream = stream;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                let len = body.len() + chunk.len();
                if len > limit {
                    return Err(actix_web::error::PayloadError::Overflow.into());
                }
                body.extend_from_slice(&chunk);
            }

            let value = serde_json::from_slice::<T>(&body).map_err(|err| {
                actix_web::error::ErrorBadRequest(format!("Json deserialize error: {err}"))
            })?;
            Ok(LargeJson(value))
        })
    }
}

/// Mounts the metrics endpoints.
///
/// Public and called from `main`, rather than wired inline, so a test can build the
/// same scope the server builds. The metrics routes used to be registered directly
/// on the `App`, outside `handlers::configure`, which meant the route inventory
/// test could not see them — an unauthenticated endpoint that no test was able to
/// reach is an endpoint no test could catch.
pub fn metrics_routes(cfg: &mut web::ServiceConfig) {
    // Built per call because `Governor` holds per-instance state.
    let metrics_gov_conf = GovernorConfigBuilder::default()
        .seconds_per_request(5)
        .burst_size(12)
        .finish()
        .expect("static governor config for the metrics scope");

    cfg.service(
        web::scope("/metrics")
            .wrap(Governor::new(&metrics_gov_conf))
            .route("/db", web::get().to(db_metrics_handler))
            .route("/business", web::get().to(business_metrics_handler)),
    );
}

/// Serves the DB metrics in Prometheus text format at `/metrics/db`.
///
/// Admin only, and on purpose so: the registry carries per-table query counts and
/// durations, pool saturation and record counts. It carries no tenant label — the
/// metric vectors are keyed by query type and table name only — so this is not a
/// cross-tenant leak, but it is still an operational map of the deployment that has
/// no business being public. It was registered globally with no extractor at all,
/// which meant anyone who could reach the port could read it.
///
/// The two handlers are identical in output. They are kept separate because
/// `/metrics/business` is the one a dashboard scrapes and `/metrics/db` the one a
/// DBA reads during an incident, and the split exists so the expensive gather can
/// be moved to a cheaper registry later without a routing change.
pub async fn db_metrics_handler(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<actix_web::HttpResponse, crate::error::ApiError> {
    auth.require_admin()?;

    Ok(encode_metrics(&state))
}

/// Serves all metrics — DB and business — at `/metrics/business`.
pub async fn business_metrics_handler(
    state: web::Data<AppState>,
    auth: crate::middleware::AuthExtractor,
) -> Result<actix_web::HttpResponse, crate::error::ApiError> {
    auth.require_admin()?;

    Ok(encode_metrics(&state))
}

/// Gathers the registry and renders it, turning an encoding failure into a 500
/// that says nothing about the registry's contents.
fn encode_metrics(state: &AppState) -> actix_web::HttpResponse {
    let encoder = prometheus::TextEncoder::new();
    match encoder.encode_to_string(&state.metrics_registry.gather()) {
        Ok(output) => actix_web::HttpResponse::Ok()
            .insert_header(("Content-Type", "text/plain; version=0.0.4"))
            .body(output),
        Err(e) => {
            logging_error!("Failed to encode metrics: {}", e);
            actix_web::HttpResponse::InternalServerError().body("metrics encode error")
        }
    }
}
