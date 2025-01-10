use anyhow::Result;
use filesync::config::Config;
use filesync::tui::Tui;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_file("config.toml").await?;

    // Create and run TUI
    let mut tui = Tui::new(config)?;
    tui.run()?;

    Ok(())
}
