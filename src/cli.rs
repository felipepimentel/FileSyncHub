use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional config file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start in sync mode without TUI
    Sync {
        /// Provider name to sync (if not specified, syncs all enabled providers)
        #[arg(short, long)]
        provider: Option<String>,
    },
}
