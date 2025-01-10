use std::sync::Arc;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::provider::{CloudProvider, ChangeType, factory};

pub struct SyncService {
    providers: Vec<Arc<Box<dyn CloudProvider>>>,
}

impl SyncService {
    pub async fn new(config: Config) -> Result<Self> {
        let mut providers = Vec::new();
        
        for provider_config in &config.providers {
            if provider_config.enabled {
                let provider = factory::create_provider(provider_config).await?;
                providers.push(Arc::new(provider));
            }
        }

        Ok(Self { providers })
    }

    pub async fn start(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);

        // Start watching for changes in each provider
        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let tx = tx.clone();

            tokio::spawn(async move {
                let mappings = provider.get_mappings().await;
                for mapping in mappings {
                    if let Err(e) = provider.watch_local_changes(&mapping.local_path, tx.clone()).await {
                        eprintln!("Error watching local changes: {}", e);
                    }
                }
            });
        }

        // Handle changes
        while let Some(change) = rx.recv().await {
            match change {
                ChangeType::Created(path) => {
                    println!("File created: {:?}", path);
                    // TODO: Handle file creation
                }
                ChangeType::Modified(path) => {
                    println!("File modified: {:?}", path);
                    // TODO: Handle file modification
                }
                ChangeType::Deleted(path) => {
                    println!("File deleted: {:?}", path);
                    // TODO: Handle file deletion
                }
            }
        }

        Ok(())
    }
}
