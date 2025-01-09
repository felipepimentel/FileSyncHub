pub mod app;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::stdout;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    pub fn run(&mut self, app: &mut app::App) -> Result<()> {
        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(f.area());

                // Title
                let title = Paragraph::new("FileSyncHub")
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                // Status messages
                let messages: Vec<ListItem> = app
                    .status_messages()
                    .iter()
                    .map(|m| ListItem::new(m.as_str()))
                    .collect();
                let messages = List::new(messages)
                    .block(Block::default().title("Status").borders(Borders::ALL));
                f.render_widget(messages, chunks[1]);

                // Help text
                let help = Paragraph::new("Press 'q' to quit")
                    .style(Style::default().fg(Color::Gray))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(help, chunks[2]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
