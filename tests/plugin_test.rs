use anyhow::Result;
use async_trait::async_trait;
use filesynchub::plugins::{FileEvent, Plugin, PluginManager};
use std::path::PathBuf;
use std::sync::Arc;
use tokio;

struct TestPlugin {
    name: String,
}

impl TestPlugin {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&self, _event: FileEvent) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_registration() -> Result<()> {
    let plugin_manager = Arc::new(PluginManager::new());
    let test_plugin = TestPlugin::new("test_plugin".to_string());

    assert!(plugin_manager
        .register_plugin(Box::new(test_plugin))
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_plugin_event_handling() -> Result<()> {
    let plugin_manager = Arc::new(PluginManager::new());
    let test_plugin = TestPlugin::new("test_plugin".to_string());
    plugin_manager
        .register_plugin(Box::new(test_plugin))
        .await?;

    let test_path = PathBuf::from("test.txt");
    let event = FileEvent::Created(Arc::new(test_path));

    assert!(plugin_manager.handle_event(event).await.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_plugin_shutdown() -> Result<()> {
    let plugin_manager = Arc::new(PluginManager::new());
    let test_plugin = TestPlugin::new("test_plugin".to_string());
    plugin_manager
        .register_plugin(Box::new(test_plugin))
        .await?;

    assert!(plugin_manager.shutdown().await.is_ok());

    Ok(())
}
