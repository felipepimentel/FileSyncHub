use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::config::Config;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
}

impl Tui {
    pub fn new(config: Config) -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal, config })
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let result = self.run_app();

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        result
    }

    fn run_app(&mut self) -> io::Result<()> {
        loop {
            let config = self.config.clone();
            self.terminal.draw(move |f| {
                Self::render_ui(f, &config);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn render_ui(f: &mut Frame, config: &Config) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());

        // Title
        let title = Paragraph::new("FileSyncHub")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Providers List
        let providers: Vec<ListItem> = config
            .providers
            .iter()
            .map(|p| {
                let status = if p.enabled { "✓" } else { "✗" };
                let content = Line::from(vec![
                    Span::raw(format!("{} ", status)),
                    Span::styled(
                        &p.name,
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - "),
                    Span::raw(match p.credentials {
                        crate::config::ProviderCredentials::GoogleDrive(_) => "Google Drive",
                        crate::config::ProviderCredentials::OneDrive(_) => "OneDrive",
                    }),
                ]);
                ListItem::new(content)
            })
            .collect();

        let providers = List::new(providers)
            .block(Block::default().title("Providers").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        f.render_widget(providers, chunks[1]);

        // Help
        let help = Paragraph::new("Press 'q' to quit")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }
}
