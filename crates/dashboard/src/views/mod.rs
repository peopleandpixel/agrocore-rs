//! views module — rendering functions for each TUI panel using SuperLightTUI.
//!
//! Layout inspired by SLT demo_dashboard: header + metric cards + stat_trend
//! + tables + sparklines. Organized into tabs for full dashboard navigation.

use crate::app::App;
use crate::services::build::BuildStatus;
use crate::services::log::{ApiLogEntry, SqlLogEntry};
use crate::services::system::SystemInfo;
use slt::{Border, Color, Context, TableState, Trend};

/// Top-level render entry point.
pub fn render(ui: &mut Context, app: &mut App) {
    let _ = ui
        .bordered(Border::Rounded)
        .title("AgroCore Dashboard")
        .p(1)
        .grow(1)
        .col(|ui| {
            render_header(ui, app);
            let _ = ui.divider_text("Dashboard");
            match app.current_tab {
                0 => render_services_tab(ui, app),
                1 => render_build_tab(ui, app),
                2 => render_git_tab(ui, app),
                3 => render_system_tab(ui, app),
                4 => render_api_log_tab(ui, app),
                5 => render_sql_log_tab(ui, app),
                _ => {}
            }
            render_controls(ui);
        });
}

/// Header: spinner + LIVE badge + uptime.
fn render_header(ui: &mut Context, app: &mut App) {
    let _ = ui.row(|ui| {
        let _ = ui.spinner(&app.spinner);
        ui.text(" LIVE").bold().fg(Color::Green);
        ui.spacer();
        let _ = ui
            .text(format!(
                "Uptime: {}d {}h {}m",
                app.uptime_seconds() / 86400,
                (app.uptime_seconds() % 86400) / 3600,
                (app.uptime_seconds() % 3600) / 60,
            ))
            .dim();
    });
}

/// Tab 0: Services — metric cards + detail table.
fn render_services_tab(ui: &mut Context, app: &mut App) {
    let s = &app.services;
    let entries = s.entries();

    let _ = ui.row(|ui| {
        for (name, status) in &entries {
            let is_up = status.is_up();
            metric_card(
                ui,
                name,
                if is_up { 100.0 } else { 0.0 },
                "",
                if is_up { Color::Green } else { Color::Red },
            );
        }
    });

    let _ = ui.divider_text("Service Details");

    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|(name, status)| {
            vec![
                (*name).to_string(),
                status.status_label().to_string(),
                status.detail.clone(),
            ]
        })
        .collect();

    app.table_cursor.set_rows(rows);

    let _ = ui
        .bordered(Border::Rounded)
        .title("Service Status")
        .p(1)
        .grow(1)
        .col(|ui| {
            let _ = ui.table(&mut app.table_cursor);
            let _ = ui.divider_text("Controls");
            let _ = ui.row(|ui| {
                if ui.button("Restart All").clicked {
                    app.restart_all();
                }
                ui.spacer();
                if ui.button("Stop All").clicked {
                    app.stop_all();
                }
                ui.spacer();
                let _ = ui.text("'s' toggle service menu | 'b' build check").dim();
            });
        });
}

/// Tab 1: Build status.
fn render_build_tab(ui: &mut Context, app: &mut App) {
    let _ = ui.row(|ui| {
        let _ = ui
            .bordered(Border::Rounded)
            .p(1)
            .grow(1)
            .col(|ui| match &app.build {
                BuildStatus::Ok => {
                    let _ = ui.stat("Build", "All green!");
                }
                BuildStatus::Failing(e) => {
                    let _ = ui.stat_colored("Build", &format!("Failing: {}", e), Color::Red);
                }
                BuildStatus::Running => {
                    let _ = ui.stat_colored("Build", "Running...", Color::Yellow);
                }
                BuildStatus::Unknown => {
                    let _ = ui.stat("Build", "Not started");
                }
            });
    });

    let _ = ui.divider_text("Git");
    let _ = ui
        .text(format!(
            "Branch: {} | Dirty: {}",
            app.git.branch, app.git.dirty
        ))
        .dim();
}

