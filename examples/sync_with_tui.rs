use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use filesynchub::{
    plugins::{google_drive::GoogleDriveClient, onedrive::OneDrivePlugin},
    tui::app::App,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::{io::stdout, time::Duration};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    app.add_status_message("🚀 Starting FileSyncHub...");

    // Initialize Google Drive
    app.add_status_message("\n📦 Google Drive Setup");
    app.add_status_message("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    app.add_status_message("Starting authentication process...");
    app.add_status_message("A browser window should open automatically.");
    app.add_status_message("If not, an authentication URL will appear shortly.");
    app.add_status_message("You have 60 seconds to complete the process.");
    
    match GoogleDriveClient::new("FileSyncHub".to_string()).await {
        Ok(_) => {
            app.add_status_message("✅ Google Drive authenticated successfully!");
        },
        Err(e) => {
            app.add_status_message("❌ Authentication failed!");
            app.add_status_message(&format!("Error: {}", e));
            if e.to_string().contains("timed out") {
                app.add_status_message("⏰ The process timed out (60s limit)");
                app.add_status_message("💡 Tip: Restart the app to try again");
            } else if e.to_string().contains("cancelled") {
                app.add_status_message("🛑 Process was cancelled");
                app.add_status_message("💡 Tip: Restart the app to try again");
            }
        }
    }
    app.add_status_message("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize OneDrive
    app.add_status_message("\n☁️ OneDrive Setup");
    app.add_status_message("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let onedrive = OneDrivePlugin::new("FileSyncHub".to_string());
    if let Err(e) = onedrive.test_connection().await {
        app.add_status_message("❌ Connection failed!");
        app.add_status_message(&format!("Error: {}", e));
    } else {
        app.add_status_message("✅ Connected successfully!");
    }
    app.add_status_message("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Main loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Min(10),    // Main content
                    Constraint::Length(3),  // Status bar
                ])
                .split(f.area());

            // Title bar with fancy styling
            let title = Paragraph::new(Text::styled(
                "FileSyncHub",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            ))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Cloud Storage Sync ")
                .title_alignment(Alignment::Center));
            f.render_widget(title, chunks[0]);

            // Main content area with messages
            let messages: Vec<ListItem> = app
                .status_messages()
                .iter()
                .map(|m| {
                    let style = if m.contains("❌") {
                        Style::default().fg(Color::Red)
                    } else if m.contains("✅") {
                        Style::default().fg(Color::Green)
                    } else if m.contains("💡") {
                        Style::default().fg(Color::Yellow)
                    } else if m.contains("⏰") {
                        Style::default().fg(Color::LightRed)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(m, style)
                    ]))
                })
                .collect();

            let messages = List::new(messages)
                .block(Block::default()
                    .title(" Status Log ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)))
                .style(Style::default().fg(Color::White));
            f.render_widget(messages, chunks[1]);

            // Status bar with help text
            let status = Paragraph::new(Text::styled(
                "Press 'q' to quit | Use ↑↓ to scroll | Ctrl+C to cancel authentication",
                Style::default().fg(Color::DarkGray)
            ))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(status, chunks[2]);

            // If there's an authentication URL, show it in a popup
            if let Some(msg) = app.status_messages().iter().find(|m| m.contains("https://")) {
                let area = centered_rect(80, 30, f.area());
                let url = msg.trim();
                let popup_text = format!(
                    "\n🔐 Authentication Required\n\n\
                    Please complete these steps:\n\n\
                    1. Copy this URL:\n   {}\n\n\
                    2. Open it in your browser\n\
                    3. Sign in with your Google account\n\
                    4. Grant the requested permissions\n\
                    5. Return here to continue\n\n\
                    ⏰ You have 60 seconds to complete this process",
                    url
                );
                
                let popup = Paragraph::new(Text::styled(
                    popup_text,
                    Style::default().fg(Color::Yellow)
                ))
                .block(Block::default()
                    .title(" Google Drive Authentication ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
                
                f.render_widget(Clear, area);
                f.render_widget(popup, area);
            }
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
