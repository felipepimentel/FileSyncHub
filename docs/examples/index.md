---
layout: default
title: Examples
nav_order: 4
has_children: true
---

# FileSyncHub Examples

This section provides practical examples and use cases for FileSyncHub.

## Basic Examples

### Simple Sync

Basic file synchronization with Google Drive:

```rust
use filesynchub::{FileSyncHub, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Create configuration
    let config = Config::builder()
        .sync_dir(PathBuf::from("~/Documents"))
        .add_plugin("google_drive", GoogleDriveConfig::default())
        .build()?;
    
    // Initialize FileSyncHub
    let mut sync_hub = FileSyncHub::new(config);
    sync_hub.init().await?;
    
    // Start syncing
    sync_hub.start().await?;
    
    Ok(())
}
```

### Multi-Service Sync

Sync with multiple cloud services:

```rust
use filesynchub::{Config, FileSyncHub};
use filesynchub::plugins::{GoogleDriveConfig, OneDriveConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure multiple services
    let config = Config::builder()
        .sync_dir("~/Documents")
        .add_plugin("google_drive", GoogleDriveConfig::default())
        .add_plugin("onedrive", OneDriveConfig::default())
        .build()?;
    
    // Initialize and start
    let mut sync_hub = FileSyncHub::new(config);
    sync_hub.init().await?;
    sync_hub.start().await?;
    
    Ok(())
}
```

## Advanced Examples

### Custom Plugin

Implementing a custom storage plugin:

```rust
use filesynchub::plugin::{SyncPlugin, FileInfo, Result};
use async_trait::async_trait;
use std::path::Path;

pub struct CustomStoragePlugin {
    config: PluginConfig,
    client: Client,
}

#[async_trait]
impl SyncPlugin for CustomStoragePlugin {
    async fn init(&mut self) -> Result<()> {
        self.client = Client::new(&self.config).await?;
        Ok(())
    }
    
    async fn upload_file(&self, path: &Path) -> Result<()> {
        let data = tokio::fs::read(path).await?;
        self.client.upload(path, data).await?;
        Ok(())
    }
    
    async fn download_file(&self, path: &Path) -> Result<()> {
        let data = self.client.download(path).await?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }
    
    async fn list_files(&self) -> Result<Vec<FileInfo>> {
        let files = self.client.list().await?;
        Ok(files.into_iter().map(FileInfo::from).collect())
    }
}
```

### Event Handling

Custom event handling implementation:

```rust
use filesynchub::events::{Event, EventHandler};
use async_trait::async_trait;

pub struct CustomEventHandler {
    notifier: Notifier,
}

#[async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_event(&self, event: Event) -> Result<()> {
        match event {
            Event::FileCreated(path) => {
                self.notifier.notify(format!("File created: {}", path.display())).await?;
            }
            Event::FileModified(path) => {
                self.notifier.notify(format!("File modified: {}", path.display())).await?;
            }
            Event::FileDeleted(path) => {
                self.notifier.notify(format!("File deleted: {}", path.display())).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Integration Examples

### Command Line Tool

Creating a CLI application:

```rust
use clap::{App, Arg, SubCommand};
use filesynchub::{Config, FileSyncHub};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("sync-tool")
        .subcommand(SubCommand::with_name("sync")
            .arg(Arg::with_name("dir")
                .takes_value(true)
                .required(true)))
        .get_matches();
    
    if let Some(matches) = matches.subcommand_matches("sync") {
        let dir = matches.value_of("dir").unwrap();
        
        let config = Config::builder()
            .sync_dir(dir)
            .add_plugin("google_drive", GoogleDriveConfig::default())
            .build()?;
        
        let mut sync_hub = FileSyncHub::new(config);
        sync_hub.init().await?;
        sync_hub.start().await?;
    }
    
    Ok(())
}
```

### GUI Application

Integration with a GUI framework:

```rust
use eframe::{egui, epi};
use filesynchub::{Config, FileSyncHub};

struct SyncApp {
    sync_hub: FileSyncHub,
    status: String,
}

impl epi::App for SyncApp {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FileSyncHub GUI");
            
            if ui.button("Start Sync").clicked() {
                let sync_hub = self.sync_hub.clone();
                tokio::spawn(async move {
                    if let Err(e) = sync_hub.start().await {
                        eprintln!("Sync error: {}", e);
                    }
                });
            }
            
            ui.label(&self.status);
        });
    }
}
```

## Configuration Examples

### Advanced Configuration

Complex configuration setup:

```rust
use filesynchub::{Config, SyncConfig, NetworkConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::builder()
        // Sync configuration
        .sync_config(SyncConfig {
            interval: Duration::from_secs(300),
            ignore_patterns: vec!["*.tmp", "*.log"],
            max_file_size: Some(100 * 1024 * 1024), // 100MB
        })
        // Network configuration
        .network_config(NetworkConfig {
            max_concurrent_transfers: 4,
            timeout: Duration::from_secs(30),
            retry_count: 3,
        })
        // Plugin configurations
        .add_plugin("google_drive", GoogleDriveConfig {
            root_folder: "Sync",
            chunk_size: 16 * 1024 * 1024, // 16MB
        })
        .build()?;
    
    let mut sync_hub = FileSyncHub::new(config);
    sync_hub.init().await?;
    sync_hub.start().await?;
    
    Ok(())
}
```

### Environment Configuration

Using environment variables:

```rust
use filesynchub::{Config, EnvConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up environment variables
    env::set_var("FILESYNCHUB_SYNC_DIR", "~/Documents");
    env::set_var("FILESYNCHUB_LOG_LEVEL", "debug");
    env::set_var("FILESYNCHUB_GOOGLE_DRIVE_CLIENT_ID", "your-client-id");
    
    // Load configuration from environment
    let config = EnvConfig::load()?;
    
    let mut sync_hub = FileSyncHub::new(config);
    sync_hub.init().await?;
    sync_hub.start().await?;
    
    Ok(())
}
```

## Testing Examples

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_file_upload() -> Result<()> {
        let config = TestConfig::default();
        let mut plugin = CustomStoragePlugin::new(config);
        
        // Initialize plugin
        plugin.init().await?;
        
        // Create test file
        let test_file = tempfile::NamedTempFile::new()?;
        tokio::fs::write(&test_file, b"test data").await?;
        
        // Test upload
        plugin.upload_file(test_file.path()).await?;
        
        // Verify upload
        let files = plugin.list_files().await?;
        assert!(files.iter().any(|f| f.path == test_file.path()));
        
        Ok(())
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_sync_workflow() -> Result<()> {
        // Set up test environment
        let temp_dir = tempfile::tempdir()?;
        let config = Config::builder()
            .sync_dir(temp_dir.path())
            .add_plugin("test", TestPluginConfig::default())
            .build()?;
        
        // Initialize FileSyncHub
        let mut sync_hub = FileSyncHub::new(config);
        sync_hub.init().await?;
        
        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;
        
        // Start sync
        sync_hub.start().await?;
        
        // Wait for sync to complete
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Verify sync
        let files = sync_hub.list_files().await?;
        assert!(files.iter().any(|f| f.path == test_file));
        
        Ok(())
    }
}
``` 