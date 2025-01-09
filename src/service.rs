use crate::{config::Config, plugins::PluginManager, watcher::FileWatcher};
use anyhow::Result;
use std::sync::Arc;

pub struct Service {
    config: Config,
    plugin_manager: Arc<PluginManager>,
    file_watcher: Option<FileWatcher>,
}

impl Service {
    pub fn new(config: Config, plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            config,
            plugin_manager,
            file_watcher: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting FileSyncHub service...");

        // Create file watcher
        let mut watcher = FileWatcher::new(self.config.root_dir.clone());

        // Start watching for events
        let plugin_manager = self.plugin_manager.clone();
        watcher
            .start(move |event| {
                let plugin_manager = plugin_manager.clone();
                async move {
                    if let Err(e) = plugin_manager.handle_event(&event.path).await {
                        log::error!("Error handling file event: {}", e);
                    }
                }
            })
            .await?;

        self.file_watcher = Some(watcher);
        log::info!("FileSyncHub service started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Stopping FileSyncHub service...");

        if let Some(mut watcher) = self.file_watcher.take() {
            watcher.stop().await?;
        }

        log::info!("FileSyncHub service stopped successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::google_drive::GoogleDrivePlugin;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_service() -> Result<()> {
        // Create temporary directories
        let temp_dir = tempdir()?;
        let root_dir = temp_dir.path().join("root");
        let temp_dir_path = temp_dir.path().join("temp");

        // Create test configuration
        let mut config = Config::new();
        config.root_dir = root_dir.clone();
        config.temp_dir = temp_dir_path;

        // Create plugin manager and register a test plugin
        let plugin_manager = Arc::new(PluginManager::new());
        let google_drive = GoogleDrivePlugin::new("test".to_string());
        plugin_manager
            .register_plugin(Box::new(google_drive))
            .await?;

        // Create and start service
        let mut service = Service::new(config, plugin_manager);
        service.start().await?;

        // Create a test file
        tokio::fs::create_dir_all(&root_dir).await?;
        let test_file = root_dir.join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        // Wait for events to be processed
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Stop service
        service.stop().await?;

        Ok(())
    }
}
