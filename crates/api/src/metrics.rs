use prometheus::{
    HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, Opts, Registry,
};

/// Toggle monitoring at runtime via env `AGROCORE_METRICS_ENABLED`.
pub fn is_enabled() -> bool {
    std::env::var("AGROCORE_METRICS_ENABLED")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Metrics registry for database query monitoring — integrates with Prometheus.
///
/// Provides:
/// - Query duration histograms (per-query-type, per-table)
/// - Pool active/idle connection gauges
/// - Slow-query detection via tracing warnings
#[derive(Clone)]
pub struct DbMetrics {
    pub query_duration_seconds: HistogramVec,
    pub pool_active_connections: IntGauge,
    pub pool_idle_connections: IntGauge,
    pub slow_query_count: IntCounterVec,
}

impl DbMetrics {
    pub fn new(registry: &Registry) -> Self {
        let query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "agrocore_db_query_duration_seconds",
                "Database query duration in seconds",
            )
            .buckets(prometheus::exponential_buckets(0.001, 2.0, 15).unwrap()),
            &["query_type", "table"],
        )
        .unwrap();
        registry
            .register(Box::new(query_duration_seconds.clone()))
            .unwrap();

        let pool_active_connections = IntGauge::with_opts(Opts::new(
            "agrocore_db_pool_active_connections",
            "Number of active pool connections (in-use)",
        ))
        .unwrap();
        registry
            .register(Box::new(pool_active_connections.clone()))
            .unwrap();

        let pool_idle_connections = IntGauge::with_opts(Opts::new(
            "agrocore_db_pool_idle_connections",
            "Number of idle pool connections (available)",
        ))
        .unwrap();
        registry
            .register(Box::new(pool_idle_connections.clone()))
            .unwrap();

        let slow_query_count = IntCounterVec::new(
            Opts::new(
                "agrocore_db_slow_queries_total",
                "Total number of slow queries (duration > threshold)",
            ),
            &["table"],
        )
        .unwrap();
        registry
            .register(Box::new(slow_query_count.clone()))
            .unwrap();

        Self {
            query_duration_seconds,
            pool_active_connections,
            pool_idle_connections,
            slow_query_count,
        }
    }

    /// Record a query execution and its duration.
    /// If duration exceeds `slow_threshold`, a tracing warning is emitted.
    pub fn record_query(
        &self,
        query_type: &str,
        table: &str,
        duration_ms: u128,
        slow_threshold_ms: u128,
    ) {
        if !is_enabled() {
            return;
        }
        let seconds = duration_ms as f64 / 1000.0;
        self.query_duration_seconds
            .with_label_values(&[query_type, table])
            .observe(seconds);
        if duration_ms >= slow_threshold_ms {
            self.slow_query_count.with_label_values(&[table]).inc();
            tracing::warn!(
                "Slow query detected: type={}, table={}, duration={}ms (threshold={:?}ms)",
                query_type,
                table,
                duration_ms,
                slow_threshold_ms,
            );
        }
    }

    /// Update pool connection gauges from sqlx Pool state.
    pub fn update_pool_metrics(&self, active: usize, idle: usize) {
        if !is_enabled() {
            return;
        }
        self.pool_active_connections.set(active as i64);
        self.pool_idle_connections.set(idle as i64);
    }
}

/// Slow-query detection threshold in milliseconds.
pub const SLOW_QUERY_THRESHOLD_MS: u128 = 1000;

/// Business-level metrics for device telemetry, inventory imports, and active device count.
#[derive(Clone)]
pub struct BusinessMetrics {
    pub active_devices: IntGauge,
    pub telemetry_messages_total: IntCounterVec,
    pub import_files_processed: IntCounter,
}

