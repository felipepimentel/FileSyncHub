use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use filesync::config::Config;
use filesync::{SyncService, Tui};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the file synchronization service
    Start {
        /// Path to the configuration file
        #[arg(short, long, default_value = "config.toml")]
        config: String,

        /// Mode to run the service in
        #[arg(value_enum)]
        mode: Mode,
    },
}

#[derive(Clone, ValueEnum)]
enum Mode {
    /// Run in terminal UI mode
    Tui,
    /// Run in daemon mode
    Daemon,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, mode } => {
            let config = Config::from_file(&config).await?;

            match mode {
                Mode::Tui => {
                    let mut tui = Tui::new(config)?;
                    tui.run()?;
                }
                Mode::Daemon => {
                    let service = SyncService::new(config).await?;
                    service.start().await?;
                }
            }
        }
    }

    Ok(())
}
