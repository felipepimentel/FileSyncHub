use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional config file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run in progress display mode
    #[arg(long)]
    pub progress: bool,

    /// Run as a daemon/service
    #[arg(long)]
    pub daemon: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new configuration file
    Init {
        /// Directory to watch
        #[arg(short, long)]
        dir: PathBuf,
    },
    /// Test the configuration
    Test {
        /// Configuration file to test
        #[arg(short, long)]
        config: PathBuf,
    },
    /// Show sync status
    Status {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Force sync now
    Sync {
        /// Sync specific directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
