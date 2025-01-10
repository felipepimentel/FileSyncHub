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
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use crate::config::Config;

pub struct Tui {
    config: Config,
}

impl Tui {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main loop
        let res = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        // Return result
        res
    }

    async fn run_app<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ].as_ref())
                    .split(f.size());

                // Title
                let title = Paragraph::new(Text::raw("FileSyncHub"))
                    .style(Style::default().fg(Color::Green))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                // Provider list
                let items: Vec<ListItem> = self.config.providers
                    .iter()
                    .map(|p| {
                        let status = if p.enabled { "✓" } else { "✗" };
                        let text = format!("{} {} ({})", status, p.name, p.credentials.provider_type);
                        ListItem::new(Text::raw(text))
                    })
                    .collect();

                let providers = List::new(items)
                    .block(Block::default().title("Providers").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White));
                f.render_widget(providers, chunks[1]);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}
