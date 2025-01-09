use anyhow::Result;
use clap::{Parser, Subcommand};
use filesynchub::{Config, FileSyncHub};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Caminho para o arquivo de configuração
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa uma nova configuração
    Init {
        /// Diretório raiz para sincronização
        #[arg(short, long)]
        root: PathBuf,
    },
    /// Inicia o serviço de sincronização
    Start {
        /// Executa em modo daemon
        #[arg(short, long)]
        daemon: bool,
    },
    /// Para o serviço de sincronização
    Stop,
    /// Força a sincronização de um arquivo ou diretório
    Sync {
        /// Caminho para sincronizar
        path: PathBuf,
    },
    /// Mostra estatísticas do sistema
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { root } => {
            println!("Inicializando configuração...");
            let mut config = Config::new();
            config.root_dir = root.clone();
            config.save(cli.config.to_str().unwrap())?;
            println!("Configuração inicializada com sucesso!");
        }
        Commands::Start { daemon: _ } => {
            let config = Config::load(cli.config.to_str().unwrap())?;
            let mut hub = FileSyncHub::new(config);
            hub.start().await?;
            println!("Serviço iniciado com sucesso!");
        }
        Commands::Stop => {
            let config = Config::load(cli.config.to_str().unwrap())?;
            let mut hub = FileSyncHub::new(config);
            hub.stop().await?;
            println!("Serviço parado com sucesso!");
        }
        Commands::Sync { path: _ } => {
            let _config = Config::load(cli.config.to_str().unwrap())?;
            // TODO: Implement sync for specific path
            println!("Sincronização manual não implementada ainda.");
        }
        Commands::Stats => {
            let _config = Config::load(cli.config.to_str().unwrap())?;
            // TODO: Implement stats
            println!("Estatísticas não implementadas ainda.");
        }
    }

    Ok(())
}
