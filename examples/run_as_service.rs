use anyhow::Result;
use filesynchub::{
    config::Config,
    plugins::{google_drive::GoogleDriveClient, onedrive::OneDrivePlugin, PluginManager},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load("config.toml")?;
    config.validate()?;

    // Initialize plugin manager
    let plugin_manager = Arc::new(PluginManager::new());

    // Register plugins
    if let Some(google_drive_config) = config.plugins.get("google_drive") {
        if google_drive_config.enabled {
            let google_drive = GoogleDriveClient::new(google_drive_config.root_folder.clone()).await?;
            plugin_manager
                .register_plugin(Box::new(google_drive))
                .await?;
            println!("Google Drive plugin registered");
        }
    }

    if let Some(onedrive_config) = config.plugins.get("onedrive") {
        if onedrive_config.enabled {
            let onedrive = OneDrivePlugin::new(onedrive_config.root_folder.clone());
            plugin_manager.register_plugin(Box::new(onedrive)).await?;
            println!("OneDrive plugin registered");
        }
    }

    println!("Service started. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    println!("Service stopped.");

    Ok(())
}
