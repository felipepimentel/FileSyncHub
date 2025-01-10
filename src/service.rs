use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashMap;
use crate::{
    config::ProviderConfig,
    provider::{factory, CloudProvider, ChangeType, RemoteItem},
    sync::SyncOperation,
};

pub struct SyncService {
    providers: Vec<ProviderConfig>,
    active_providers: HashMap<String, Box<dyn CloudProvider>>,
}

impl SyncService {
    pub fn new(providers: Vec<ProviderConfig>) -> Self {
        Self {
            providers,
            active_providers: HashMap::new(),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        // Initialize providers
        for provider in &self.providers {
            if provider.enabled {
                println!("Starting sync for provider: {}", provider.name);
                let mut provider_instance = factory::create_provider(provider).await?;
                provider_instance.initialize().await?;
                
                // Set up change monitoring channels
                let (local_tx, mut local_rx) = mpsc::channel(100);
                let (remote_tx, mut remote_rx) = mpsc::channel(100);

                // Get provider mappings
                let mappings = provider_instance.get_mappings().await;

                // Create sync operation handler
                let sync_op = SyncOperation::new(provider_instance);
                let sync_op_clone = SyncOperation::new(factory::create_provider(provider).await?);

                // Start monitoring for each mapping
                for mapping in &mappings {
                    let local_tx = local_tx.clone();
                    let remote_tx = remote_tx.clone();
                    let mapping_clone = mapping.clone();
                    let provider_instance = sync_op.provider.as_ref();

                    // Monitor local changes
                    tokio::spawn(async move {
                        if let Err(e) = provider_instance.watch_local_changes(&mapping_clone.local_path, local_tx).await {
                            eprintln!("Error watching local changes: {}", e);
                        }
                    });

                    // Monitor remote changes
                    tokio::spawn(async move {
                        if let Err(e) = provider_instance.watch_remote_changes(&mapping_clone.remote_path, remote_tx).await {
                            eprintln!("Error watching remote changes: {}", e);
                        }
                    });
                }

                let mappings_clone = mappings.clone();

                // Handle local changes
                tokio::spawn(async move {
                    while let Some(change) = local_rx.recv().await {
                        for mapping in &mappings {
                            match &change {
                                ChangeType::Created(path) | ChangeType::Modified(path) => {
                                    if let Some(remote_path) = sync_op.get_remote_path(path, mapping) {
                                        let result = match change {
                                            ChangeType::Created(_) => {
                                                sync_op.handle_local_create(path, &remote_path).await
                                            }
                                            ChangeType::Modified(_) => {
                                                sync_op.handle_local_modify(path, &remote_path).await
                                            }
                                            _ => Ok(()),
                                        };

                                        if let Err(e) = result {
                                            eprintln!("Error handling local change: {}", e);
                                        }
                                    }
                                }
                                ChangeType::Deleted(path) => {
                                    if let Some(remote_path) = sync_op.get_remote_path(path, mapping) {
                                        if let Err(e) = sync_op.handle_local_delete(&remote_path).await {
                                            eprintln!("Error handling local deletion: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                // Handle remote changes
                tokio::spawn(async move {
                    while let Some(item) = remote_rx.recv().await {
                        for mapping in &mappings_clone {
                            if let Err(e) = sync_op_clone.handle_remote_change(item.clone(), &mapping.local_path).await {
                                eprintln!("Error handling remote change: {}", e);
                            }
                        }
                    }
                });

                println!("Sync started for provider: {}", provider.name);
            }
        }
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Clean up resources and stop sync
        self.active_providers.clear();
        Ok(())
    }
}


