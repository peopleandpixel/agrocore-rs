use crate::config::EnvironmentType;
use crate::error::{LoggingError, LoggingResult};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::Subscriber;
use tracing::field::Field;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use uuid::Uuid;

/// Service context attached to all spans
#[derive(Debug, Clone, Default)]
pub struct ServiceContext {
    pub service_name: String,
    pub environment: EnvironmentType,
    pub version: String,
    pub instance_id: String,
    pub metadata: HashMap<String, String>,
}

impl ServiceContext {
    pub fn new(service_name: impl Into<String>, environment: EnvironmentType) -> Self {
        Self {
            service_name: service_name.into(),
            environment,
            version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = id.into();
        self
    }

    /// Create a new span with service context
    pub fn span(&self, name: &str) -> tracing::Span {
        let span = tracing::info_span!(
            "service_operation",
            service = %self.service_name,
            environment = %format!("{:?}", self.environment),
            version = %self.version,
            instance_id = %self.instance_id,
            operation = %name,
        );

        for (k, v) in &self.metadata {
            span.record(k.as_str(), v.as_str());
        }

        span
    }
}

/// Request context for HTTP handlers
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub method: String,
    pub path: String,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
}

impl RequestContext {
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            ..Default::default()
        }
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_http(mut self, method: impl Into<String>, path: impl Into<String>) -> Self {
        self.method = method.into();
        self.path = path.into();
        self
    }

    pub fn with_remote(mut self, addr: impl Into<String>) -> Self {
        self.remote_addr = Some(addr.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Create a span with request context
    pub fn span(&self, name: &str) -> tracing::Span {
        let mut span = tracing::info_span!(
            "http_request",
            request_id = %self.request_id,
            operation = %name,
        );

        if let Some(tenant) = &self.tenant_id {
            span.record("tenant_id", tenant.as_str());
        }

        if let Some(user) = &self.user_id {
            span.record("user_id", user.as_str());
        }

        span.record("http.method", self.method.as_str());
        span.record("http.path", self.path.as_str());

        if let Some(addr) = &self.remote_addr {
            span.record("http.remote_addr", addr.as_str());
        }

        if let Some(ua) = &self.user_agent {
            span.record("http.user_agent", ua.as_str());
        }

        span
    }
}

/// Extension trait for adding structured fields to spans
pub trait SpanExt {
    fn record_error(&self, error: &dyn std::error::Error);
    fn record_latency(&self, micros: u64);
    fn record_db_query(&self, query: &str, duration_ms: u64, rows: Option<u64>);
    fn record_http_status(&self, status: u16);
    fn record_tenant(&self, tenant_id: &str);
    fn record_user(&self, user_id: &str);
}

impl SpanExt for tracing::Span {
    fn record_error(&self, error: &dyn std::error::Error) {
        self.record("error", true);
        self.record("error.message", error.to_string());
        if let Some(source) = error.source() {
            self.record("error.source", source.to_string());
        }
    }

    fn record_latency(&self, micros: u64) {
        self.record("latency_us", micros);
    }

    fn record_db_query(&self, query: &str, duration_ms: u64, rows: Option<u64>) {
        self.record("db.query", query);
        self.record("db.duration_ms", duration_ms);
        if let Some(r) = rows {
            self.record("db.rows", r);
        }
    }

    fn record_http_status(&self, status: u16) {
        self.record("http.status", status);
    }

    fn record_tenant(&self, tenant_id: &str) {
        self.record("tenant_id", tenant_id);
    }

    fn record_user(&self, user_id: &str) {
        self.record("user_id", user_id);
    }
}

/// Macro for creating a span with service + request context
#[macro_export]
macro_rules! agrocore_span {
    ($name:expr) => {
        tracing::info_span!($name)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}

/// Macro for structured info logging with context
#[macro_export]
macro_rules! agrocore_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// Macro for structured error logging with context
#[macro_export]
macro_rules! agrocore_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

/// Macro for structured warn logging with context
#[macro_export]
macro_rules! agrocore_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Macro for structured debug logging with context
#[macro_export]
macro_rules! agrocore_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

/// Thread-local service context for automatic span enrichment
thread_local! {
    static SERVICE_CONTEXT: std::cell::RefCell<Option<Arc<ServiceContext>>> = const { std::cell::RefCell::new(None) };
}

/// Set the service context for the current thread
pub fn set_service_context(ctx: Arc<ServiceContext>) {
    SERVICE_CONTEXT.with(|c| *c.borrow_mut() = Some(ctx));
}

/// Clear the service context
pub fn clear_service_context() {
    SERVICE_CONTEXT.with(|c| *c.borrow_mut() = None);
}

/// Get the current service context
pub fn get_service_context() -> Option<Arc<ServiceContext>> {
    SERVICE_CONTEXT.with(|c| c.borrow().clone())
}

/// Layer that automatically adds service context to all spans
pub struct ServiceContextLayer {
    context: Arc<ServiceContext>,
}

impl ServiceContextLayer {
    pub fn new(context: Arc<ServiceContext>) -> Self {
        Self { context }
    }
}

impl<S> tracing_subscriber::Layer<S> for ServiceContextLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            extensions.insert(self.context.clone());

            let mut visitor = ServiceContextVisitor(&self.context);
            attrs.record(&mut visitor);
        }
    }

    fn on_record(&self, id: &tracing::Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut visitor = ServiceContextVisitor(&self.context);
            values.record(&mut visitor);
        }
    }
}

struct ServiceContextVisitor<'a>(&'a Arc<ServiceContext>);

impl<'a> tracing::field::Visit for ServiceContextVisitor<'a> {
    fn record_f64(&mut self, _field: &Field, _value: f64) {}
    fn record_i64(&mut self, _field: &Field, _value: i64) {}
    fn record_u64(&mut self, _field: &Field, _value: u64) {}
    fn record_bool(&mut self, _field: &Field, _value: bool) {}
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "service"
            || field.name() == "environment"
            || field.name() == "version"
            || field.name() == "instance_id"
        {
            return;
        }
    }
    fn record_error(&mut self, _field: &Field, _value: &(dyn std::error::Error + 'static)) {}
    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

/// Create a new span with full context (service + request)
pub fn create_span(
    service_ctx: &ServiceContext,
    request_ctx: Option<&RequestContext>,
    name: &str,
) -> tracing::Span {
    let span = service_ctx.span(name);

    if let Some(req_ctx) = request_ctx {
        let req_span = req_ctx.span("request");
        span.follows_from(&req_span);
    }

    span
}
