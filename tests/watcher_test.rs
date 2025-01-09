use anyhow::Result;
use filesynchub::plugins::PluginManager;
use filesynchub::watcher::FileWatcher;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio;

#[tokio::test]
async fn test_watcher_creation() -> Result<()> {
    let dir = tempdir()?;
    let watch_path = dir.path().to_path_buf();
    let plugin_manager = Arc::new(PluginManager::new());

    let watcher = FileWatcher::new(plugin_manager, vec![watch_path]);
    assert!(watcher.start().await.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_file_event_detection() -> Result<()> {
    let dir = tempdir()?;
    let watch_path = dir.path().to_path_buf();
    let plugin_manager = Arc::new(PluginManager::new());

    let watcher = FileWatcher::new(plugin_manager, vec![watch_path.clone()]);
    watcher.start().await?;

    // Create a test file
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "test content")?;

    // Give some time for the event to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}
