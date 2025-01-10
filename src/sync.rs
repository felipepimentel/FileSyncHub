use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::provider::{CloudProvider, RemoteItem};

pub struct SyncOperation {
    provider: Box<dyn CloudProvider>,
}

impl SyncOperation {
    pub fn new(provider: Box<dyn CloudProvider>) -> Self {
        Self { provider }
    }

    pub async fn handle_local_create(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        println!("Uploading new file: {:?} to {}", local_path, remote_path);
        self.provider.upload_file(local_path, remote_path).await?;
        Ok(())
    }

    pub async fn handle_local_modify(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        println!("Uploading modified file: {:?} to {}", local_path, remote_path);
        self.provider.upload_file(local_path, remote_path).await?;
        Ok(())
    }

    pub async fn handle_local_delete(&self, remote_path: &str) -> Result<()> {
        println!("Deleting remote file: {}", remote_path);
        self.provider.delete(remote_path).await?;
        Ok(())
    }

    pub async fn handle_remote_change(&self, item: RemoteItem, local_base_path: &Path) -> Result<()> {
        let local_path = local_base_path.join(&item.name);

        // Check if the file exists locally
        let exists = local_path.exists();
        
        if item.is_folder {
            if !exists {
                println!("Creating local directory: {:?}", local_path);
                fs::create_dir_all(&local_path).await?;
            }
        } else {
            if exists {
                // Compare modification times and sizes
                let metadata = fs::metadata(&local_path).await?;
                let local_modified = metadata.modified()?.into();
                let local_size = metadata.len();

                if item.modified > local_modified || item.size != local_size {
                    println!("Downloading updated file: {} to {:?}", item.id, local_path);
                    self.provider.download_file(&item.id, &local_path).await?;
                }
            } else {
                println!("Downloading new file: {} to {:?}", item.id, local_path);
                self.provider.download_file(&item.id, &local_path).await?;
            }
        }

        Ok(())
    }

    pub fn get_remote_path(&self, local_path: &Path, mapping: &crate::config::FolderMapping) -> Option<String> {
        local_path
            .strip_prefix(&mapping.local_path)
            .ok()
            .map(|relative_path| {
                format!(
                    "{}/{}",
                    mapping.remote_path.trim_end_matches('/'),
                    relative_path.to_string_lossy()
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FolderMapping;
    use std::path::PathBuf;
    use async_trait::async_trait;
    use chrono::Utc;
    use tokio::sync::mpsc::Sender;

    #[test]
    fn test_get_remote_path() {
        let provider = MockProvider::new();
        let sync_op = SyncOperation::new(Box::new(provider));

        let mapping = FolderMapping {
            local_path: PathBuf::from("/local/sync"),
            remote_path: String::from("/remote/sync"),
        };

        let local_path = PathBuf::from("/local/sync/docs/file.txt");
        let remote_path = sync_op.get_remote_path(&local_path, &mapping);

        assert_eq!(remote_path, Some(String::from("/remote/sync/docs/file.txt")));
    }
}

#[cfg(test)]
struct MockProvider {
    mappings: Vec<FolderMapping>,
}

#[cfg(test)]
impl MockProvider {
    fn new() -> Self {
        Self {
            mappings: vec![],
        }
    }
}

#[cfg(test)]
#[async_trait]
impl CloudProvider for MockProvider {
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn list_files(&self, _remote_path: &str) -> Result<Vec<RemoteItem>> {
        Ok(vec![])
    }

    async fn upload_file(&self, _local_path: &Path, _remote_path: &str) -> Result<RemoteItem> {
        Ok(RemoteItem {
            name: "test.txt".to_string(),
            id: "test-id".to_string(),
            size: 0,
            modified: Utc::now(),
            is_folder: false,
        })
    }

    async fn download_file(&self, _remote_path: &str, _local_path: &Path) -> Result<()> {
        Ok(())
    }

    async fn create_directory(&self, _remote_path: &str) -> Result<RemoteItem> {
        Ok(RemoteItem {
            name: "test-dir".to_string(),
            id: "test-dir-id".to_string(),
            size: 0,
            modified: Utc::now(),
            is_folder: true,
        })
    }

    async fn delete(&self, _remote_path: &str) -> Result<()> {
        Ok(())
    }

    async fn exists(&self, _remote_path: &str) -> Result<bool> {
        Ok(true)
    }

    async fn get_item(&self, _remote_path: &str) -> Result<Option<RemoteItem>> {
        Ok(None)
    }

    async fn watch_local_changes(&self, _local_path: &Path, _tx: Sender<ChangeType>) -> Result<()> {
        Ok(())
    }

    async fn watch_remote_changes(&self, _remote_path: &str, _tx: Sender<RemoteItem>) -> Result<()> {
        Ok(())
    }

    async fn get_mappings(&self) -> Vec<FolderMapping> {
        self.mappings.clone()
    }
} 