pub mod google_drive;
pub mod onedrive;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::{path::Path, sync::Arc};
use tokio::sync::RwLock;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn upload_chunk(&self, path: &Path, data: Bytes, offset: u64) -> Result<()>;
    async fn download_chunk(&self, path: &str, offset: u64, size: usize) -> Result<Bytes>;
    async fn delete_file(&self, path: &Path) -> Result<()>;
    fn clone_box(&self) -> Box<dyn Plugin>;
}

pub struct PluginManager {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.push(plugin);
        Ok(())
    }

    pub async fn handle_event(&self, path: &Path) -> Result<()> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.iter() {
            if let Err(e) = plugin.upload_chunk(path, Bytes::new(), 0).await {
                log::error!("Error in plugin {}: {}", plugin.name(), e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    struct MockPlugin {
        name: String,
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        async fn upload_chunk(&self, _path: &Path, _data: Bytes, _offset: u64) -> Result<()> {
            Ok(())
        }

        async fn download_chunk(&self, _path: &str, _offset: u64, _size: usize) -> Result<Bytes> {
            Ok(Bytes::new())
        }

        async fn delete_file(&self, _path: &Path) -> Result<()> {
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn Plugin> {
            Box::new(MockPlugin {
                name: self.name.clone(),
            })
        }
    }

    #[tokio::test]
    async fn test_plugin_manager() -> Result<()> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        let plugin_manager = PluginManager::new();
        let mock_plugin = MockPlugin {
            name: "mock".to_string(),
        };

        plugin_manager
            .register_plugin(Box::new(mock_plugin))
            .await?;
        plugin_manager.handle_event(&test_file).await?;

        Ok(())
    }
}
