---
layout: default
title: Plugin Development
parent: User Guide
nav_order: 5
---

# Plugin Development Guide

This guide explains how to create custom plugins for FileSyncHub.

## Plugin Architecture

FileSyncHub uses a trait-based plugin system. Each plugin must implement the `SyncPlugin` trait, which defines the core functionality required for file synchronization.

### Core Components

```rust
use async_trait::async_trait;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait SyncPlugin: Send + Sync {
    async fn init(&mut self) -> Result<()>;
    async fn upload_file(&self, path: &Path) -> Result<()>;
    async fn download_file(&self, path: &Path) -> Result<()>;
    async fn list_files(&self) -> Result<Vec<FileInfo>>;
    async fn delete_file(&self, path: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub hash: String,
}
```

## Creating a Plugin

### Basic Structure

Here's a template for creating a new plugin:

```rust
use filesynchub::plugin::{SyncPlugin, FileInfo, Result};
use async_trait::async_trait;
use std::path::Path;

pub struct MyPlugin {
    config: MyPluginConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct MyPluginConfig {
    api_key: String,
    base_url: String,
    timeout: u64,
}

impl MyPlugin {
    pub fn new(config: MyPluginConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SyncPlugin for MyPlugin {
    async fn init(&mut self) -> Result<()> {
        // Initialize your plugin
        self.client = Client::new_with_config(&self.config).await?;
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

    async fn delete_file(&self, path: &Path) -> Result<()> {
        self.client.delete(path).await?;
        Ok(())
    }
}
```

### Configuration

Create a configuration structure for your plugin:

```rust
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct MyPluginConfig {
    // Required fields
    api_key: String,
    base_url: String,

    // Optional fields with defaults
    #[serde(default = "default_timeout")]
    timeout: Duration,

    #[serde(default)]
    max_retries: u32,

    #[serde(default = "default_chunk_size")]
    chunk_size: usize,
}

impl Default for MyPluginConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::new(),
            timeout: default_timeout(),
            max_retries: 3,
            chunk_size: default_chunk_size(),
        }
    }
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_chunk_size() -> usize {
    16 * 1024 * 1024  // 16MB
}
```

## Integration

### Plugin Registration

Register your plugin with FileSyncHub:

```rust
use filesynchub::{Config, FileSyncHub};

#[tokio::main]
async fn main() -> Result<()> {
    // Create plugin configuration
    let plugin_config = MyPluginConfig {
        api_key: "your-api-key".to_string(),
        base_url: "https://api.example.com".to_string(),
        ..Default::default()
    };

    // Create FileSyncHub configuration
    let config = Config::builder()
        .sync_dir("~/Documents")
        .add_plugin("my_plugin", plugin_config)
        .build()?;

    // Initialize FileSyncHub with your plugin
    let mut sync_hub = FileSyncHub::new(config);
    sync_hub.init().await?;
    sync_hub.start().await?;

    Ok(())
}
```

### Error Handling

Implement proper error handling:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyPluginError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<MyPluginError> for filesynchub::Error {
    fn from(err: MyPluginError) -> Self {
        filesynchub::Error::Plugin(Box::new(err))
    }
}
```

## Testing

### Unit Tests

Write comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use mockall::predicate::*;

    #[test]
    async fn test_upload_file() -> Result<()> {
        let mut mock_client = MockClient::new();
        mock_client
            .expect_upload()
            .with(eq("test.txt"))
            .returning(|_| Ok(()));

        let plugin = MyPlugin {
            config: MyPluginConfig::default(),
            client: mock_client,
        };

        let test_file = tempfile::NamedTempFile::new()?;
        tokio::fs::write(&test_file, b"test data").await?;

        plugin.upload_file(test_file.path()).await?;
        Ok(())
    }

    #[test]
    async fn test_download_file() -> Result<()> {
        let mut mock_client = MockClient::new();
        mock_client
            .expect_download()
            .with(eq("test.txt"))
            .returning(|_| Ok(vec![1, 2, 3]));

        let plugin = MyPlugin {
            config: MyPluginConfig::default(),
            client: mock_client,
        };

        let test_file = tempfile::NamedTempFile::new()?;
        plugin.download_file(test_file.path()).await?;

        let content = tokio::fs::read(test_file.path()).await?;
        assert_eq!(content, vec![1, 2, 3]);
        Ok(())
    }
}
```

