use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::{CloudProvider, RemoteItem, ChangeType, FolderMapping};

pub struct OneDriveProvider {
    #[allow(dead_code)]
    client_id: String,
    #[allow(dead_code)]
    client_secret: String,
    mappings: Vec<FolderMapping>,
}

impl OneDriveProvider {
    pub fn new(client_id: String, client_secret: String, mappings: Vec<FolderMapping>) -> Self {
        Self {
            client_id,
            client_secret,
            mappings,
        }
    }
}

#[async_trait]
impl CloudProvider for OneDriveProvider {
    async fn list_files(&self, _remote_path: &str) -> Result<Vec<RemoteItem>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn upload_file(&self, _local_path: &Path, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement
        unimplemented!()
    }

    async fn download_file(&self, _remote_path: &str, _local_path: &Path) -> Result<()> {
        // TODO: Implement
        unimplemented!()
    }

    async fn create_directory(&self, _remote_path: &str) -> Result<RemoteItem> {
        // TODO: Implement
        unimplemented!()
    }

    async fn delete(&self, _remote_path: &str) -> Result<()> {
        // TODO: Implement
        unimplemented!()
    }

    async fn exists(&self, _remote_path: &str) -> Result<bool> {
        // TODO: Implement
        Ok(false)
    }

    async fn get_item(&self, _remote_path: &str) -> Result<Option<RemoteItem>> {
        // TODO: Implement
        Ok(None)
    }

    async fn watch_local_changes(&self, _local_path: &Path, _tx: mpsc::Sender<ChangeType>) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    async fn watch_remote_changes(&self, _remote_path: &str, _tx: mpsc::Sender<RemoteItem>) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    async fn get_mappings(&self) -> Vec<FolderMapping> {
        self.mappings.clone()
    }
} 