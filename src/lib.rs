pub mod config;
pub mod plugins;
pub mod service;
pub mod tui;
pub mod watcher;

// Re-export commonly used items
pub use crate::config::Config;
pub use crate::plugins::{Plugin, PluginManager};
pub use crate::service::Service;
pub use crate::watcher::FileWatcher;

use std::sync::Arc;

pub struct FileSyncHub {
    config: Config,
    service: Service,
    plugin_manager: Arc<PluginManager>,
}

impl FileSyncHub {
    pub fn new(config: Config) -> Self {
        let plugin_manager = Arc::new(PluginManager::new());
        Self {
            service: Service::new(config.clone(), Arc::clone(&plugin_manager)),
            config,
            plugin_manager,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.service.start().await
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.service.stop().await
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> anyhow::Result<()> {
        self.plugin_manager.register_plugin(plugin).await
    }
}
