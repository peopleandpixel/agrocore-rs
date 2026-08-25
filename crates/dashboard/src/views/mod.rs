//! views module — rendering functions for each TUI panel.

use ratatui::{
    layout::{Constraint, Direction, Rect},
    prelude::{Alignment, Color, Frame, Line, Span, Style, Text},
    widgets::{Block, Paragraph},
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let right = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_services(f, left[0], app);
    render_build(f, left[1], app);
    render_git(f, right[0], app);
    render_system(f, right[1], app);

    // Service control overlay
    if app.show_service_menu {
        render_service_menu(f, area);
    }

    let footer_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);
    let footer = Paragraph::new(Text::from(vec![
        Line::from(Span::raw(" AgroCore Live Dashboard ")),
        Line::from(Span::raw(format!("Uptime: {}s ", app.uptime_seconds()))),
        Line::from(Span::raw(
            "Press 'q' to quit | 's' service menu | Auto-refresh: 2s",
        )),
    ]))
    .style(Style::default().bg(Color::Blue))
    .alignment(Alignment::Center);
    f.render_widget(footer, footer_area);
}

fn render_services(f: &mut Frame, area: Rect, app: &App) {
    let title = format!("Services (uptime: {}s)", app.uptime_seconds());
    let block = Block::bordered().title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let s = &app.services;
    let lines = vec![
        Line::from(Span::raw(format!("  PostgreSQL: {}", s.postgres.status()))),
        Line::from(Span::raw(format!("  NATS:       {}", s.nats.status()))),
        Line::from(Span::raw(format!("  MQTT:       {}", s.mqtt.status()))),
        Line::from(Span::raw(format!("  Redis:      {}", s.redis.status()))),
        Line::from(Span::raw(format!("  API:        {}", s.api.status()))),
        Line::from(Span::raw(format!("  Admin UI:   {}", s.admin_ui.status()))),
    ];
    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn render_build(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered().title("Build (cargo check)");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let build_line = match &app.build {
        crate::services::build::BuildStatus::Ok => "  All green!".to_string(),
        crate::services::build::BuildStatus::Failing(e) => {
            format!("  Failing: {}", e)
        }
        crate::services::build::BuildStatus::Running => "  Running...".to_string(),
        crate::services::build::BuildStatus::Unknown => "  Not started".to_string(),
    };

    let color = match &app.build {
        crate::services::build::BuildStatus::Ok => Color::Green,
        crate::services::build::BuildStatus::Failing(_) => Color::Red,
        _ => Color::Yellow,
    };

    let para = Paragraph::new(Text::from(Line::from(Span::raw(build_line))))
        .style(Style::default().fg(color));
    f.render_widget(para, inner);
}

fn render_git(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::bordered().title("Git Status");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let g = &app.git;
    let status_color = if g.dirty { Color::Red } else { Color::Green };
    let mut lines: Vec<Line> = vec![
        Line::from(Span::raw(format!("  Branch: {}", g.branch))),
        Line::from(Span::raw(format!(
            "  Status: {}",
            if g.dirty { "dirty" } else { "clean" }
        ))),
    ];

    if g.dirty {
        for file in &g.changed_files {
            lines.push(Line::from(Span::raw(format!("  {}", file))));
        }
    } else {
        lines.push(Line::from(Span::raw("  No uncommitted changes")));
    }

    let para = Paragraph::new(Text::from(lines)).style(Style::default().fg(status_color));
    f.render_widget(para, inner);
}

fn render_system(f: &mut Frame, area: Rect, app: &App) {
    let sys = &app.system;
    let block = Block::bordered().title("System Info");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Build sparklines from history
    let cpu_spark: Vec<u64> = app.cpu_history.iter().map(|v| (v / 5.0) as u64).collect();
    let mem_spark: Vec<u64> = app.mem_history.iter().map(|v| (v / 5.0) as u64).collect();

    let mem_used_mb = sys.used_mem as f64 / 1024.0 / 1024.0;
    let mem_total_mb = sys.total_mem as f64 / 1024.0 / 1024.0;

    // CPU bar (20 chars wide)
    let cpu_width = 20;
    let filled = ((sys.cpu_percent / 5.0) as usize).min(cpu_width);
    let cpu_bar = format!("{}{}", "█".repeat(filled), "░".repeat(cpu_width - filled));

    let lines: Vec<Line> = vec![
        Line::from(Span::raw(format!(
            "  CPU:     {:>5.1}% |{}|",
            sys.cpu_percent, cpu_bar
        ))),
        Line::from(Span::raw("  CPU Spark:")),
        Line::from(Span::raw(format!("  {}", sparkline_to_string(&cpu_spark)))),
        Line::from(Span::raw(format!(
            "  Memory:  {:>5.1}% |{:.0} MB / {:.0} MB|",
            sys.mem_percent, mem_used_mb, mem_total_mb
        ))),
        Line::from(Span::raw("  Mem Spark:")),
        Line::from(Span::raw(format!("  {}", sparkline_to_string(&mem_spark)))),
        Line::from(Span::raw(format!("  Disk:    {}", sys.disk_info()))),
    ];

    let para = Paragraph::new(Text::from(lines)).style(Style::default().fg(Color::Green));
    f.render_widget(para, inner);
}

/// Convert spark data to ASCII string representation using Unicode block characters.
fn sparkline_to_string(data: &[u64]) -> String {
    let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let max_val = data.iter().copied().max().unwrap_or(1).max(1);
    data.iter()
        .map(|v| {
            let idx = ((*v as f64 / max_val as f64) * (bars.len() - 1) as f64) as usize;
            bars[idx.min(bars.len() - 1)]
        })
        .collect::<String>()
}

fn render_service_menu(f: &mut Frame, area: Rect) {
    use ratatui::layout::Constraint as C;
    let popup = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([C::Length(14), C::Min(0)])
        .split(area);

    let menu_area = Rect::new(area.x + (area.width - 40) / 2, popup[0].y, 40, 12);

    let block = Block::bordered().title("Service Control");
    let inner = block.inner(menu_area);
    f.render_widget(block, menu_area);

    let lines = vec![
        Line::from(Span::raw("  [r] Restart PostgreSQL")),
        Line::from(Span::raw("  [n] Restart NATS")),
        Line::from(Span::raw("  [m] Restart MQTT")),
        Line::from(Span::raw("  [i] Restart Redis")),
        Line::from(Span::raw("  [a] Restart All")),
        Line::from(Span::raw("  [x] Stop All")),
        Line::from(Span::raw("  [s] Close Menu")),
    ];
    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}