/// Tab 2: Git status.
fn render_git_tab(ui: &mut Context, app: &mut App) {
    let g = &app.git;
    let color = if g.dirty { Color::Red } else { Color::Green };

    let _ = ui
        .bordered(Border::Rounded)
        .title("Git Status")
        .p(1)
        .grow(1)
        .col(|ui| {
            let _ = ui.text(format!("Branch: {}", g.branch)).bold();
            let _ = ui
                .text(format!(
                    "Status: {}",
                    if g.dirty { "dirty" } else { "clean" }
                ))
                .fg(color);
        });

    if g.dirty {
        let _ = ui.divider_text("Changed Files");
        let _ = ui.bordered(Border::Rounded).p(1).grow(1).col(|ui| {
            for file in &g.changed_files {
                let _ = ui.text(file);
            }
        });
    } else {
        let _ = ui.text("No uncommitted changes").dim();
    }
}

/// Tab 3: System metrics.
fn render_system_tab(ui: &mut Context, app: &mut App) {
    let s = &app.system;
    let disk_pct = s.disk_percent();

    let _ = ui.divider_text("System Metrics");
    let _ = ui.row(|ui| {
        metric_card(ui, "CPU", s.cpu_percent as f64, "%", Color::Cyan);
        metric_card(ui, "Memory", s.mem_percent as f64, "%", Color::Yellow);
        metric_card(
            ui,
            "Disk",
            disk_pct as f64,
            "%",
            if disk_pct > 80.0 {
                Color::Red
            } else {
                Color::Green
            },
        );
    });

    let _ = ui.divider_text("Key Metrics");
    let _ = ui.row(|ui| {
        let _ = ui.bordered(Border::Rounded).p(1).grow(1).col(|ui| {
            let _ = ui.stat_trend("CPU", &format!("{:.1}%", s.cpu_percent), Trend::Up);
        });
        let _ = ui.bordered(Border::Rounded).p(1).grow(1).col(|ui| {
            let trend = if s.mem_percent > 50.0 {
                Trend::Up
            } else {
                Trend::Down
            };
            let _ = ui.stat_trend("Memory", &format!("{:.1}%", s.mem_percent), trend);
        });
    });

    let _ = ui.divider_text("Usage Trends");
    let _ = ui.row(|ui| {
        let cpu_data: Vec<f64> = app.cpu_history.iter().map(|&v| v as f64).collect();
        let mem_data: Vec<f64> = app.mem_history.iter().map(|&v| v as f64).collect();

        let _ = ui
            .bordered(Border::Rounded)
            .title("CPU")
            .p(1)
            .grow(1)
            .col(|ui| {
                let _ = ui.sparkline(&cpu_data, 40);
            });
        let _ = ui
            .bordered(Border::Rounded)
            .title("Memory")
            .p(1)
            .grow(1)
            .col(|ui| {
                let _ = ui.sparkline(&mem_data, 40);
            });
    });

    let _ = ui.divider_text("Disk");
    let _ = ui.row(|ui| {
        let _ = ui.bordered(Border::Rounded).p(1).grow(1).col(|ui| {
            let _ = ui.text(format!(
                "Disk: {} MB / {} MB ({}%)",
                s.disk_used / 1024,
                s.disk_total / 1024,
                s.disk_percent().round()
            ));
        });
    });
}

/// Tab 4: API call log — scrollable table of recent API calls.
fn render_api_log_tab(ui: &mut Context, app: &mut App) {
    let logs: &[ApiLogEntry] = &app.api_log.entries;
    let count = logs.len();
    let _ = ui.divider_text(&format!("API Call Log ({} entries)", count));

    if logs.is_empty() {
        let _ = ui
            .bordered(Border::Rounded)
            .title("No API calls yet")
            .p(1)
            .grow(1)
            .col(|ui| {
                let _ = ui.text("Waiting for API calls...").dim();
            });
        return;
    }

    let rows: Vec<Vec<String>> = logs
        .iter()
        .rev()
        .map(|e| {
            let status_color = if e.status >= 500 {
                format!("🔴 {}", e.status)
            } else if e.status >= 400 {
                format!("🟡 {}", e.status)
            } else {
                format!("🟢 {}", e.status)
            };
            vec![
                e.timestamp.format("%H:%M:%S").to_string(),
                e.method.clone(),
                e.path.clone(),
                status_color,
                format!("{}ms", e.latency_ms),
                e.client.clone(),
            ]
        })
        .collect();

    let mut table = TableState::new(
        vec![
            "Time".to_string(),
            "Method".to_string(),
            "Path".to_string(),
            "Status".to_string(),
            "Latency".to_string(),
            "Client".to_string(),
        ],
        rows,
    );
    let _ = ui
        .bordered(Border::Rounded)
        .title("Recent API Calls")
        .col(|ui| {
            let _ = ui.scrollable(&mut app.api_log_scroll).grow(1).col(|ui| {
                let _ = ui.table(&mut table);
            });
        });
}

