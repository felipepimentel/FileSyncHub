use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use super::app::{App, SyncState, Tab};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Tabs
                Constraint::Length(3), // Progress bar
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Help
            ]
            .as_ref(),
        )
        .split(f.size());

    draw_title(f, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    draw_progress(f, app, chunks[2]);

    match app.current_tab() {
        Tab::Status => draw_status_list(f, app, chunks[3]),
        Tab::Files => draw_file_list(f, app, chunks[3]),
        Tab::Logs => draw_logs(f, app, chunks[3]),
    }

    draw_help(f, chunks[4]);
}

fn draw_title<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let title = Paragraph::new("FileSyncHub")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let titles = vec!["Status", "Files", "Logs"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.current_tab() as usize)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn draw_progress<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let progress = app.current_progress();
    let label = if let Some(file) = &progress.current_file {
        format!(
            "{}/{} files ({:.1}MB/{:.1}MB) - {}",
            progress.files_processed,
            progress.total_files,
            progress.bytes_processed as f64 / 1_000_000.0,
            progress.total_bytes as f64 / 1_000_000.0,
            file
        )
    } else {
        "Waiting for changes...".to_string()
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("Sync Progress")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(label)
        .ratio(progress.progress_percent as f64 / 100.0);
    f.render_widget(gauge, area);
}

fn draw_status_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .sync_history()
        .iter()
        .map(|status| {
            let color = match status.state {
                SyncState::Success => Color::Green,
                SyncState::Error => Color::Red,
                SyncState::InProgress => Color::Yellow,
            };

            let header = Spans::from(vec![
                Span::styled(
                    format!("{} ", status.timestamp.format("%H:%M:%S")),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled("►", Style::default().fg(color)),
                Span::raw(" "),
            ]);

            let content = Spans::from(vec![Span::styled(
                &status.message,
                Style::default().fg(color),
            )]);

            ListItem::new(vec![header, content])
        })
        .collect();

    let status_list = List::new(items)
        .block(Block::default().title("Sync Status").borders(Borders::ALL))
        .start_corner(ratatui::layout::Corner::BottomLeft);

    f.render_widget(status_list, area);
}

fn draw_file_list<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .monitored_files()
        .iter()
        .map(|file| {
            let style = if file.is_syncing {
                Style::default().fg(Color::Yellow)
            } else if file.is_synced {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(vec![Spans::from(vec![
                Span::styled(&file.path, style),
                Span::raw(" "),
                Span::styled(
                    format!("({:.1}MB)", file.size as f64 / 1_000_000.0),
                    Style::default().fg(Color::Gray),
                ),
            ])])
        })
        .collect();

    let files_list = List::new(items)
        .block(
            Block::default()
                .title("Monitored Files")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(files_list, area);
}

fn draw_logs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs()
        .iter()
        .map(|log| {
            let style = match log.level {
                log::Level::Error => Style::default().fg(Color::Red),
                log::Level::Warn => Style::default().fg(Color::Yellow),
                log::Level::Info => Style::default().fg(Color::White),
                log::Level::Debug => Style::default().fg(Color::Gray),
                log::Level::Trace => Style::default().fg(Color::DarkGray),
            };

            ListItem::new(vec![Spans::from(vec![
                Span::styled(
                    format!("{} ", log.timestamp.format("%H:%M:%S")),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(&log.message, style),
            ])])
        })
        .collect();

    let logs_list = List::new(logs)
        .block(Block::default().title("Logs").borders(Borders::ALL))
        .start_corner(ratatui::layout::Corner::BottomLeft);

    f.render_widget(logs_list, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let text = vec![Spans::from(vec![
        Span::raw("Press "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to switch views • "),
        Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to refresh • "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to quit"),
    ])];

    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}
