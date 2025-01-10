use anyhow::Result;
use clap::Parser;
use filesync::{config::Config, service::SyncService, tui::Tui};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Provider to sync (if not specified, syncs all enabled providers)
    #[arg(short, long)]
    provider: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_file(&cli.config).await?;

    if let Some(provider_name) = cli.provider {
        // Sync specific provider
        let provider = config.providers
            .iter()
            .find(|p| p.name == provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", provider_name))?;

        let mut service = SyncService::new(vec![provider.clone()]);
        service.start().await?;
    } else {
        // Start TUI mode
        let mut tui = Tui::new(config)?;
        tui.run().await?;
    }

    Ok(())
}
