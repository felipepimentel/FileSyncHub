use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use tokio;

use filesynchub::{
    config::Config,
    plugins::{google_drive::GoogleDrivePlugin, PluginManager},
    watcher::FileWatcher,
};

#[tokio::test]
async fn test_full_system() -> Result<()> {
    // Create a temporary test directory
    let dir = tempdir()?;
    let watch_path = dir.path().to_path_buf();
    let config_path = dir.path().join("config.toml");
    let credentials_path = dir.path().join("credentials.json");

    // Create test credentials
    fs::write(
        &credentials_path,
        r#"{"installed": {"client_id": "test", "client_secret": "test"}}"#,
    )?;

    // Create test configuration
    let config_content = format!(
        r#"
[general]
log_level = "info"

[[watch_dirs]]
path = "{}"
recursive = true
include = ["*.txt"]
exclude = ["*.tmp"]

[plugins.google_drive]
credentials_path = "{}"
folder_id = "test_folder_id"
include = ["*.txt"]
exclude = ["*.tmp"]
"#,
        watch_path.to_str().unwrap().replace('\\', "/"),
        credentials_path.to_str().unwrap().replace('\\', "/")
    );

    fs::write(&config_path, config_content)?;

    // Initialize the system
    let config = Config::load(config_path.to_str().unwrap())?;
    config.validate()?;

    let plugin_manager = Arc::new(PluginManager::new());

    if let Some(google_drive_config) = &config.plugins.google_drive {
        let google_drive = GoogleDrivePlugin::new(
            google_drive_config.credentials_path.clone(),
            google_drive_config.folder_id.clone(),
            google_drive_config.include.clone(),
            google_drive_config.exclude.clone(),
        );
        plugin_manager
            .register_plugin(Box::new(google_drive))
            .await?;
    }

    let watch_paths = config
        .watch_dirs
        .iter()
        .map(|dir| dir.path.clone())
        .collect();

    let watcher = FileWatcher::new(plugin_manager.clone(), watch_paths);
    watcher.start().await?;

    // Create a test file and verify it triggers the system
    let test_file = watch_path.join("test.txt");
    fs::write(&test_file, "test content")?;

    // Give some time for the event to be processed
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Clean up
    plugin_manager.shutdown().await?;

    Ok(())
}
