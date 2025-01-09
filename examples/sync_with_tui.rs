use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use filesynchub::{
    plugins::{google_drive::GoogleDrivePlugin, onedrive::OneDrivePlugin, Plugin},
    tui::app::App,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{io::stdout, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create plugins
    let google_drive = GoogleDrivePlugin::new("FileSyncHub".to_string());
    let onedrive = OneDrivePlugin::new("FileSyncHub".to_string());

    // Create app state
    let mut app = App::new();
    app.add_status_message("Initializing plugins...");

    // Test connections
    app.add_status_message("Testing Google Drive connection...");
    if let Err(e) = google_drive.test_connection().await {
        app.add_status_message(&format!("Google Drive error: {}", e));
    } else {
        app.add_status_message("Google Drive connection successful!");
    }

    app.add_status_message("Testing OneDrive connection...");
    if let Err(e) = onedrive.test_connection().await {
        app.add_status_message(&format!("OneDrive error: {}", e));
    } else {
        app.add_status_message("OneDrive connection successful!");
    }

    // Main loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

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
            let messages =
                List::new(messages).block(Block::default().title("Status").borders(Borders::ALL));
            f.render_widget(messages, chunks[1]);

            // Help text
            let help = Paragraph::new("Press 'q' to quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
