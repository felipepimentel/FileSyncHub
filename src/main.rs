use anyhow::Result;
use filesynchub::{
    config::Config,
    plugins::{google_drive::GoogleDriveClient, onedrive::OneDrivePlugin, PluginManager},
    service::Service,
    tui::{app::App, Tui},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <command> [options]", args[0]);
        println!("Commands:");
        println!("  service - Run as a background service");
        println!("  tui     - Run with terminal user interface");
        return Ok(());
    }

    match args[1].as_str() {
        "service" => run_service().await,
        "tui" => run_tui().await,
        _ => {
            println!("Unknown command: {}", args[1]);
            Ok(())
        }
    }
}

async fn run_service() -> Result<()> {
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
        }
    }

    if let Some(onedrive_config) = config.plugins.get("onedrive") {
        if onedrive_config.enabled {
            let onedrive = OneDrivePlugin::new(onedrive_config.root_folder.clone());
            plugin_manager.register_plugin(Box::new(onedrive)).await?;
        }
    }

    // Create and start service
    let mut service = Service::new(config, plugin_manager);
    service.start().await?;

    println!("Service started successfully!");
    println!("Press Ctrl+C to stop the service");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    // Stop service
    service.stop().await?;
    println!("Service stopped successfully!");

    Ok(())
}

async fn run_tui() -> Result<()> {
    // Initialize TUI
    let mut tui = Tui::new()?;
    let mut app = App::new();

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
            app.add_status_message("Google Drive plugin registered");
        }
    }

    if let Some(onedrive_config) = config.plugins.get("onedrive") {
        if onedrive_config.enabled {
            let onedrive = OneDrivePlugin::new(onedrive_config.root_folder.clone());
            plugin_manager.register_plugin(Box::new(onedrive)).await?;
            app.add_status_message("OneDrive plugin registered");
        }
    }

    // Create and start service
    let mut service = Service::new(config, plugin_manager);
    service.start().await?;
    app.add_status_message("Service started successfully");

    // Run TUI
    tui.run(&mut app)?;

    // Stop service
    service.stop().await?;
    app.add_status_message("Service stopped successfully");

    Ok(())
}
