use anyhow::Result;
use filesynchub::{
    config::Config,
    plugins::{google_drive::GoogleDrivePlugin, onedrive::OneDrivePlugin, PluginManager},
    service::Service,
};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load("examples/config/example_config.toml")?;
    config.validate()?;

    // Initialize plugin manager
    let plugin_manager = Arc::new(PluginManager::new());

    // Register plugins based on configuration
    if let Some(google_drive_config) = &config.plugins.get("google_drive") {
        let google_drive = GoogleDrivePlugin::new(google_drive_config.root_folder.clone());
        plugin_manager
            .register_plugin(Box::new(google_drive))
            .await?;
    }

    if let Some(onedrive_config) = &config.plugins.get("onedrive") {
        let onedrive = OneDrivePlugin::new(onedrive_config.root_folder.clone());
        plugin_manager.register_plugin(Box::new(onedrive)).await?;
    }

    // Create and start service
    let mut service = Service::new(config, plugin_manager);
    service.start().await?;

    println!("Service started successfully!");
    println!("Press Ctrl+C to stop the service");

    // Wait for Ctrl+C
    signal::ctrl_c().await?;

    // Stop service
    service.stop().await?;
    println!("Service stopped successfully!");

    Ok(())
}
