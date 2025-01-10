use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::{ChangeType, CloudProvider, RemoteItem, FolderMapping};

pub struct OneDriveProvider {
    client_id: String,
    client_secret: String,
    mappings: Vec<FolderMapping>,
}

impl OneDriveProvider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            mappings: Vec::new(),
        }
    }
}

#[async_trait]
impl CloudProvider for OneDriveProvider {
    async fn list_files(&self, _remote_path: &str) -> Result<Vec<RemoteItem>> {
        // TODO: Implement list_files
        Ok(Vec::new())
    }

    async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement upload_file
        Ok(RemoteItem {
            id: "".to_string(),
            name: "".to_string(),
            is_folder: false,
            size: 0,
            modified: None,
        })
    }

    async fn download_file(&self, _remote_path: &str, _local_path: &Path) -> Result<()> {
        // TODO: Implement download_file
        Ok(())
    }

    async fn create_directory(&self, remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement create_directory
        Ok(RemoteItem {
            id: "".to_string(),
            name: "".to_string(),
            is_folder: true,
            size: 0,
            modified: None,
        })
    }

    async fn delete(&self, _remote_path: &str) -> Result<()> {
        // TODO: Implement delete
        Ok(())
    }

    async fn exists(&self, _remote_path: &str) -> Result<bool> {
        // TODO: Implement exists
        Ok(false)
    }

    async fn get_item(&self, _remote_path: &str) -> Result<Option<RemoteItem>> {
        // TODO: Implement get_item
        Ok(None)
    }

    async fn watch_local_changes(&self, _local_path: &Path, _tx: mpsc::Sender<ChangeType>) -> Result<()> {
        // TODO: Implement watch_local_changes
        Ok(())
    }

    async fn watch_remote_changes(&self, remote_path: &str, tx: mpsc::Sender<RemoteItem>) -> Result<()> {
        // TODO: Implement watch_remote_changes
        Ok(())
    }

    async fn get_mappings(&self) -> Vec<FolderMapping> {
        self.mappings.clone()
    }
} 