impl BusinessMetrics {
    pub fn new(registry: &Registry) -> Self {
        let active_devices = IntGauge::with_opts(Opts::new(
            "agrocore_business_active_devices",
            "Number of currently active devices (connected/sending telemetry)",
        ))
        .unwrap();
        registry.register(Box::new(active_devices.clone())).unwrap();

        let telemetry_messages_total = IntCounterVec::new(
            Opts::new(
                "agrocore_business_telemetry_messages_total",
                "Total telemetry messages received per device-type",
            ),
            &["device_type"],
        )
        .unwrap();
        registry
            .register(Box::new(telemetry_messages_total.clone()))
            .unwrap();

        let import_files_processed = IntCounter::with_opts(Opts::new(
            "agrocore_business_import_files_processed_total",
            "Total number of import files processed (SIGPAC, GeoJSON, Shapefile)",
        ))
        .unwrap();
        registry
            .register(Box::new(import_files_processed.clone()))
            .unwrap();

        Self {
            active_devices,
            telemetry_messages_total,
            import_files_processed,
        }
    }

    /// Increment the active-device gauge by `delta` (+1 on connect, -1 on disconnect).
    pub fn update_active_devices(&self, delta: i64) {
        self.active_devices.add(delta);
    }

    /// Increment the telemetry message counter for a given device type.
    pub fn record_telemetry(&self, device_type: &str) {
        self.telemetry_messages_total
            .with_label_values(&[device_type])
            .inc();
    }

    /// Increment the import-file processed counter.
    pub fn record_import_file(&self) {
        self.import_files_processed.inc();
    }
}

/// Helper to extract a table name from a SQL query string (best-effort).
/// Looks for common patterns like "FROM table", "INTO table", "UPDATE table", "DELETE FROM table".
pub fn extract_table_from_sql(sql: &str) -> &'static str {
    let lower = sql.to_lowercase();
    let keywords = ["from ", "into ", "update ", "delete from ", "join "];
    for kw in keywords {
        if let Some(pos) = lower.find(kw) {
            let after = &sql[pos + kw.len()..];
            let table = after.split_whitespace().next().unwrap_or("unknown");
            return match table {
                "sites" => "sites",
                "workers" => "workers",
                "users" => "users",
                "equipment" => "equipment",
                "animals" => "animals",
                "orders" => "orders",
                "tasks" => "tasks",
                "inventory_items" => "inventory_items",
                "financial_records" => "financial_records",
                "compliance_checklists" => "compliance_checklists",
                "weather_data" => "weather_data",
                "phenology_records" => "phenology_records",
                "water_sources" => "water_sources",
                "water_usage" => "water_usage",
                "cost_centers" => "cost_centers",
                "task_data" => "task_data",
                _ => "unknown",
            };
        }
    }
    "unknown"
}

/// Classify a SQL query into a high-level type (SELECT, INSERT, UPDATE, DELETE).
pub fn classify_query_type(sql: &str) -> &'static str {
    let trimmed = sql.trim_start().to_uppercase();
    if trimmed.starts_with("SELECT") {
        "select"
    } else if trimmed.starts_with("INSERT") {
        "insert"
    } else if trimmed.starts_with("UPDATE") {
        "update"
    } else if trimmed.starts_with("DELETE") {
        "delete"
    } else if trimmed.starts_with("WITH") {
        "cte"
    } else {
        "other"
    }
}

/// Measure and record SQL query duration via a macro.
/// Usage: `measure_sqlx_query!(&pool, &metrics, sqlx::query(sql), "select")`
#[macro_export]
macro_rules! measure_sqlx_query {
    ($pool:expr, $metrics:expr, $sql:expr, $query_type_override:expr) => {{
        let start = std::time::Instant::now();
        let result = $sql;
        let elapsed_ms = start.elapsed().as_millis();
        let qtype = $query_type_override;
        let table = $crate::metrics::extract_table_from_sql($sql);
        $metrics.record_query(
            qtype,
            table,
            elapsed_ms,
            $crate::metrics::SLOW_QUERY_THRESHOLD_MS,
        );
        result
    }};
}
