//! AgroCore Live Dashboard — a SuperLightTUI-based TUI showing real-time
//! service status, build state, git info, system metrics, and live logs.

use agrocore_dashboard::app::App;
use agrocore_dashboard::services::build::BuildStatus;
use agrocore_dashboard::views;
use slt::{Context, KeyCode, KeyModifiers, RunConfig};
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    let mut app = App::new();
    let refresh_interval = Duration::from_secs(2);
    let mut last_refresh = Instant::now();

    slt::run_with(RunConfig::default().mouse(true), |ui: &mut Context| {
        // Handle global key events
        if ui.key_mod('q', KeyModifiers::CONTROL) || ui.key_code(KeyCode::Esc) {
            ui.quit();
        }
        if ui.key('1') || ui.key_code(KeyCode::Tab) {
            app.current_tab = 0;
        }
        if ui.key('2') {
            app.current_tab = 1;
        }
        if ui.key('3') {
            app.current_tab = 2;
        }
        if ui.key('4') {
            app.current_tab = 3;
        }
        if ui.key('5') {
            app.current_tab = 4;
        }
        if ui.key('6') {
            app.current_tab = 5;
        }

        // Toggle service control menu
        if ui.key('s') {
            app.show_service_menu = !app.show_service_menu;
        }

        // Service control hotkeys (when menu is open)
        if app.show_service_menu {
            if ui.key('r') {
                app.restart_service("postgres");
                app.show_service_menu = false;
            }
            if ui.key('n') {
                app.restart_service("nats");
                app.show_service_menu = false;
            }
            if ui.key('m') {
                app.restart_service("mqtt");
                app.show_service_menu = false;
            }
            if ui.key('i') {
                app.restart_service("redis");
                app.show_service_menu = false;
            }
            if ui.key('a') {
                app.restart_all();
                app.show_service_menu = false;
            }
            if ui.key('x') {
                app.stop_all();
                app.show_service_menu = false;
            }
        }

        // Trigger build check on 'b'
        if ui.key('b') {
            app.build = BuildStatus::Running;
            app.build_pending = true;
        }

        // If a build was triggered, run the check (blocking — but quick)
        if app.build_pending {
            app.build = BuildStatus::run_check();
            app.build_pending = false;
        }

        // Auto-refresh data every 2 seconds
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh();
            last_refresh = Instant::now();
        }

        // Render the dashboard
        views::render(ui, &mut app);

        // Toast notifications overlay
        ui.toast(&mut app.toast_state);

        // Service control overlay
        if app.show_service_menu {
            views::render_service_menu(ui, &mut app);
        }
    })
}
