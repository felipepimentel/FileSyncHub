use anyhow::Result;
use clap::{Parser, Subcommand};
use filesync::{Config, SyncService};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicia o serviço de sincronização
    Start,

    /// Adiciona um novo provedor
    AddProvider {
        /// Nome do provedor (ex: googledrive-main)
        name: String,
        /// Tipo do provedor (googledrive ou onedrive)
        #[arg(value_enum)]
        provider_type: ProviderType,
        /// Client ID do provedor
        client_id: String,
        /// Client Secret do provedor
        client_secret: String,
    },

    /// Adiciona um novo mapeamento de pasta para um provedor
    AddMapping {
        /// Nome do provedor
        provider: String,
        /// Caminho local para sincronizar
        local_path: PathBuf,
        /// Caminho remoto no provedor
        remote_path: String,
    },

    /// Lista todos os provedores configurados
    ListProviders,

    /// Lista todos os mapeamentos de um provedor
    ListMappings {
        /// Nome do provedor
        provider: String,
    },

    /// Remove um provedor
    RemoveProvider {
        /// Nome do provedor
        name: String,
    },

    /// Remove um mapeamento de pasta
    RemoveMapping {
        /// Nome do provedor
        provider: String,
        /// Caminho local do mapeamento
        local_path: PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum ProviderType {
    GoogleDrive,
    OneDrive,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            let config = Config::load()?;
            let mut service = SyncService::new(config).await?;
            println!("Iniciando serviço de sincronização...");
            service.start().await?;
            println!("Serviço iniciado. Pressione Ctrl+C para parar.");
            tokio::signal::ctrl_c().await?;
        }

        Commands::AddProvider {
            name,
            provider_type,
            client_id,
            client_secret,
        } => {
            let mut config = Config::load()?;
            
            // Verificar se já existe um provedor com este nome
            if config.find_provider(&name).is_some() {
                anyhow::bail!("Já existe um provedor com o nome: {}", name);
            }

            // Criar configuração do provedor
            let credentials = match provider_type {
                ProviderType::GoogleDrive => filesync::config::ProviderCredentials::GoogleDrive(
                    filesync::config::GoogleDriveCredentials {
                        client_id,
                        client_secret,
                        token: None,
                    },
                ),
                ProviderType::OneDrive => filesync::config::ProviderCredentials::OneDrive(
                    filesync::config::OneDriveCredentials {
                        client_id,
                        client_secret,
                        token: None,
                    },
                ),
            };

            config.providers.push(filesync::config::ProviderConfig {
                name,
                credentials,
                mappings: Vec::new(),
                enabled: true,
            });

            config.save()?;
            println!("Provedor adicionado com sucesso!");
        }

        Commands::AddMapping {
            provider,
            local_path,
            remote_path,
        } => {
            let mut config = Config::load()?;
            
            let provider_config = config
                .find_provider_mut(&provider)
                .ok_or_else(|| anyhow::anyhow!("Provedor não encontrado: {}", provider))?;

            // Verificar se o caminho local já está mapeado
            if provider_config
                .mappings
                .iter()
                .any(|m| m.local_path == local_path)
            {
                anyhow::bail!("Este caminho local já está mapeado: {:?}", local_path);
            }

            provider_config.mappings.push(filesync::config::FolderMapping {
                local_path,
                remote_path,
            });

            config.save()?;
            println!("Mapeamento adicionado com sucesso!");
        }

        Commands::ListProviders => {
            let config = Config::load()?;
            println!("Provedores configurados:");
            for provider in &config.providers {
                println!(
                    "- {} ({}) [{}]",
                    provider.name,
                    match provider.credentials {
                        filesync::config::ProviderCredentials::GoogleDrive(_) => "Google Drive",
                        filesync::config::ProviderCredentials::OneDrive(_) => "OneDrive",
                    },
                    if provider.enabled { "ativo" } else { "inativo" }
                );
            }
        }

        Commands::ListMappings { provider } => {
            let config = Config::load()?;
            let provider_config = config
                .find_provider(&provider)
                .ok_or_else(|| anyhow::anyhow!("Provedor não encontrado: {}", provider))?;

            println!("Mapeamentos do provedor {}:", provider);
            for mapping in &provider_config.mappings {
                println!(
                    "- Local: {:?} -> Remoto: {}",
                    mapping.local_path, mapping.remote_path
                );
            }
        }

        Commands::RemoveProvider { name } => {
            let mut config = Config::load()?;
            config.providers.retain(|p| p.name != name);
            config.save()?;
            println!("Provedor removido com sucesso!");
        }

        Commands::RemoveMapping { provider, local_path } => {
            let mut config = Config::load()?;
            let provider_config = config
                .find_provider_mut(&provider)
                .ok_or_else(|| anyhow::anyhow!("Provedor não encontrado: {}", provider))?;

            provider_config
                .mappings
                .retain(|m| m.local_path != local_path);

            config.save()?;
            println!("Mapeamento removido com sucesso!");
        }
    }

    Ok(())
}