### Integration Tests

Create integration tests:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_full_sync_cycle() -> Result<()> {
        // Set up test environment
        let temp_dir = tempfile::tempdir()?;
        let config = MyPluginConfig {
            api_key: std::env::var("TEST_API_KEY")?,
            base_url: "https://api.test.example.com".to_string(),
            ..Default::default()
        };

        let mut plugin = MyPlugin::new(config);
        plugin.init().await?;

        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&test_file, b"test data").await?;

        // Test upload
        plugin.upload_file(&test_file).await?;

        // List files
        let files = plugin.list_files().await?;
        assert!(files.iter().any(|f| f.path == test_file));

        // Delete file
        tokio::fs::remove_file(&test_file).await?;

        // Test download
        plugin.download_file(&test_file).await?;
        assert!(test_file.exists());

        Ok(())
    }
}
```

## Best Practices

### Performance

1. Implement efficient chunked uploads/downloads:

```rust
impl MyPlugin {
    async fn upload_large_file(&self, path: &Path) -> Result<()> {
        let file = tokio::fs::File::open(path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.config.chunk_size];

        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            self.client.upload_chunk(&buffer[..n]).await?;
        }
        Ok(())
    }
}
```

2. Use connection pooling:

```rust
impl MyPlugin {
    fn create_client(&self) -> Client {
        ClientBuilder::new()
            .pool_max_idle_per_host(self.config.max_connections)
            .timeout(self.config.timeout)
            .build()
            .unwrap()
    }
}
```

### Security

1. Implement secure authentication:

```rust
impl MyPlugin {
    async fn authenticate(&self) -> Result<()> {
        let credentials = Credentials::new(
            &self.config.api_key,
            Some(&self.config.api_secret),
        );
        
        self.client.authenticate(credentials).await?;
        Ok(())
    }
}
```

2. Handle sensitive data:

```rust
impl MyPlugin {
    fn protect_sensitive_data(&self) -> Result<()> {
        // Use secure storage for tokens
        let keyring = Keyring::new("filesynchub", "my_plugin");
        keyring.set_password(&self.config.api_key)?;
        
        // Clear sensitive data from memory
        self.config.api_key.zeroize();
        Ok(())
    }
}
```

### Error Handling

1. Implement retry logic:

```rust
impl MyPlugin {
    async fn with_retry<F, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < self.config.max_retries => {
                    attempts += 1;
                    tokio::time::sleep(self.get_backoff(attempts)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn get_backoff(&self, attempt: u32) -> Duration {
        Duration::from_secs(2u64.pow(attempt - 1))
    }
}
```

## Publishing

### Documentation

1. Add comprehensive documentation:

```rust
/// MyPlugin provides integration with Example Cloud Storage.
///
/// # Examples
///
/// ```rust
/// use filesynchub::plugin::MyPlugin;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let config = MyPluginConfig::default();
///     let mut plugin = MyPlugin::new(config);
///     plugin.init().await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MyPlugin {
    // ...
}
```

2. Include usage examples:

```rust
/// Uploads a file to the remote storage.
///
/// # Arguments
///
/// * `path` - The path to the file to upload
///
/// # Examples
///
/// ```rust
/// # use filesynchub::plugin::MyPlugin;
/// # use std::path::Path;
/// #
/// # async fn example() -> Result<()> {
/// let plugin = MyPlugin::new(MyPluginConfig::default());
/// plugin.upload_file(Path::new("test.txt")).await?;
/// # Ok(())
/// # }
/// ```
async fn upload_file(&self, path: &Path) -> Result<()> {
    // Implementation
}
```

### Distribution

1. Create a Cargo.toml for your plugin:

```toml
[package]
name = "filesynchub-myplugin"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]
edition = "2021"
description = "MyPlugin for FileSyncHub"
license = "MIT"
repository = "https://github.com/yourusername/filesynchub-myplugin"

[dependencies]
filesynchub = "0.1"
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
```

2. Publish to crates.io:

```bash
# Update version in Cargo.toml
cargo test
cargo fmt
cargo clippy
cargo publish
``` 