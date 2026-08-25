//! AgroCore Live Dashboard — a Ratatui-based TUI showing real-time
//! service status, build state, git info, and system metrics.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    prelude::{Color, Style},
    widgets::{Block, Paragraph},
};
use std::time::{Duration, Instant};

mod app;
mod services;
mod views;

use app::App;

fn main() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut last_refresh = Instant::now();

    // Initial render
    terminal.draw(|f| {
        let area = f.area();
        let block = Block::bordered().title("AgroCore Dashboard");
        let inner = block.inner(area);
        f.render_widget(block, area);
        let txt = Paragraph::new("Loading services...").style(Style::default().fg(Color::Cyan));
        f.render_widget(txt, inner);
    })?;

    loop {
        // Handle events — 'q' to quit
        #[allow(clippy::collapsible_if)]
        if event::poll(Duration::from_millis(100))? {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Refresh every 2 seconds
        if last_refresh.elapsed() >= Duration::from_secs(2) {
            app.update_blocking();
            last_refresh = Instant::now();
        }

        terminal.draw(|f| views::render(f, &mut app))?;
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
