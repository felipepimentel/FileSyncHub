use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::config::{Config, ProviderConfig};
use crate::provider::{CloudProvider, ChangeType, RemoteItem, create_provider};

pub struct SyncService {
    config: Config,
    providers: HashMap<String, Box<dyn CloudProvider>>,
}

impl SyncService {
    pub async fn new(config: Config) -> Result<Self> {
        let mut providers = HashMap::new();

        for provider_config in &config.providers {
            if provider_config.enabled {
                let provider = create_provider(provider_config).await?;
                providers.insert(provider_config.name.clone(), provider);
            }
        }

        Ok(Self { config, providers })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Inicializa todos os provedores
        for (name, provider) in &mut self.providers {
            provider.initialize().await
                .with_context(|| format!("Failed to initialize provider: {}", name))?;
        }

        // Para cada provedor habilitado, inicia o monitoramento
        for provider_config in self.config.providers.iter().filter(|p| p.enabled) {
            if let Some(provider) = self.providers.get(&provider_config.name) {
                self.start_provider_sync(provider_config, provider.as_ref()).await?;
            }
        }

        Ok(())
    }

    async fn start_provider_sync(
        &self,
        config: &ProviderConfig,
        provider: &dyn CloudProvider,
    ) -> Result<()> {
        // Para cada mapeamento, inicia o monitoramento local e remoto
        for mapping in &config.mappings {
            let (local_tx, mut local_rx) = mpsc::channel(100);
            let (remote_tx, mut remote_rx) = mpsc::channel(100);

            // Clone necessário para o closure
            let provider_name = config.name.clone();
            let local_path = mapping.local_path.clone();
            let remote_path = mapping.remote_path.clone();
            let provider = self.providers[&provider_name].as_ref();

            // Monitora mudanças locais
            provider.watch_local_changes(&mapping.local_path, local_tx).await?;

            // Monitora mudanças remotas
            provider.watch_remote_changes(&mapping.remote_path, remote_tx).await?;

            // Processa mudanças locais
            tokio::spawn({
                let provider_name = provider_name.clone();
                let provider = self.providers[&provider_name].as_ref();
                async move {
                    while let Some(change) = local_rx.recv().await {
                        match change {
                            ChangeType::Created(path) => {
                                if let Err(e) = handle_local_create(provider, &path, &local_path, &remote_path).await {
                                    eprintln!("Error handling local create: {}", e);
                                }
                            }
                            ChangeType::Modified(path) => {
                                if let Err(e) = handle_local_modify(provider, &path, &local_path, &remote_path).await {
                                    eprintln!("Error handling local modify: {}", e);
                                }
                            }
                            ChangeType::Deleted(path) => {
                                if let Err(e) = handle_local_delete(provider, &path, &local_path, &remote_path).await {
                                    eprintln!("Error handling local delete: {}", e);
                                }
                            }
                        }
                    }
                }
            });

            // Processa mudanças remotas
            tokio::spawn({
                let provider_name = provider_name.clone();
                let provider = self.providers[&provider_name].as_ref();
                async move {
                    while let Some(item) = remote_rx.recv().await {
                        if let Err(e) = handle_remote_change(provider, &item, &local_path, &remote_path).await {
                            eprintln!("Error handling remote change: {}", e);
                        }
                    }
                }
            });
        }

        Ok(())
    }
}

async fn handle_local_create(
    provider: &dyn CloudProvider,
    path: &PathBuf,
    local_base: &PathBuf,
    remote_base: &str,
) -> Result<()> {
    let relative_path = path.strip_prefix(local_base)?;
    let remote_path = format!("{}/{}", remote_base, relative_path.display());

    if path.is_dir() {
        provider.create_directory(&remote_path).await?;
    } else {
        provider.upload_file(path, &remote_path).await?;
    }

    Ok(())
}

async fn handle_local_modify(
    provider: &dyn CloudProvider,
    path: &PathBuf,
    local_base: &PathBuf,
    remote_base: &str,
) -> Result<()> {
    if path.is_file() {
        let relative_path = path.strip_prefix(local_base)?;
        let remote_path = format!("{}/{}", remote_base, relative_path.display());
        provider.upload_file(path, &remote_path).await?;
    }
    Ok(())
}

async fn handle_local_delete(
    provider: &dyn CloudProvider,
    path: &PathBuf,
    local_base: &PathBuf,
    remote_base: &str,
) -> Result<()> {
    let relative_path = path.strip_prefix(local_base)?;
    let remote_path = format!("{}/{}", remote_base, relative_path.display());
    provider.delete(&remote_path).await?;
    Ok(())
}

async fn handle_remote_change(
    provider: &dyn CloudProvider,
    item: &RemoteItem,
    local_base: &PathBuf,
    remote_base: &str,
) -> Result<()> {
    let relative_path = item.path.strip_prefix(remote_base)
        .with_context(|| format!("Failed to strip remote base: {} from {}", remote_base, item.path))?;
    let local_path = local_base.join(relative_path);

    if item.is_dir {
        tokio::fs::create_dir_all(&local_path).await?;
    } else {
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        provider.download_file(&item.path, &local_path).await?;
    }

    Ok(())
}