/// Tab 5: SQL query log — scrollable table of recent SQL queries.
fn render_sql_log_tab(ui: &mut Context, app: &mut App) {
    let logs: &[SqlLogEntry] = &app.sql_log.entries;
    let count = logs.len();
    let _ = ui.divider_text(&format!("SQL Query Log ({} entries)", count));

    if logs.is_empty() {
        let _ = ui
            .bordered(Border::Rounded)
            .title("No SQL queries yet")
            .p(1)
            .grow(1)
            .col(|ui| {
                let _ = ui.text("Waiting for SQL queries...").dim();
            });
        return;
    }

    let rows: Vec<Vec<String>> = logs
        .iter()
        .rev()
        .map(|e| {
            let status = if e.success { "✅" } else { "❌" };
            let query_preview = if e.query.len() > 60 {
                format!("{}...", &e.query[..60])
            } else {
                e.query.clone()
            };
            vec![
                e.timestamp.format("%H:%M:%S").to_string(),
                query_preview,
                format!("{}ms", e.duration_ms),
                status.to_string(),
                e.db_label.clone(),
            ]
        })
        .collect();

    let mut table = TableState::new(
        vec![
            "Time".to_string(),
            "Query".to_string(),
            "Duration".to_string(),
            "OK".to_string(),
            "DB".to_string(),
        ],
        rows,
    );
    let _ = ui
        .bordered(Border::Rounded)
        .title("Recent SQL Queries")
        .col(|ui| {
            let _ = ui.scrollable(&mut app.sql_log_scroll).grow(1).col(|ui| {
                let _ = ui.table(&mut table);
            });
        });
}

/// Footer: Controls help bar.
fn render_controls(ui: &mut Context) {
    let _ = ui.help(&[
        ("Ctrl+Q", "quit"),
        ("1-6", "tabs"),
        ("s", "service menu"),
        ("b", "build check"),
        ("Tab", "focus"),
    ]);
}

/// Overlay: Service control menu (modal popup).
pub fn render_service_menu(ui: &mut Context, _app: &mut App) {
    let _ = ui.modal(|ui| {
        let _ = ui
            .bordered(Border::Rounded)
            .title("Service Control")
            .p(2)
            .col(|ui| {
                let _ = ui.text("  [r] Restart PostgreSQL");
                let _ = ui.text("  [n] Restart NATS");
                let _ = ui.text("  [m] Restart MQTT");
                let _ = ui.text("  [i] Restart Redis");
                let _ = ui.text("  [a] Restart All");
                let _ = ui.text("  [x] Stop All");
                let _ = ui.text("  [s] Close Menu");
            });
    });
}

/// Render a metric card: label + value + progress bar.
fn metric_card(ui: &mut Context, label: &str, value: f64, unit: &str, color: Color) {
    let _ = ui.bordered(Border::Single).p(1).grow(1).col(|ui| {
        let _ = ui.text(label).dim();
        let _ = ui.text(format!("{value:.1}{unit}")).bold().fg(color);
        let bar_w = 10;
        let filled = ((value / 100.0).clamp(0.0, 1.0) * bar_w as f64) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_w - filled);
        let _ = ui.text(bar).fg(color);
        if value > 80.0 {
            let _ = ui.badge_colored("HIGH", Color::Red);
        }
    });
}

/// Compute disk usage percentage.
trait SystemInfoExt {
    fn disk_percent(&self) -> f32;
}

impl SystemInfoExt for SystemInfo {
    fn disk_percent(&self) -> f32 {
        if self.disk_total > 0 {
            (self.disk_used as f32 / self.disk_total as f32) * 100.0
        } else {
            0.0
        }
    }
}
