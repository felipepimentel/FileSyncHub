use anyhow::Result;
use clap::{Parser, Subcommand};
use filesync::config::Config;
use filesync::{SyncService, Tui};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the service in daemon mode
    Daemon {
        /// Path to the configuration file
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Start the service in TUI mode
    Tui {
        /// Path to the configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    let config_path = match &cli.command {
        Some(Commands::Daemon { config }) => config.as_ref().unwrap_or(&cli.config),
        Some(Commands::Tui { config }) => config.as_ref().unwrap_or(&cli.config),
        None => &cli.config,
    };

    let config = Config::from_file(config_path).await?;

    match cli.command {
        Some(Commands::Daemon { .. }) => {
            let service = SyncService::new(config).await?;
            service.start().await?;
        }
        Some(Commands::Tui { .. }) | None => {
            let mut tui = Tui::new(config)?;
            tui.run()?;
        }
    }

    Ok(())
}
