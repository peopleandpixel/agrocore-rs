//! Prometheus metrics for the backup service.
//!
//! Exposes:
//!   * `backup_duration_seconds` — histogram of completed backup durations
//!   * `backup_size_bytes`       — histogram of backup payload sizes
//!   * `backup_success_total`   — counter of successful backups, by type
//!   * `backup_failed_total`    — counter of failed backups, by type
//!   * `backup_restore_total`   — counter of restores, by type and outcome
//!
//! The registry is process-global and lazily created so callers can record
//! without threading a handle through the call stack.

use prometheus::{Encoder, Histogram, HistogramOpts, IntCounterVec, Opts, Registry, TextEncoder};

/// Metric handles used across the service.
#[derive(Clone)]
pub struct BackupMetrics {
    registry: Registry,
    duration: Histogram,
    size: Histogram,
    success: IntCounterVec,
    failed: IntCounterVec,
    restore: IntCounterVec,
}

/// Global metrics instance, created on first use.
static METRICS: std::sync::OnceLock<BackupMetrics> = std::sync::OnceLock::new();

impl BackupMetrics {
    /// Build a fresh, empty metric set.
    fn new() -> Self {
        let registry = Registry::new();

        let duration = Histogram::with_opts(
            HistogramOpts::new(
                "backup_duration_seconds",
                "Wall-clock duration of a completed backup",
            )
            // 1s .. 1h in 12 buckets
            .buckets(vec![
                1.0, 5.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1200.0, 1800.0, 2700.0, 3600.0,
            ]),
        )
        .expect("valid histogram opts");

        let size = Histogram::with_opts(
            HistogramOpts::new("backup_size_bytes", "Size of the backup payload in bytes")
                // 1 MiB .. 100 GiB in 12 buckets
                .buckets(vec![
                    1_048_576.0,
                    10_485_760.0,
                    52_428_800.0,
                    104_857_600.0,
                    524_288_000.0,
                    1_073_741_824.0,
                    5_368_709_120.0,
                    10_737_418_240.0,
                    53_687_091_200.0,
                    107_374_182_400.0,
                ]),
        )
        .expect("valid histogram opts");

        let success = IntCounterVec::new(
            Opts::new(
                "backup_success_total",
                "Number of successful backups by backup type",
            ),
            &["backup_type"],
        )
        .expect("valid counter opts");

        let failed = IntCounterVec::new(
            Opts::new(
                "backup_failed_total",
                "Number of failed backups by backup type",
            ),
            &["backup_type"],
        )
        .expect("valid counter opts");

        let restore = IntCounterVec::new(
            Opts::new(
                "backup_restore_total",
                "Number of restore operations by outcome",
            ),
            &["outcome"],
        )
        .expect("valid counter opts");

        // Registration failures would mean a duplicate name, which cannot
        // happen for a fresh registry; log and continue with unregistered
        // metrics rather than aborting service startup.
        for (name, res) in [
            (
                "backup_duration_seconds",
                registry.register(Box::new(duration.clone())),
            ),
            (
                "backup_size_bytes",
                registry.register(Box::new(size.clone())),
            ),
            (
                "backup_success_total",
                registry.register(Box::new(success.clone())),
            ),
            (
                "backup_failed_total",
                registry.register(Box::new(failed.clone())),
            ),
            (
                "backup_restore_total",
                registry.register(Box::new(restore.clone())),
            ),
        ] {
            if let Err(e) = res {
                agrocore_logging::warn!("Failed to register metric {name}: {e}");
            }
        }

        Self {
            registry,
            duration,
            size,
            success,
            failed,
            restore,
        }
    }

    /// Access the process-wide metrics instance.
    pub fn get() -> &'static Self {
        METRICS.get_or_init(Self::new)
    }

    /// Record a successful backup.
    pub fn record_success(&self, backup_type: &str, duration_seconds: f64, size_bytes: u64) {
        self.duration.observe(duration_seconds);
        self.size.observe(size_bytes as f64);
        self.success.with_label_values(&[backup_type]).inc();
    }

    /// Record a failed backup.
    pub fn record_failure(&self, backup_type: &str) {
        self.failed.with_label_values(&[backup_type]).inc();
    }

    /// Record a restore attempt.
    pub fn record_restore(&self, succeeded: bool) {
        let outcome = if succeeded { "success" } else { "failure" };
        self.restore.with_label_values(&[outcome]).inc();
    }

    /// Render the metrics in Prometheus text exposition format.
    pub fn render(&self) -> String {
        let encoder = TextEncoder::new();
        let families = self.registry.gather();
        let mut buffer = Vec::new();
        if encoder.encode(&families, &mut buffer).is_err() {
            return String::new();
        }
        String::from_utf8(buffer).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increase_per_backup_type() {
        let m = BackupMetrics::new();

        m.record_success("database", 1.5, 2048);
        m.record_success("database", 2.5, 4096);
        m.record_success("config", 0.5, 128);
        m.record_failure("database");

        let text = m.render();
        assert!(
            text.contains(r#"backup_success_total{backup_type="database"} 2"#),
            "database successes missing:\n{text}"
        );
        assert!(
            text.contains(r#"backup_success_total{backup_type="config"} 1"#),
            "config success missing:\n{text}"
        );
        assert!(
            text.contains(r#"backup_failed_total{backup_type="database"} 1"#),
            "failure not counted:\n{text}"
        );
    }

    #[test]
    fn duration_and_size_are_observed() {
        let m = BackupMetrics::new();
        m.record_success("database", 12.0, 5_242_880);

        // Histograms expose _bucket/_sum/_count series; the rendered text
        // carries the base name plus labels, so match on the series names.
        let text = m.render();
        assert!(
            text.contains("backup_duration_seconds_count"),
            "duration histogram missing:\n{text}"
        );
        assert!(
            text.contains("backup_size_bytes_count"),
            "size histogram missing:\n{text}"
        );
        // One observation was recorded.
        assert!(
            text.contains("backup_duration_seconds_count 1"),
            "duration observation not counted:\n{text}"
        );
        assert!(
            text.contains("backup_size_bytes_count 1"),
            "size observation not counted:\n{text}"
        );
    }

    #[test]
    fn restore_outcomes_are_separated() {
        let m = BackupMetrics::new();
        m.record_restore(true);
        m.record_restore(false);
        m.record_restore(false);

        let text = m.render();
        assert!(
            text.contains(r#"backup_restore_total{outcome="success"} 1"#),
            "restore success missing:\n{text}"
        );
        assert!(
            text.contains(r#"backup_restore_total{outcome="failure"} 2"#),
            "restore failures missing:\n{text}"
        );
    }

    #[test]
    fn render_is_valid_utf8_and_non_empty() {
        let m = BackupMetrics::new();
        m.record_success("full", 1.0, 10);
        let text = m.render();
        assert!(!text.is_empty());
        assert!(text.contains("# HELP"));
    }
}
