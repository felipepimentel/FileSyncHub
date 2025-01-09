---
layout: default
title: API Documentation
nav_order: 3
has_children: true
---

# FileSyncHub API Documentation

Welcome to the FileSyncHub API documentation. This section provides detailed information about the public API for developers who want to integrate with or extend FileSyncHub.

## Core Traits

### SyncPlugin

The main trait for implementing cloud storage plugins:

```rust
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait SyncPlugin: Send + Sync {
    /// Initialize the plugin
    async fn init(&mut self) -> Result<()>;
    
    /// Upload a file to remote storage
    async fn upload_file(&self, path: &Path) -> Result<()>;
    
    /// Download a file from remote storage
    async fn download_file(&self, path: &Path) -> Result<()>;
    
    /// List files in remote storage
    async fn list_files(&self) -> Result<Vec<FileInfo>>;
    
    /// Clean up resources
    async fn cleanup(&mut self) -> Result<()>;
}
```

### EventHandler

Trait for handling file system events:

```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a file system event
    async fn handle_event(&self, event: Event) -> Result<()>;
    
    /// Handle multiple events in batch
    async fn handle_events(&self, events: Vec<Event>) -> Result<()>;
}
```

## Core Types

### FileInfo

Information about a file in the sync system:

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileInfo {
    /// File path relative to sync root
    pub path: PathBuf,
    
    /// File metadata
    pub metadata: FileMetadata,
    
    /// File status
    pub status: FileStatus,
}
```

### Event

File system events that trigger sync actions:

```rust
#[derive(Debug, Clone)]
pub enum Event {
    /// File created
    FileCreated(PathBuf),
    
    /// File modified
    FileModified(PathBuf),
    
    /// File deleted
    FileDeleted(PathBuf),
    
    /// File renamed
    FileRenamed {
        from: PathBuf,
        to: PathBuf,
    },
}
```

## Error Handling

### Error Types

Custom error types for different failure scenarios:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Configuration

### Config Types

Configuration structures for FileSyncHub:

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    
    /// Sync settings
    pub sync: SyncConfig,
    
    /// Plugin configurations
    pub plugins: HashMap<String, PluginConfig>,
}
```

## Plugin Development

### Creating a Plugin

Basic structure for implementing a new plugin:

```rust
use filesynchub::plugin::{SyncPlugin, Result};

pub struct MyPlugin {
    config: PluginConfig,
}

#[async_trait]
impl SyncPlugin for MyPlugin {
    async fn init(&mut self) -> Result<()> {
        // Initialize plugin
        Ok(())
    }
    
    async fn upload_file(&self, path: &Path) -> Result<()> {
        // Implement file upload
        Ok(())
    }
    
    // Implement other required methods...
}
```

## Usage Examples

### Basic Usage

```rust
use filesynchub::{FileSyncHub, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::load("config.toml")?;
    
    // Create FileSyncHub instance
    let mut sync_hub = FileSyncHub::new(config);
    
    // Initialize
    sync_hub.init().await?;
    
    // Start syncing
    sync_hub.start().await?;
    
    Ok(())
}
```

### Plugin Registration

```rust
use filesynchub::PluginRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    let mut registry = PluginRegistry::new();
    
    // Register custom plugin
    let plugin = MyPlugin::new(config);
    registry.register("my-plugin", Box::new(plugin));
    
    Ok(())
}
```

## API Stability

FileSyncHub follows semantic versioning:

- Major version changes (1.0.0 -> 2.0.0) may include breaking changes
- Minor version changes (1.0.0 -> 1.1.0) add functionality in a backward-compatible manner
- Patch version changes (1.0.0 -> 1.0.1) include backward-compatible bug fixes

### Stability Guarantees

- Public traits and types marked with `#[stable]` are guaranteed to maintain compatibility
- Items marked with `#[experimental]` may change in minor versions
- Internal implementation details may change at any time

## Best Practices

### Error Handling

```rust
use filesynchub::{Result, Error};

async fn example() -> Result<()> {
    // Use ? operator for error propagation
    let file = File::open("test.txt")
        .await
        .map_err(Error::io)?;
    
    // Use context for better error messages
    process_file(file)
        .await
        .context("Failed to process file")?;
    
    Ok(())
}
```

### Async Operations

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout<F, T>(future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    timeout(Duration::from_secs(30), future)
        .await
        .map_err(|_| Error::timeout())?
}
```

## API Reference

For detailed API documentation, see the following sections:

- [Core API](core.md)
- [Plugin API](plugins.md)
- [Event System](events.md)
- [Configuration](configuration.md)
- [Error Handling](errors.md)

## Contributing

For information about contributing to FileSyncHub, see:

- [Contributing Guide](../contributing/index.md)
- [Code Style](../contributing/code-style.md)
- [Testing Guide](../contributing/testing.md) 