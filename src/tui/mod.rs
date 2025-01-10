use std::io;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::config::Config;

pub struct Tui {
    config: Config,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new(config: Config) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { config, terminal })
    }

    pub fn run(&mut self) -> Result<()> {
        let res = self.run_app();

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }

        Ok(())
    }

    fn run_app(&mut self) -> Result<()> {
        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ].as_ref())
                    .split(f.size());

                let title = Paragraph::new(Line::from(vec![
                    Span::styled("FileSyncHub ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("- Press 'q' to quit, 's' to start syncing, 'p' to pause"),
                ]))
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                let providers: Vec<ListItem> = self.config.providers
                    .iter()
                    .map(|p| {
                        let mut lines = vec![Line::from(vec![
                            Span::styled(
                                format!("{} ", p.name),
                                Style::default().fg(Color::Yellow),
                            ),
                        ])];

                        lines.extend(p.mappings.iter().map(|m| {
                            Line::from(vec![
                                Span::raw("  "),
                                Span::styled(
                                    format!("{} → {}", m.local_path.display(), m.remote_path),
                                    Style::default().fg(Color::Blue),
                                ),
                            ])
                        }));

                        ListItem::new(lines)
                    })
                    .collect();

                let providers = List::new(providers)
                    .block(Block::default().title("Providers").borders(Borders::ALL));
                f.render_widget(providers, chunks[1]);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('s') => {
                        // TODO: Start syncing
                    }
                    KeyCode::Char('p') => {
                        // TODO: Pause syncing
                    }
                    _ => {}
                }
            }
        }
    }
}
